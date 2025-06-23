import {AveragesResponse, PercentileStats, PlayerStatsEntry, StatPercentile} from "./data";
import { findValueAtPercentile } from "./percentile";

export interface AdvancedStats {
  at_bats: number;
  plate_appearances: number;
  hits: number;
  doubles: number;
  triples: number;
  home_runs: number;
  walked: number;
  struck_out: number;
  ba: number;
  obp: number;
  slg: number;
  ops: number;
  ops_plus: number;

  stolen_bases: number;
  caught_stealing: number;
  sb_success: number;

  ip: number;
  appearances: number;
  starts: number;
  wins: number;
  losses: number;
  hits_allowed: number;
  home_runs_allowed: number;
  strikeouts: number;
  walks: number;
  era: number;
  era_minus: number;
  fip: number;
  fip_minus: number;
  whip: number;
  h9: number;
  bb9: number;
  k9: number;
  hr9: number;
}

export function calculateAdvancedStats(
    data: PlayerStatsEntry,
    leagueAvgStats: AveragesResponse | undefined,
): AdvancedStats {
  if(leagueAvgStats === undefined) {
    throw new Error("Invalid league ID for finding league averages");
  }
  const meanObp = leagueAvgStats.obp;
  const meanSlg = leagueAvgStats.slg;
  const meanEra = leagueAvgStats.era;
  const meanFipBase = leagueAvgStats.fip_base;

  const singles = data.stats.singles ?? 0;
  const doubles = data.stats.doubles ?? 0;
  const triples = data.stats.triples ?? 0;
  const home_runs = data.stats.home_runs ?? 0;
  const earned_runs = data.stats.earned_runs ?? 0;
  const outs = data.stats.outs ?? 0;
  const walks = data.stats.walks ?? 0;
  const hits_allowed = data.stats.hits_allowed ?? 0;
  const strikeouts = data.stats.strikeouts ?? 0;
  const home_runs_allowed = data.stats.home_runs_allowed ?? 0;
  const walked = data.stats.walked ?? 0;
  const struck_out = data.stats.struck_out ?? 0;
  const hit_by_pitch = data.stats.hit_by_pitch ?? 0;
  const hit_batters = data.stats.hit_batters ?? 0;

  const stolen_bases = data.stats.stolen_bases ?? 0;
  const caught_stealing = data.stats.caught_stealing ?? 0;
  const sb_success = stolen_bases / (stolen_bases + caught_stealing);

  const ip = outs / 3;
  const appearances = data.stats.appearances ?? 0;
  const starts = data.stats.starts ?? 0;
  const wins = data.stats.wins ?? 0;
  const losses = data.stats.losses ?? 0;

  const abs = data.stats.at_bats ?? 0;
  const pas = data.stats.plate_appearances ?? 0;

  const hits = singles + doubles + triples + home_runs;

  const ba = hits / abs;
  const obp = (hits + walked + hit_by_pitch) / pas;
  const slg = (singles + doubles * 2 + triples * 3 + home_runs * 4) / abs;
  const ops = obp + slg;
  const ops_plus = 100 * ((obp / meanObp) + (slg / meanSlg) - 1);

  const era = (9 * earned_runs) / ip;
  const era_minus = (100 * era) / meanEra;
  const fip_base =
    (13 * home_runs_allowed + 3 * (walks + hit_batters) - 2 * strikeouts) / ip;
  const fip = fip_base + (meanEra - meanFipBase);
  const fip_minus = (100 * fip) / meanEra;
  const whip = (walks + hits_allowed) / ip;
  const hr9 = (9 * home_runs_allowed) / ip;
  const bb9 = (9 * walks) / ip;
  const k9 = (9 * strikeouts) / ip;
  const h9 = (9 * hits_allowed) / ip;

  return {
    at_bats: abs,
    plate_appearances: pas,
    hits,
    doubles,
    triples,
    home_runs,
    walked,
    struck_out,
    ba,
    obp,
    slg,
    ops,
    ops_plus,

    stolen_bases,
    caught_stealing,
    sb_success,

    ip,
    appearances,
    starts,
    wins,
    losses,
    hits_allowed,
    home_runs_allowed,
    strikeouts,
    walks,
    era,
    era_minus,
    fip,
    fip_minus,
    whip,
    h9,
    bb9,
    k9,
    hr9,
  };
}
