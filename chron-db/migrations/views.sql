drop materialized view if exists game_player_stats_league_aggregate cascade;
drop materialized view if exists game_player_stats_global_aggregate cascade;
drop materialized view if exists pitches cascade;
 
drop view if exists league_percentiles;
drop view if exists game_player_stats_advanced;

create or replace view game_player_stats_advanced as
with helper as (
    select
        coalesce((data->'allowed_stolen_bases')::int, 0) as allowed_stolen_bases,
        coalesce((data->'allowed_stolen_bases_risp')::int, 0) as allowed_stolen_bases_risp,
        coalesce((data->'appearances')::int, 0) as appearances,
        coalesce((data->'assists')::int, 0) as assists,
        coalesce((data->'assists_risp')::int, 0) as assists_risp,
        coalesce((data->'at_bats')::int, 0) as at_bats,
        coalesce((data->'at_bats_risp')::int, 0) as at_bats_risp,
        coalesce((data->'batters_faced')::int, 0) as batters_faced,
        coalesce((data->'batters_faced_risp')::int, 0) as batters_faced_risp,
        coalesce((data->'blown_saves')::int, 0) as blown_saves,
        coalesce((data->'caught_double_play')::int, 0) as caught_double_play,
        coalesce((data->'caught_double_play_risp')::int, 0) as caught_double_play_risp,
        coalesce((data->'caught_stealing')::int, 0) as caught_stealing,
        coalesce((data->'caught_stealing_risp')::int, 0) as caught_stealing_risp,
        coalesce((data->'complete_games')::int, 0) as complete_games,
        coalesce((data->'double_plays')::int, 0) as double_plays,
        coalesce((data->'double_plays_risp')::int, 0) as double_plays_risp,
        coalesce((data->'doubles')::int, 0) as doubles,
        coalesce((data->'doubles_risp')::int, 0) as doubles_risp,
        coalesce((data->'earned_runs')::int, 0) as earned_runs,
        coalesce((data->'earned_runs_risp')::int, 0) as earned_runs_risp,
        coalesce((data->'errors')::int, 0) as errors,
        coalesce((data->'errors_risp')::int, 0) as errors_risp,
        coalesce((data->'field_out')::int, 0) as field_out,
        coalesce((data->'field_out_risp')::int, 0) as field_out_risp,
        coalesce((data->'fielders_choice')::int, 0) as fielders_choice,
        coalesce((data->'fielders_choice_risp')::int, 0) as fielders_choice_risp,
        coalesce((data->'flyouts')::int, 0) as flyouts,
        coalesce((data->'flyouts_risp')::int, 0) as flyouts_risp,
        coalesce((data->'force_outs')::int, 0) as force_outs,
        coalesce((data->'force_outs_risp')::int, 0) as force_outs_risp,
        coalesce((data->'games_finished')::int, 0) as games_finished,
        coalesce((data->'grounded_into_double_play')::int, 0) as grounded_into_double_play,
        coalesce((data->'grounded_into_double_play_risp')::int, 0) as grounded_into_double_play_risp,
        coalesce((data->'groundout')::int, 0) as groundout,
        coalesce((data->'groundout_risp')::int, 0) as groundout_risp,
        coalesce((data->'hit_batters')::int, 0) as hit_batters,
        coalesce((data->'hit_batters_risp')::int, 0) as hit_batters_risp,
        coalesce((data->'hit_by_pitch')::int, 0) as hit_by_pitch,
        coalesce((data->'hit_by_pitch_risp')::int, 0) as hit_by_pitch_risp,
        coalesce((data->'hits_allowed')::int, 0) as hits_allowed,
        coalesce((data->'hits_allowed_risp')::int, 0) as hits_allowed_risp,
        coalesce((data->'home_runs')::int, 0) as home_runs,
        coalesce((data->'home_runs_allowed')::int, 0) as home_runs_allowed,
        coalesce((data->'home_runs_allowed_risp')::int, 0) as home_runs_allowed_risp,
        coalesce((data->'home_runs_risp')::int, 0) as home_runs_risp,
        coalesce((data->'inherited_runners')::int, 0) as inherited_runners,
        coalesce((data->'inherited_runners_risp')::int, 0) as inherited_runners_risp,
        coalesce((data->'inherited_runs_allowed')::int, 0) as inherited_runs_allowed,
        coalesce((data->'inherited_runs_allowed_risp')::int, 0) as inherited_runs_allowed_risp,
        coalesce((data->'left_on_base')::int, 0) as left_on_base,
        coalesce((data->'left_on_base_risp')::int, 0) as left_on_base_risp,
        coalesce((data->'lineouts')::int, 0) as lineouts,
        coalesce((data->'lineouts_risp')::int, 0) as lineouts_risp,
        coalesce((data->'losses')::int, 0) as losses,
        coalesce((data->'mound_visits')::int, 0) as mound_visits,
        coalesce((data->'no_hitters')::int, 0) as no_hitters,
        coalesce((data->'outs')::int, 0) as outs,
        coalesce((data->'perfect_games')::int, 0) as perfect_games,
        coalesce((data->'pitches_thrown')::int, 0) as pitches_thrown,
        coalesce((data->'pitches_thrown_risp')::int, 0) as pitches_thrown_risp,
        coalesce((data->'plate_appearances')::int, 0) as plate_appearances,
        coalesce((data->'plate_appearances_risp')::int, 0) as plate_appearances_risp,
        coalesce((data->'popouts')::int, 0) as popouts,
        coalesce((data->'popouts_risp')::int, 0) as popouts_risp,
        coalesce((data->'putouts')::int, 0) as putouts,
        coalesce((data->'putouts_risp')::int, 0) as putouts_risp,
        coalesce((data->'quality_starts')::int, 0) as quality_starts,
        coalesce((data->'reached_on_error')::int, 0) as reached_on_error,
        coalesce((data->'reached_on_error_risp')::int, 0) as reached_on_error_risp,
        coalesce((data->'runners_caught_stealing')::int, 0) as runners_caught_stealing,
        coalesce((data->'runners_caught_stealing_risp')::int, 0) as runners_caught_stealing_risp,
        coalesce((data->'runs')::int, 0) as runs,
        coalesce((data->'runs_batted_in')::int, 0) as runs_batted_in,
        coalesce((data->'runs_batted_in_risp')::int, 0) as runs_batted_in_risp,
        coalesce((data->'runs_risp')::int, 0) as runs_risp,
        coalesce((data->'sac_flies')::int, 0) as sac_flies,
        coalesce((data->'sac_flies_risp')::int, 0) as sac_flies_risp,
        coalesce((data->'sacrifice_double_plays')::int, 0) as sacrifice_double_plays,
        coalesce((data->'sacrifice_double_plays_risp')::int, 0) as sacrifice_double_plays_risp,
        coalesce((data->'saves')::int, 0) as saves,
        coalesce((data->'shutouts')::int, 0) as shutouts,
        coalesce((data->'singles')::int, 0) as singles,
        coalesce((data->'singles_risp')::int, 0) as singles_risp,
        coalesce((data->'starts')::int, 0) as starts,
        coalesce((data->'stolen_bases')::int, 0) as stolen_bases,
        coalesce((data->'stolen_bases_risp')::int, 0) as stolen_bases_risp,
        coalesce((data->'strikeouts')::int, 0) as strikeouts,
        coalesce((data->'strikeouts_risp')::int, 0) as strikeouts_risp,
        coalesce((data->'struck_out')::int, 0) as struck_out,
        coalesce((data->'struck_out_risp')::int, 0) as struck_out_risp,
        coalesce((data->'triples')::int, 0) as triples,
        coalesce((data->'triples_risp')::int, 0) as triples_risp,
        coalesce((data->'unearned_runs')::int, 0) as unearned_runs,
        coalesce((data->'unearned_runs_risp')::int, 0) as unearned_runs_risp,
        coalesce((data->'walked')::int, 0) as walked,
        coalesce((data->'walked_risp')::int, 0) as walked_risp,
        coalesce((data->'walks')::int, 0) as walks,
        coalesce((data->'walks_risp')::int, 0) as walks_risp,
        coalesce((data->'wins')::int, 0) as wins,

        coalesce((data->'singles')::int, 0) + coalesce((data->'doubles')::int, 0) + coalesce((data->'triples')::int, 0) + coalesce((data->'home_runs')::int, 0) as hits,
        (coalesce((data->'outs')::real, 0) / 3) as ip,
        game_player_stats.*
    from game_player_stats
)
select
    h.season,
    h.player_id,
    h.team_id,
    -- (select league_id from teams t where t.team_id = h.team_id limit 1),

    sum(plate_appearances) as plate_appearances,
    sum(at_bats) as at_bats,

    sum(hits) as hits,
    sum(doubles) as doubles,
    sum(triples) as triples,
    sum(home_runs) as home_runs,
    sum(walked) as walked,
    sum(struck_out) as struck_out,
    sum(sac_flies) as sac_flies,

    ((sum(hits)::real) / nullif(sum(at_bats)::real, 0)) as ba,
    (sum(hits + walked + hit_by_pitch)::real / nullif(sum(plate_appearances)::real, 0)) as obp,
    (sum(singles + doubles * 2 + triples * 3 + home_runs * 4)::real / nullif(sum(at_bats)::real, 0)) as slg,
    (sum(hits + walked + hit_by_pitch)::real / nullif(sum(plate_appearances)::real, 0) + sum(singles + doubles * 2 + triples * 3 + home_runs * 4)::real / nullif(sum(at_bats)::real, 0)) as ops,
    (sum(singles + doubles + triples)::real / nullif(sum(at_bats - struck_out + sac_flies)::real, 0)) as babip,

    sum(appearances) as appearances,
    sum(starts) as starts,
    sum(ip) as ip,
    sum(outs) as outs,

    sum(strikeouts) as strikeouts,
    sum(walks) as walks,
    sum(hits_allowed) as hits_allowed,
    sum(home_runs_allowed) as home_runs_allowed,
    sum(hit_batters) as hit_batters,
    sum(earned_runs) as earned_runs,
    sum(runs) as runs,

    ((9 * sum(earned_runs))::real / nullif(sum(ip), 0)) as era,
    (sum(walks + hits_allowed)::real / nullif(sum(ip), 0)) as whip,
    (sum(9 * home_runs_allowed)::real / nullif(sum(ip), 0)) as hr9,
    (sum(9 * walks)::real / nullif(sum(ip), 0)) as bb9,
    (sum(9 * strikeouts)::real / nullif(sum(ip), 0)) as k9,
    (sum(9 * hits_allowed)::real / nullif(sum(ip), 0)) as h9,
    (sum(13 * home_runs_allowed + 3 * (walks + hit_batters) - 2 * strikeouts)::real / nullif(sum(ip), 0)) as fip_base,

    sum(stolen_bases) as stolen_bases,
    sum(caught_stealing) as caught_stealing,
    sum(stolen_bases + caught_stealing) as sb_attempts,
    (sum(stolen_bases)::real / nullif(sum(stolen_bases + caught_stealing)::real, 0)) as sb_success,

    sum(errors) as errors,
    (sum(putouts + assists)::real / nullif(sum(putouts + assists + errors)::real, 0)) as fpct
from helper h
group by (season, h.player_id, h.team_id);

-- Materialized view for calculating league-average stats, for use in normalized statistics and the like. Averages over
-- individual leagues, so there will be a bit of variation in how OPS translates to OPS+ for different Lesser Leagues,
-- if you use this. Join this to the team info on league_id to select the relevant league average.
-- particular note: FIP constant is implicitly defined in here: use fip_const = era - fip_base to get it
create materialized view if not exists game_player_stats_league_aggregate as
with helper as (
    select
        coalesce((data->'allowed_stolen_bases')::int, 0) as allowed_stolen_bases,
        coalesce((data->'allowed_stolen_bases_risp')::int, 0) as allowed_stolen_bases_risp,
        coalesce((data->'appearances')::int, 0) as appearances,
        coalesce((data->'assists')::int, 0) as assists,
        coalesce((data->'assists_risp')::int, 0) as assists_risp,
        coalesce((data->'at_bats')::int, 0) as at_bats,
        coalesce((data->'at_bats_risp')::int, 0) as at_bats_risp,
        coalesce((data->'batters_faced')::int, 0) as batters_faced,
        coalesce((data->'batters_faced_risp')::int, 0) as batters_faced_risp,
        coalesce((data->'blown_saves')::int, 0) as blown_saves,
        coalesce((data->'caught_double_play')::int, 0) as caught_double_play,
        coalesce((data->'caught_double_play_risp')::int, 0) as caught_double_play_risp,
        coalesce((data->'caught_stealing')::int, 0) as caught_stealing,
        coalesce((data->'caught_stealing_risp')::int, 0) as caught_stealing_risp,
        coalesce((data->'complete_games')::int, 0) as complete_games,
        coalesce((data->'double_plays')::int, 0) as double_plays,
        coalesce((data->'double_plays_risp')::int, 0) as double_plays_risp,
        coalesce((data->'doubles')::int, 0) as doubles,
        coalesce((data->'doubles_risp')::int, 0) as doubles_risp,
        coalesce((data->'earned_runs')::int, 0) as earned_runs,
        coalesce((data->'earned_runs_risp')::int, 0) as earned_runs_risp,
        coalesce((data->'errors')::int, 0) as errors,
        coalesce((data->'errors_risp')::int, 0) as errors_risp,
        coalesce((data->'field_out')::int, 0) as field_out,
        coalesce((data->'field_out_risp')::int, 0) as field_out_risp,
        coalesce((data->'fielders_choice')::int, 0) as fielders_choice,
        coalesce((data->'fielders_choice_risp')::int, 0) as fielders_choice_risp,
        coalesce((data->'flyouts')::int, 0) as flyouts,
        coalesce((data->'flyouts_risp')::int, 0) as flyouts_risp,
        coalesce((data->'force_outs')::int, 0) as force_outs,
        coalesce((data->'force_outs_risp')::int, 0) as force_outs_risp,
        coalesce((data->'games_finished')::int, 0) as games_finished,
        coalesce((data->'grounded_into_double_play')::int, 0) as grounded_into_double_play,
        coalesce((data->'grounded_into_double_play_risp')::int, 0) as grounded_into_double_play_risp,
        coalesce((data->'groundout')::int, 0) as groundout,
        coalesce((data->'groundout_risp')::int, 0) as groundout_risp,
        coalesce((data->'hit_batters')::int, 0) as hit_batters,
        coalesce((data->'hit_batters_risp')::int, 0) as hit_batters_risp,
        coalesce((data->'hit_by_pitch')::int, 0) as hit_by_pitch,
        coalesce((data->'hit_by_pitch_risp')::int, 0) as hit_by_pitch_risp,
        coalesce((data->'hits_allowed')::int, 0) as hits_allowed,
        coalesce((data->'hits_allowed_risp')::int, 0) as hits_allowed_risp,
        coalesce((data->'home_runs')::int, 0) as home_runs,
        coalesce((data->'home_runs_allowed')::int, 0) as home_runs_allowed,
        coalesce((data->'home_runs_allowed_risp')::int, 0) as home_runs_allowed_risp,
        coalesce((data->'home_runs_risp')::int, 0) as home_runs_risp,
        coalesce((data->'inherited_runners')::int, 0) as inherited_runners,
        coalesce((data->'inherited_runners_risp')::int, 0) as inherited_runners_risp,
        coalesce((data->'inherited_runs_allowed')::int, 0) as inherited_runs_allowed,
        coalesce((data->'inherited_runs_allowed_risp')::int, 0) as inherited_runs_allowed_risp,
        coalesce((data->'left_on_base')::int, 0) as left_on_base,
        coalesce((data->'left_on_base_risp')::int, 0) as left_on_base_risp,
        coalesce((data->'lineouts')::int, 0) as lineouts,
        coalesce((data->'lineouts_risp')::int, 0) as lineouts_risp,
        coalesce((data->'losses')::int, 0) as losses,
        coalesce((data->'mound_visits')::int, 0) as mound_visits,
        coalesce((data->'no_hitters')::int, 0) as no_hitters,
        coalesce((data->'outs')::int, 0) as outs,
        coalesce((data->'perfect_games')::int, 0) as perfect_games,
        coalesce((data->'pitches_thrown')::int, 0) as pitches_thrown,
        coalesce((data->'pitches_thrown_risp')::int, 0) as pitches_thrown_risp,
        coalesce((data->'plate_appearances')::int, 0) as plate_appearances,
        coalesce((data->'plate_appearances_risp')::int, 0) as plate_appearances_risp,
        coalesce((data->'popouts')::int, 0) as popouts,
        coalesce((data->'popouts_risp')::int, 0) as popouts_risp,
        coalesce((data->'putouts')::int, 0) as putouts,
        coalesce((data->'putouts_risp')::int, 0) as putouts_risp,
        coalesce((data->'quality_starts')::int, 0) as quality_starts,
        coalesce((data->'reached_on_error')::int, 0) as reached_on_error,
        coalesce((data->'reached_on_error_risp')::int, 0) as reached_on_error_risp,
        coalesce((data->'runners_caught_stealing')::int, 0) as runners_caught_stealing,
        coalesce((data->'runners_caught_stealing_risp')::int, 0) as runners_caught_stealing_risp,
        coalesce((data->'runs')::int, 0) as runs,
        coalesce((data->'runs_batted_in')::int, 0) as runs_batted_in,
        coalesce((data->'runs_batted_in_risp')::int, 0) as runs_batted_in_risp,
        coalesce((data->'runs_risp')::int, 0) as runs_risp,
        coalesce((data->'sac_flies')::int, 0) as sac_flies,
        coalesce((data->'sac_flies_risp')::int, 0) as sac_flies_risp,
        coalesce((data->'sacrifice_double_plays')::int, 0) as sacrifice_double_plays,
        coalesce((data->'sacrifice_double_plays_risp')::int, 0) as sacrifice_double_plays_risp,
        coalesce((data->'saves')::int, 0) as saves,
        coalesce((data->'shutouts')::int, 0) as shutouts,
        coalesce((data->'singles')::int, 0) as singles,
        coalesce((data->'singles_risp')::int, 0) as singles_risp,
        coalesce((data->'starts')::int, 0) as starts,
        coalesce((data->'stolen_bases')::int, 0) as stolen_bases,
        coalesce((data->'stolen_bases_risp')::int, 0) as stolen_bases_risp,
        coalesce((data->'strikeouts')::int, 0) as strikeouts,
        coalesce((data->'strikeouts_risp')::int, 0) as strikeouts_risp,
        coalesce((data->'struck_out')::int, 0) as struck_out,
        coalesce((data->'struck_out_risp')::int, 0) as struck_out_risp,
        coalesce((data->'triples')::int, 0) as triples,
        coalesce((data->'triples_risp')::int, 0) as triples_risp,
        coalesce((data->'unearned_runs')::int, 0) as unearned_runs,
        coalesce((data->'unearned_runs_risp')::int, 0) as unearned_runs_risp,
        coalesce((data->'walked')::int, 0) as walked,
        coalesce((data->'walked_risp')::int, 0) as walked_risp,
        coalesce((data->'walks')::int, 0) as walks,
        coalesce((data->'walks_risp')::int, 0) as walks_risp,
        coalesce((data->'wins')::int, 0) as wins,

        coalesce((data->'singles')::int, 0) + coalesce((data->'doubles')::int, 0) + coalesce((data->'triples')::int, 0) + coalesce((data->'home_runs')::int, 0) as hits,
        (coalesce((data->'outs')::real, 0) / 3) as ip,
        game_player_stats.*
    from game_player_stats
)
select
    season,
    league_id,
    -- (select league_id from teams t where t.team_id = h.team_id limit 1),

    sum(outs / 3) as ip,

    sum(plate_appearances) as plate_appearances,
    sum(at_bats) as at_bats,

    ((sum(hits)::real) / nullif(sum(at_bats)::real, 0)) as ba,
    (sum(hits + walked + hit_by_pitch)::real / nullif(sum(plate_appearances)::real, 0)) as obp,
    (sum(singles + doubles * 2 + triples * 3 + home_runs * 4)::real / nullif(sum(at_bats)::real, 0)) as slg,
    (sum(hits + walked + hit_by_pitch)::real / nullif(sum(plate_appearances)::real, 0) + sum(singles + doubles * 2 + triples * 3 + home_runs * 4)::real / nullif(sum(at_bats)::real, 0)) as ops,

    ((9 * sum(earned_runs))::real / nullif(sum(ip), 0)) as era,
    (sum(walks + hits_allowed)::real / nullif(sum(ip), 0)) as whip,
    (sum(9 * home_runs_allowed)::real / nullif(sum(ip), 0)) as hr9,
    (sum(9 * walks)::real / nullif(sum(ip), 0)) as bb9,
    (sum(9 * strikeouts)::real / nullif(sum(ip), 0)) as k9,
    (sum(9 * hits_allowed)::real / nullif(sum(ip), 0)) as h9,
    (sum(13 * home_runs_allowed + 3 * (walks + hit_batters) - 2 * strikeouts)::real / nullif(sum(ip), 0)) as fip_base,

    sum(stolen_bases + caught_stealing) as sb_attempts,
    (sum(stolen_bases)::real / nullif(sum(stolen_bases + caught_stealing)::real, 0)) as sb_success,

    (sum(singles + doubles + triples)::real / nullif(sum(at_bats - struck_out + sac_flies)::real, 0)) as babip,
    (sum(putouts + assists)::real / nullif(sum(putouts + assists + errors)::real, 0)) as fpct
from helper h
         inner join teams t on (t.team_id = h.team_id)
group by (season, league_id)
with data;
create unique index game_player_stats_league_aggregate_idx on game_player_stats_league_aggregate(season, league_id);

-- Materialized view for calculating globally averaged stats. Unlike game_player_stats_league_aggregate, does not group
-- by league, so BA here is (total hits across every league in MMOLB) / (total at-bats across every league in MMOLB) for
-- a given season.
-- particular note: FIP constant is implicitly defined in here: use fip_const = era - fip_base to get it
create materialized view if not exists game_player_stats_global_aggregate as
with helper as (
    select
        coalesce((data->'allowed_stolen_bases')::int, 0) as allowed_stolen_bases,
        coalesce((data->'allowed_stolen_bases_risp')::int, 0) as allowed_stolen_bases_risp,
        coalesce((data->'appearances')::int, 0) as appearances,
        coalesce((data->'assists')::int, 0) as assists,
        coalesce((data->'assists_risp')::int, 0) as assists_risp,
        coalesce((data->'at_bats')::int, 0) as at_bats,
        coalesce((data->'at_bats_risp')::int, 0) as at_bats_risp,
        coalesce((data->'batters_faced')::int, 0) as batters_faced,
        coalesce((data->'batters_faced_risp')::int, 0) as batters_faced_risp,
        coalesce((data->'blown_saves')::int, 0) as blown_saves,
        coalesce((data->'caught_double_play')::int, 0) as caught_double_play,
        coalesce((data->'caught_double_play_risp')::int, 0) as caught_double_play_risp,
        coalesce((data->'caught_stealing')::int, 0) as caught_stealing,
        coalesce((data->'caught_stealing_risp')::int, 0) as caught_stealing_risp,
        coalesce((data->'complete_games')::int, 0) as complete_games,
        coalesce((data->'double_plays')::int, 0) as double_plays,
        coalesce((data->'double_plays_risp')::int, 0) as double_plays_risp,
        coalesce((data->'doubles')::int, 0) as doubles,
        coalesce((data->'doubles_risp')::int, 0) as doubles_risp,
        coalesce((data->'earned_runs')::int, 0) as earned_runs,
        coalesce((data->'earned_runs_risp')::int, 0) as earned_runs_risp,
        coalesce((data->'errors')::int, 0) as errors,
        coalesce((data->'errors_risp')::int, 0) as errors_risp,
        coalesce((data->'field_out')::int, 0) as field_out,
        coalesce((data->'field_out_risp')::int, 0) as field_out_risp,
        coalesce((data->'fielders_choice')::int, 0) as fielders_choice,
        coalesce((data->'fielders_choice_risp')::int, 0) as fielders_choice_risp,
        coalesce((data->'flyouts')::int, 0) as flyouts,
        coalesce((data->'flyouts_risp')::int, 0) as flyouts_risp,
        coalesce((data->'force_outs')::int, 0) as force_outs,
        coalesce((data->'force_outs_risp')::int, 0) as force_outs_risp,
        coalesce((data->'games_finished')::int, 0) as games_finished,
        coalesce((data->'grounded_into_double_play')::int, 0) as grounded_into_double_play,
        coalesce((data->'grounded_into_double_play_risp')::int, 0) as grounded_into_double_play_risp,
        coalesce((data->'groundout')::int, 0) as groundout,
        coalesce((data->'groundout_risp')::int, 0) as groundout_risp,
        coalesce((data->'hit_batters')::int, 0) as hit_batters,
        coalesce((data->'hit_batters_risp')::int, 0) as hit_batters_risp,
        coalesce((data->'hit_by_pitch')::int, 0) as hit_by_pitch,
        coalesce((data->'hit_by_pitch_risp')::int, 0) as hit_by_pitch_risp,
        coalesce((data->'hits_allowed')::int, 0) as hits_allowed,
        coalesce((data->'hits_allowed_risp')::int, 0) as hits_allowed_risp,
        coalesce((data->'home_runs')::int, 0) as home_runs,
        coalesce((data->'home_runs_allowed')::int, 0) as home_runs_allowed,
        coalesce((data->'home_runs_allowed_risp')::int, 0) as home_runs_allowed_risp,
        coalesce((data->'home_runs_risp')::int, 0) as home_runs_risp,
        coalesce((data->'inherited_runners')::int, 0) as inherited_runners,
        coalesce((data->'inherited_runners_risp')::int, 0) as inherited_runners_risp,
        coalesce((data->'inherited_runs_allowed')::int, 0) as inherited_runs_allowed,
        coalesce((data->'inherited_runs_allowed_risp')::int, 0) as inherited_runs_allowed_risp,
        coalesce((data->'left_on_base')::int, 0) as left_on_base,
        coalesce((data->'left_on_base_risp')::int, 0) as left_on_base_risp,
        coalesce((data->'lineouts')::int, 0) as lineouts,
        coalesce((data->'lineouts_risp')::int, 0) as lineouts_risp,
        coalesce((data->'losses')::int, 0) as losses,
        coalesce((data->'mound_visits')::int, 0) as mound_visits,
        coalesce((data->'no_hitters')::int, 0) as no_hitters,
        coalesce((data->'outs')::int, 0) as outs,
        coalesce((data->'perfect_games')::int, 0) as perfect_games,
        coalesce((data->'pitches_thrown')::int, 0) as pitches_thrown,
        coalesce((data->'pitches_thrown_risp')::int, 0) as pitches_thrown_risp,
        coalesce((data->'plate_appearances')::int, 0) as plate_appearances,
        coalesce((data->'plate_appearances_risp')::int, 0) as plate_appearances_risp,
        coalesce((data->'popouts')::int, 0) as popouts,
        coalesce((data->'popouts_risp')::int, 0) as popouts_risp,
        coalesce((data->'putouts')::int, 0) as putouts,
        coalesce((data->'putouts_risp')::int, 0) as putouts_risp,
        coalesce((data->'quality_starts')::int, 0) as quality_starts,
        coalesce((data->'reached_on_error')::int, 0) as reached_on_error,
        coalesce((data->'reached_on_error_risp')::int, 0) as reached_on_error_risp,
        coalesce((data->'runners_caught_stealing')::int, 0) as runners_caught_stealing,
        coalesce((data->'runners_caught_stealing_risp')::int, 0) as runners_caught_stealing_risp,
        coalesce((data->'runs')::int, 0) as runs,
        coalesce((data->'runs_batted_in')::int, 0) as runs_batted_in,
        coalesce((data->'runs_batted_in_risp')::int, 0) as runs_batted_in_risp,
        coalesce((data->'runs_risp')::int, 0) as runs_risp,
        coalesce((data->'sac_flies')::int, 0) as sac_flies,
        coalesce((data->'sac_flies_risp')::int, 0) as sac_flies_risp,
        coalesce((data->'sacrifice_double_plays')::int, 0) as sacrifice_double_plays,
        coalesce((data->'sacrifice_double_plays_risp')::int, 0) as sacrifice_double_plays_risp,
        coalesce((data->'saves')::int, 0) as saves,
        coalesce((data->'shutouts')::int, 0) as shutouts,
        coalesce((data->'singles')::int, 0) as singles,
        coalesce((data->'singles_risp')::int, 0) as singles_risp,
        coalesce((data->'starts')::int, 0) as starts,
        coalesce((data->'stolen_bases')::int, 0) as stolen_bases,
        coalesce((data->'stolen_bases_risp')::int, 0) as stolen_bases_risp,
        coalesce((data->'strikeouts')::int, 0) as strikeouts,
        coalesce((data->'strikeouts_risp')::int, 0) as strikeouts_risp,
        coalesce((data->'struck_out')::int, 0) as struck_out,
        coalesce((data->'struck_out_risp')::int, 0) as struck_out_risp,
        coalesce((data->'triples')::int, 0) as triples,
        coalesce((data->'triples_risp')::int, 0) as triples_risp,
        coalesce((data->'unearned_runs')::int, 0) as unearned_runs,
        coalesce((data->'unearned_runs_risp')::int, 0) as unearned_runs_risp,
        coalesce((data->'walked')::int, 0) as walked,
        coalesce((data->'walked_risp')::int, 0) as walked_risp,
        coalesce((data->'walks')::int, 0) as walks,
        coalesce((data->'walks_risp')::int, 0) as walks_risp,
        coalesce((data->'wins')::int, 0) as wins,

        coalesce((data->'singles')::int, 0) + coalesce((data->'doubles')::int, 0) + coalesce((data->'triples')::int, 0) + coalesce((data->'home_runs')::int, 0) as hits,
        (coalesce((data->'outs')::real, 0) / 3) as ip,
        game_player_stats.*
    from game_player_stats
)
select
    season,

    sum(outs / 3) as ip,

    sum(plate_appearances) as plate_appearances,
    sum(at_bats) as at_bats,

    ((sum(hits)::real) / nullif(sum(at_bats)::real, 0)) as ba,
    (sum(hits + walked + hit_by_pitch)::real / nullif(sum(plate_appearances)::real, 0)) as obp,
    (sum(singles + doubles * 2 + triples * 3 + home_runs * 4)::real / nullif(sum(at_bats)::real, 0)) as slg,
    (sum(hits + walked + hit_by_pitch)::real / nullif(sum(plate_appearances)::real, 0) + sum(singles + doubles * 2 + triples * 3 + home_runs * 4)::real / nullif(sum(at_bats)::real, 0)) as ops,

    ((9 * sum(earned_runs))::real / nullif(sum(ip), 0)) as era,
    (sum(walks + hits_allowed)::real / nullif(sum(ip), 0)) as whip,
    (sum(9 * home_runs_allowed)::real / nullif(sum(ip), 0)) as hr9,
    (sum(9 * walks)::real / nullif(sum(ip), 0)) as bb9,
    (sum(9 * strikeouts)::real / nullif(sum(ip), 0)) as k9,
    (sum(9 * hits_allowed)::real / nullif(sum(ip), 0)) as h9,
    (sum(13 * home_runs_allowed + 3 * (walks + hit_batters) - 2 * strikeouts)::real / nullif(sum(ip), 0)) as fip_base,

    sum(stolen_bases + caught_stealing) as sb_attempts,
    (sum(stolen_bases)::real / nullif(sum(stolen_bases + caught_stealing)::real, 0)) as sb_success,

    (sum(singles + doubles + triples)::real / nullif(sum(at_bats - struck_out + sac_flies)::real, 0)) as babip,
    (sum(putouts + assists)::real / nullif(sum(putouts + assists + errors)::real, 0)) as fpct
from helper h
         inner join teams t on (t.team_id = h.team_id)
group by (season)
with data;
create unique index game_player_stats_global_aggregate_idx on game_player_stats_global_aggregate(season);

drop function if exists league_percentiles;
create or replace function league_percentiles(real[]) returns table(
                                                                       season int,
                                                                       league_id text,

                                                                       ba real[],
                                                                       obp real[],
                                                                       slg real[],
                                                                       ops real[],
                                                                       sb_success real[],
                                                                       era real[],
                                                                       whip real[],
                                                                       fip_base real[],
                                                                       fip_const real[],
                                                                       h9 real[],
                                                                       k9 real[],
                                                                       bb9 real[],
                                                                       hr9 real[]
                                                                   )
AS $$
select
    season,
    league_id,

    percentile_cont($1) within group (order by h.ba) filter (where h.plate_appearances > 20) as ba,
    percentile_cont($1) within group (order by h.obp) filter (where h.plate_appearances > 20) as obp,
    percentile_cont($1) within group (order by h.slg) filter (where h.plate_appearances > 20) as slg,
    percentile_cont($1) within group (order by h.ops) filter (where h.plate_appearances > 20) as ops,
    percentile_cont($1) within group (order by h.sb_success) filter (where h.plate_appearances > 20) as sb_success,
    percentile_cont($1) within group (order by h.era desc) filter (where h.ip > 20) as era,
    percentile_cont($1) within group (order by h.whip desc) filter (where h.ip > 20) as whip,
    percentile_cont($1) within group (order by h.fip_base desc) filter (where h.ip > 20) as fip_base,
    percentile_cont($1) within group (order by h.era - h.fip_base desc) filter (where h.ip > 20) as fip_const,
    percentile_cont($1) within group (order by h.h9 desc) filter (where h.ip > 20) as h9,
    percentile_cont($1) within group (order by h.k9) filter (where h.ip > 20) as k9,
    percentile_cont($1) within group (order by h.bb9 desc) filter (where h.ip > 20) as bb9,
    percentile_cont($1) within group (order by h.hr9 desc) filter (where h.ip > 20) as hr9
from game_player_stats_advanced h
         inner join teams t on (t.team_id = h.team_id)
group by (season, t.league_id);
$$
    language sql;

drop function if exists global_percentiles;
create or replace function global_percentiles(real[]) returns table(
                                                                       season int,

                                                                       ba real[],
                                                                       obp real[],
                                                                       slg real[],
                                                                       ops real[],
                                                                       sb_success real[],
                                                                       era real[],
                                                                       whip real[],
                                                                       fip_base real[],
                                                                       fip_const real[],
                                                                       h9 real[],
                                                                       k9 real[],
                                                                       bb9 real[],
                                                                       hr9 real[]
                                                                   )
AS $$
select
    season,

    percentile_cont($1) within group (order by h.ba) filter (where h.plate_appearances > 20) as ba,
    percentile_cont($1) within group (order by h.obp) filter (where h.plate_appearances > 20) as obp,
    percentile_cont($1) within group (order by h.slg) filter (where h.plate_appearances > 20) as slg,
    percentile_cont($1) within group (order by h.ops) filter (where h.plate_appearances > 20) as ops,
    percentile_cont($1) within group (order by h.sb_success) filter (where h.plate_appearances > 20) as sb_success,
    percentile_cont($1) within group (order by h.era desc) filter (where h.ip > 20) as era,
    percentile_cont($1) within group (order by h.whip desc) filter (where h.ip > 20) as whip,
    percentile_cont($1) within group (order by h.fip_base desc) filter (where h.ip > 20) as fip_base,
    percentile_cont($1) within group (order by h.era - h.fip_base desc) filter (where h.ip > 20) as fip_const,
    percentile_cont($1) within group (order by h.h9 desc) filter (where h.ip > 20) as h9,
    percentile_cont($1) within group (order by h.k9) filter (where h.ip > 20) as k9,
    percentile_cont($1) within group (order by h.bb9 desc) filter (where h.ip > 20) as bb9,
    percentile_cont($1) within group (order by h.hr9 desc) filter (where h.ip > 20) as hr9
from game_player_stats_advanced h
         inner join teams t on (t.team_id = h.team_id)
group by (season);
$$
    language sql;

drop function if exists league_avgs;
create or replace function league_avgs() returns table(
                                                          season int,
                                                          league_id text,

                                                          ba real,
                                                          obp real,
                                                          slg real,
                                                          ops real,
                                                          sb_success real,
                                                          era real,
                                                          whip real,
                                                          fip_base real,
                                                          fip_const real,
                                                          h9 real,
                                                          k9 real,
                                                          bb9 real,
                                                          hr9 real
                                                      )
AS $$
select
    season,
    league_id,

    avg(ba) filter (where h.plate_appearances > 20) as ba,
    avg(obp) filter (where h.plate_appearances > 20) as obp,
    avg(slg) filter (where h.plate_appearances > 20) as slg,
    avg(ops) filter (where h.plate_appearances > 20) as ops,
    avg(sb_success) filter (where h.plate_appearances > 20) as sb_success,
    avg(era) filter (where h.ip > 20) as era,
    avg(whip) filter (where h.ip > 20) as whip,
    avg(fip_base) filter (where h.ip > 20) as fip_base,
    avg(era - fip_base) filter (where h.ip > 20) as fip_const,
    avg(h9) filter (where h.ip > 20) as h9,
    avg(k9) filter (where h.ip > 20) as k9,
    avg(bb9) filter (where h.ip > 20) as bb9,
    avg(hr9) filter (where h.ip > 20) as hr9
from game_player_stats_advanced h
         inner join teams t on (t.team_id = h.team_id)
group by (season, t.league_id);
$$
    language sql;

drop function if exists global_avgs;
create or replace function global_avgs() returns table(
                                                          season int,

                                                          ba real,
                                                          obp real,
                                                          slg real,
                                                          ops real,
                                                          sb_success real,
                                                          era real,
                                                          whip real,
                                                          fip_base real,
                                                          fip_const real,
                                                          h9 real,
                                                          k9 real,
                                                          bb9 real,
                                                          hr9 real
                                                      )
AS $$
select
    season,

    avg(ba) filter (where h.plate_appearances > 20) as ba,
    avg(obp) filter (where h.plate_appearances > 20) as obp,
    avg(slg) filter (where h.plate_appearances > 20) as slg,
    avg(ops) filter (where h.plate_appearances > 20) as ops,
    avg(sb_success) filter (where h.plate_appearances > 20) as sb_success,
    avg(era) filter (where h.ip > 20) as era,
    avg(whip) filter (where h.ip > 20) as whip,
    avg(fip_base) filter (where h.ip > 20) as fip_base,
    avg(era - fip_base) filter (where h.ip > 20) as fip_const,
    avg(h9) filter (where h.ip > 20) as h9,
    avg(k9) filter (where h.ip > 20) as k9,
    avg(bb9) filter (where h.ip > 20) as bb9,
    avg(hr9) filter (where h.ip > 20) as hr9
from game_player_stats_advanced h
         inner join teams t on (t.team_id = h.team_id)
group by (season);
$$
    language sql;

drop function if exists global_stddevs;
create or replace function global_stddevs() returns table(
                                                            season int,

                                                            ba real,
                                                            obp real,
                                                            slg real,
                                                            ops real,
                                                            sb_success real,
                                                            era real,
                                                            whip real,
                                                            fip_base real,
                                                            fip_const real,
                                                            h9 real,
                                                            k9 real,
                                                            bb9 real,
                                                            hr9 real
                                                        )
AS $$
select
    season,

    stddev_samp(ba) filter (where h.plate_appearances > 20) as ba,
    stddev_samp(obp) filter (where h.plate_appearances > 20) as obp,
    stddev_samp(slg) filter (where h.plate_appearances > 20) as slg,
    stddev_samp(ops) filter (where h.plate_appearances > 20) as ops,
    avg(sb_success) filter (where h.plate_appearances > 20) as sb_success,
    stddev_samp(era) filter (where h.ip > 20) as era,
    stddev_samp(whip) filter (where h.ip > 20) as whip,
    stddev_samp(fip_base) filter (where h.ip > 20) as fip_base,
    stddev_samp(era - fip_base) filter (where h.ip > 20) as fip_const,
    stddev_samp(h9) filter (where h.ip > 20) as h9,
    stddev_samp(k9) filter (where h.ip > 20) as k9,
    stddev_samp(bb9) filter (where h.ip > 20) as bb9,
    stddev_samp(hr9) filter (where h.ip > 20) as hr9
from game_player_stats_advanced h
         inner join teams t on (t.team_id = h.team_id)
group by (season);
$$
    language sql;

create materialized view if not exists pitches as
    select 
        *,
        split_part(data->>'pitch_info', ' ', 1)::real as pitch_speed,
        split_part(data->>'pitch_info', 'MPH ', 2) as pitch_type,
        nullif(data->>'zone', '')::smallint as pitch_zone
    from game_events
        where pitcher_id is distinct from null and data->>'event' = 'Pitch';
create unique index pitches_idx on pitches(game_id, index);