import requests, datetime

s = requests.Session()
teams = []
next_page = ""
while next_page is not None:
    resp = s.get("https://freecashe.ws/api/chron/v0/entities?kind=team&count=1000" + (f"&page={next_page}" if next_page else "")).json()
    next_page = resp["next_page"]
    teams += resp["items"]
    print(len(teams))

ends_per_day = {}
games_per_day = {}
for t in teams:
    for feed_item in t["data"].get("Feed", []):
        if feed_item["type"] == "game" and "FINAL" in feed_item["text"]:
            game_ids =  [l["id"] for l in feed_item["links"] if l["type"] == "game"]
            if not game_ids:
                continue
            game_id = game_ids[0]
            end_time = feed_item["ts"]
            day = feed_item["day"]
            if day not in ends_per_day:
                ends_per_day[day] = []
            if day not in games_per_day:
                games_per_day[day] = []
            ends_per_day[day].append(end_time)
            games_per_day[day].append(game_id)

for day in sorted(ends_per_day.keys()):
    first_end = min(ends_per_day[day])
    first_end = datetime.datetime.fromisoformat(first_end)
    
    last_start_game = min(games_per_day[day])
    last_start = int(last_start_game[:8], 16)
    last_start = datetime.datetime.fromtimestamp(last_start, tz=datetime.UTC)

    game = requests.get("https://mmolb.com/api/game/" + last_start_game).json()
    ticks = len(game["EventLog"])

    duration = first_end - last_start
    tickrate = duration / ticks
    print(f"day {day}: fastest game took {ticks} ticks ({duration}), avg tickrate = {tickrate}")