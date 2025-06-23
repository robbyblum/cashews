alter table game_events add column pitcher_id text default null;
alter table game_events add column batter_id text default null;

alter table game_events add column season smallint not null default -1;
alter table game_events add column day smallint not null default -1;

create index game_events_sd_idx on game_events(season, day, index);
create index game_events_season_pitcher_idx on game_events(season, pitcher_id);
create index game_events_season_batter_idx on game_events(season, batter_id);