create index game_player_stats_by_player_idx on game_player_stats(player_id, season, day);
create index game_player_stats_by_team_idx on game_player_stats(team_id, season, day);
create index game_player_stats_by_season_day_idx on game_player_stats(season, day);
create index game_player_stats_data_idx on game_player_stats using gin(data);

create index versions_sort_idx on versions(kind, valid_from, entity_id);
create index versions_single_idx on versions(kind, entity_id, valid_to);

