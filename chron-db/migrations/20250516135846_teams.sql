create table teams (
    team_id text primary key,
    league_id text,
    location text,
    name text,
    full_location text,
    emoji text,
    color text,
    abbreviation text
);
create index teams_by_league_idx on teams(league_id);

create table leagues (
    league_id text primary key,
    league_type text,
    name text,
    color text,
    emoji text
);