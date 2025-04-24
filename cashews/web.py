from fastapi import FastAPI, Request, HTTPException
from fastapi.templating import Jinja2Templates
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware

from cashews import utils
import datetime, json
app = FastAPI(docs_url=None, redoc_url=None)
app.mount("/static", StaticFiles(directory="static"), name="static")
templates = Jinja2Templates(directory="templates")


app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

def filter_props(props):
    def inner(obj):
        return {k: v for k, v in obj.items() if k in props}
    return inner

def to_delta(timestamp):
    dt = datetime.datetime.fromtimestamp(timestamp / 1000, tz=datetime.UTC)
    delta = datetime.datetime.now(datetime.UTC) - dt
    tsecs = delta.total_seconds()

    if tsecs <= 60:
        return "now"

    hours = int(tsecs // (60*60))
    mins = int((tsecs % (60*60)) // (60))
    # secs = int((tsecs % (60)))
    out_str = ""
    if hours > 0:
        out_str += f"{hours}h "
    if mins > 0:
        out_str += f"{mins}m "
    
    out_str += "ago"
    return out_str

# @app.get("/api/leagues")
# async def api_leagues():
#     keep_properties = set(["Color", "Emoji", "LeagueType", "Name"])
#     leagues = utils.get_all_as_dict("league", filter_props(keep_properties))
#     return leagues

@app.get("/api/allteams")
async def api_teams(league: str | None = None):
    keep_properties = set(["Color", "Emoji", "FullLocation", "Location", "League", "Name", "Record", "_id"])
    teams = utils.get_all_as_dict("team", filter_props(keep_properties))
    if league:
        teams = {k: v for k, v in teams.items() if v["League"] == league}
    return teams

@app.get("/api/games")
async def api_games(team: str | None = None, season: int | None = None, day: int | None = None, state: str | None = None):
    all_teams = utils.get_all_as_dict("team")
    with utils.db() as con:
        cur = con.cursor()

        filters = []
        params = []

        if team:
            filters.append("(home_team_id = ? or away_team_id = ?)")
            params.append(team)
            params.append(team)
        if season is not None:
            filters.append("season = ?")
            params.append(season)
        if day is not None:
            filters.append("day = ?")
            params.append(day)
        if state is not None:
            filters.append("state = ?")
            params.append(state)
        
        where_clause = f"where {' and '.join(filters)}" if filters else ''
        res = cur.execute(f"select id, season, day, home_team_id, away_team_id, state, last_update from games {where_clause}", params).fetchall()
    out = []
    for game_id, season, day, home_team_id, away_team_id, state, last_update in res:
        # data = utils.decode_json(data_blob)

        # team = utils.get_object()
        league = all_teams[home_team_id]["League"]

        out.append({
            "game_id": game_id,
            "season": season,
            "day": day,
            "home_team_id": home_team_id,
            "away_team_id": away_team_id,
            "last_update": last_update,
            "state": state,
            # "league_id": league
        })
    return out
        
def formatted_and_iso(timestamp_secs):
    dt = datetime.datetime.fromtimestamp(timestamp_secs, tz=datetime.UTC)
    formatted = dt.strftime("%Y-%m-%d %H:%M") + " UTC"
    iso = dt.isoformat()
    return formatted, iso

@app.get("/games/{team_id}", response_class=HTMLResponse)
async def games_by_team(request: Request, team_id: str):
    team = utils.get_object("team", team_id)
    if not team:
        raise HTTPException(status_code=404, detail="team not found")
    
    games = list(utils.get_all_games_for_team(team_id))

    def sort_key(row):
        _, game = row
        return game["Season"], game["Day"]
    games.sort(key=sort_key, reverse=True)

    games_list = []
    for game_id, game in games:
        game["_id"] = game_id
        game["last"] = game["EventLog"][-1]

        game_timestamp = utils.id_timestamp(game_id)
        game["time_formatted"], game["time_iso"] = formatted_and_iso(game_timestamp / 1000)

        home_pitchers = []
        away_pitchers = []
        for evt in game["EventLog"]:
            if evt["inning_side"] == 0:
                if evt["pitcher"] and evt["pitcher"] not in home_pitchers:
                    home_pitchers.append(evt["pitcher"])
            if evt["inning_side"] == 1:
                if evt["pitcher"] and evt["pitcher"] not in away_pitchers:
                    away_pitchers.append(evt["pitcher"])

        # game["timestamp"]
        game["away_pitchers"] = ", ".join(away_pitchers)
        game["home_pitchers"] = ", ".join(home_pitchers)
        games_list.append(game)

    return templates.TemplateResponse(
        request=request, name="games.html", context={"team": team, "games": games_list}
    )

def get_all_locations():
    with utils.db() as con:
        cur = con.cursor()
        res = cur.execute("select loc, data from locations").fetchall()
        return {loc: json.loads(data) for loc, data in res}

@app.get("/api/locations")
async def locations():
    return get_all_locations()

@app.get("/api/playersbyteam/{team_id}")
async def players_by_team(team_id: str):
    team = utils.get_object("team", team_id)
    if not team:
        raise HTTPException(status_code=404, detail="team not found")

    player_ids = utils.team_player_ids(team)

    players = {}
    for player_id in player_ids:
        player = utils.get_object("player", player_id)
        if not player:
            continue
        player["_last_update"] = utils.get_last_update("player", player_id)
        players[player_id] = player
    return {
        "team": team,
        "players": players
    }

@app.get("/api/teamlocations")
async def teamlocations():
    locations = get_all_locations()

    teams = []
    for team_id, team, _ in utils.get_all("team"):
        teams.append({
            "id": team_id,
            "location": locations.get(team["FullLocation"]),
            "name": utils.team_name(team),
            "emoji": team["Emoji"],
            "color": team["Color"]
        })
    return teams

@app.get("/map", response_class=HTMLResponse)
async def map(request: Request):
    return templates.TemplateResponse(
        request=request, name="map.html"
    )

@app.get("/teams", response_class=HTMLResponse)
async def teams(request: Request):
    state = utils.get_object("state", "state")
    leagues_dict = utils.get_all_as_dict("league")
    teams_dict = utils.get_all_as_dict("team")

    def team_sort(team):
        return utils.team_name(team)
    
    time = utils.fetch_time()
    season = time["season_number"]
    day = time["season_day"]

    game_datas = utils.get_all_game_data(season, day)
    game_ids_by_team = {}
    for id, away_team_id, home_team_id, _, _ in game_datas:
        game_ids_by_team[away_team_id] = id
        game_ids_by_team[home_team_id] = id

    leagues = []
    all_teams = []
    for league_id in state["GreaterLeagues"] + state["LesserLeagues"]:
        league = leagues_dict[league_id]

        league_teams = []
        for team_id in league["Teams"]:
            if team_id in ["68070065ce5a952465d7c177", "6807014aee9f269dec724ea2"]:
                # destroy my mistakes
                continue
            if team_id not in teams_dict:
                print("!!! couldn't find team:", team_id)
                continue
            team = teams_dict[team_id]

            record = team["Record"]["Regular Season"]
            team["wins"] = record["Wins"]
            team["losses"] = record["Losses"]
            team["rd"] = record["RunDifferential"]
            team["league"] = league["Name"]
            team["league_emoji"] = league["Emoji"]
            team["player_count"] = len([1 for p in team["Players"] if p["PlayerID"] != "#"])

            team_timestamp = utils.id_timestamp(team_id)
            team["created_formatted"], team["created_iso"] = formatted_and_iso(team_timestamp / 1000)

            team["last_updated"] = to_delta(team["_last_updated"])

            team["game_id"] = game_ids_by_team.get(team_id)
            
            league_teams.append(team)
            all_teams.append(team)

        
        league_teams.sort(key=team_sort)
        leagues.append({
            "name": league["Name"],
            "emoji": league["Emoji"],
            "teams": league_teams
        })
    
    def league_sort(league):
        return league["name"]
    
    all_teams.sort(key=team_sort)
    leagues.sort(key=league_sort)
    
    return templates.TemplateResponse(
        request=request, name="teams.html", context={"leagues": leagues, "teams": all_teams}
    )

@app.get("/team/{team_id}/stats", response_class=HTMLResponse)
async def stats(request: Request, team_id: str):
    team = utils.get_object("team", team_id)
    if not team:
        raise HTTPException(status_code=404, detail="team not found")

    player_ids = utils.team_player_ids(team)
    

    # all_players = utils.get_all_as_dict("player")
    all_teams = utils.get_all_as_dict("team")


    import time
    from cashews import stats
    ttl = int(time.time() // 60)
    league_agg = stats.league_agg_stats(ttl)

    stats_by_player = {row["player_id"]: row for row in stats.all_player_stats()}
    
    players = []
    for i, player_id in enumerate(player_ids):
        player = utils.get_object("player", player_id)
        team_id = player["TeamID"]
        team = all_teams.get(team_id)

        player_dict = {
            "player_id": player_id,
            "position": player["Position"],
            "idx": i,
            "name": utils.player_name(player),
            "team_name": utils.team_name(team) if team else "Null Team",
            "team_emoji": team["Emoji"] if team else "❓",
            "stats": stats_by_player.get(player_id, None)
        }
        players.append(player_dict)

    def format_int(number):
        return str(number)
    
    def format_float3(number):
        if type(number) == int or type(number) == float:
            return f"{number:.03f}"
        return ""
    
    def format_float2(number):
        if type(number) == int or type(number) == float:
            return f"{number:.02f}"
        return ""
    
    def format_ip(number):
        ip_whole = int(number)
        ip_remainder = int((number - ip_whole) / 3 * 10)
        if ip_remainder == 0:
            ip_str = f"{ip_whole}"
        else:
            ip_str = f"{ip_whole}.{ip_remainder}"
        return ip_str


    defs = [
        ("PA", "plate_appearances", None, format_int),
        ("BA", "ba", True, format_float3),
        ("OBP", "obp", True, format_float3),
        ("SLG", "slg", True, format_float3),
        ("OPS", "ops", True, format_float3),
        ("SB", "stolen_bases", None, format_int),
        ("CS", "caught_stealing", None, format_int),
        ("SB%", "sb_success", True, format_float3),

        ("IP", "ip", None, format_ip),
        ("ERA", "era", False, format_float2),
        ("WHIP", "whip", False, format_float2),
        ("HR/9", "hr9", False, format_float2),
        ("BB/9", "bb9", False, format_float2),
        ("K/9", "k9", True, format_float2),
        ("H/9", "h9", False, format_float2),
    ]

    defs2 = []
    for name, key, up_good, formatter in defs:
        defs2.append({
            "name": name,
            "key": key,
            "up_good": up_good,
            "league_avg": league_agg[key]["avg"],
            "league_stddev": league_agg[key]["stddev"],
            "format": formatter
        })

    def color_stat(key, value, avg, stddev, up_good):
        if value is None:
            return ""
        if up_good is None:
            return ""
        if stddev < 0.0001:
            stddev_diffs = 0
        else:
            stddev_diffs = (value - avg) / stddev
        if not up_good:
            stddev_diffs = -stddev_diffs
        
        if stddev_diffs < -2.5:
            # -2.5 or below
            return "text-red-800"
        if stddev_diffs < -1.5:
            # -2.5 to -1.5
            return "text-red-700"
        if stddev_diffs < -0.5:
            # -1.5 to -0.5
            return "text-orange-700"
        if stddev_diffs < 0.5:
            # -0.5 to 0.5
            return "text-yellow-700"
        if stddev_diffs < 1.5:
            # 0.5 to 1.5
            return "text-green-700"
        # 1.5 or above
        return "text-green-800"

    print("render start")
    return templates.TemplateResponse(
        request=request, name="stats.html", context={"players": players, "team": team, "league": league_agg, "stat_defs": defs2, "color_stat": color_stat}
    )