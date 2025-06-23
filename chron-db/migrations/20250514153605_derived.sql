create table games (
    game_id text primary key,
    season int,
    day int,
    home_team_id text,
    away_team_id text,
    state text,
    event_count int,
    last_update jsonb
);

create table game_events (
    game_id text not null,
    index int not null,
    data jsonb not null,
    primary key (game_id, index)
);

create table game_player_stats (
    game_id text not null,
    team_id text not null,
    player_id text not null,
    season smallint not null,
    day smallint not null,
    data jsonb not null,
    primary key (game_id, team_id, player_id)
);