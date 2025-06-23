// import { useQuery, UseQueryResult } from "@tanstack/react-query";

import {
  useQueries,
  useQuery,
  UseQueryResult,
  useSuspenseQueries,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { cache } from "react";
import { PercentileStat } from "./percentile";

// export function useTeams(): UseQueryResult<any> {
//     return useQuery({
//         queryKey: ["teams"],
//         queryFn: async () => {
//             return fetch("/api/allteams").then(r => r.json())
//         },
//         initialData: []
//     })
// }

export interface Team {}

// const API_BASE = "https://freecashe.ws";

// export const getTeams = cache(
//   () =>
//     fetch(API_BASE + "/api/allteams", {
//       cache: "force-cache",
//     }).then((x) => x.json()) as Promise<Record<string, Team>>
// );

// async function getEntities(id: string): Promise<Team> {
//   const teams = await getTeams();
//   return teams[id];
// }

export const API_BASE = import.meta.env.SSR
  ? (process.env.API_BASE ?? "http://localhost:3000/api")
  : "/api";

export async function getEntity<T>(
  kind: string,
  id: string
): Promise<ChronEntity<T>> {
  const resp = await fetch(
    API_BASE + `/chron/v0/entities?kind=${kind}&id=${id}`
  );
  const data = (await resp.json()) as ChronPaginatedResponse<ChronEntity<T>>;
  return data.items[0];
}
export async function getEntities<T>(
  kind: string,
  id: string[]
): Promise<Record<string, T>> {
  const dedupIds = [...new Set(id)];
  const resp = await fetch(
    API_BASE + `/chron/v0/entities?kind=${kind}&id=${dedupIds.join(",")}`
  );
  const data = (await resp.json()) as ChronPaginatedResponse<ChronEntity<T>>;

  return Object.fromEntries(data.items.map((x) => [x.entity_id, x.data]));
}

export async function getBasicTeams(): Promise<Record<string, BasicTeam>> {
  const resp = await fetch(API_BASE + `/teams`, {});
  const data = (await resp.json()) as ChronPaginatedResponse<BasicTeam>;

  return Object.fromEntries(data.items.map((x) => [x.team_id, x]));
}

export async function getBasicLeagues(): Promise<Record<string, BasicLeague>> {
  const resp = await fetch(API_BASE + `/leagues`, {});
  const data = (await resp.json()) as ChronPaginatedResponse<BasicLeague>;

  return Object.fromEntries(data.items.map((x) => [x.league_id, x]));
}

export async function getGames(q: {
  season?: number;
  day?: number;
  team?: string;
}): Promise<ChronPaginatedResponse<ChronGame>> {
  const qs = new URLSearchParams();
  if (q.season !== undefined) qs.set("season", q.season.toString());
  if (q.day !== undefined) qs.set("day", q.day.toString());
  if (q.team !== undefined) qs.set("team", q.team);

  const resp = await fetch(API_BASE + `/games?${qs.toString()}`);
  const data = (await resp.json()) as ChronPaginatedResponse<ChronGame>;
  return data;
}

export async function getTeamStats(
  team: string,
  season: number
): Promise<PlayerStatsEntry[]> {
  const resp = await fetch(
    API_BASE +
      `/player-stats?team=${team}&start=${season},0&end=${season + 1},0`
  );
  const data = (await resp.json()) as PlayerStatsEntry[];
  return data;
}
export interface PercentileStats {
  ba: PercentileStat;
  obp: PercentileStat;
  slg: PercentileStat;
  ops: PercentileStat;
  era: PercentileStat;
  whip: PercentileStat;
  h9: PercentileStat;
  k9: PercentileStat;
  bb9: PercentileStat;
  hr9: PercentileStat;
  fip_const: PercentileStat;
}

export interface PercentileResponse {
  leagues: Record<string, PercentileStats>;
}

export interface AveragesResponse {
  season: number;
  league_id: string;
  ip: number;
  plate_appearances: number;
  at_bats: number;
  ba: number;
  obp: number;
  slg: number;
  ops: number;
  era: number;
  whip: number;
  hr9: number;
  bb9: number;
  k9: number;
  h9: number;
  fip_base: number;
  sb_attempts: number;
  sb_success: number;
  babip: number;
  fpct: number;
}

export async function getLeagueAggregates(
  season: number
): Promise<PercentileResponse> {
  const resp = await fetch(
    API_BASE + `/league-aggregate-stats?season=${season}`
  );
  // const data = (await resp.json()) as PercentileResponse;
  return (await resp.json()) as PercentileResponse;
}

export async function getLeagueAverages(
    season: number
): Promise<AveragesResponse[]> {
  const resp = await fetch(
      API_BASE + `/league-averages?season=${season}`
  );
  // const data = (await resp.json()) as AveragesResponse;
  return (await resp.json()) as AveragesResponse[];
}

export function useAllTeams() {
  const { data } = useSuspenseQuery({
    queryKey: ["teams"],
    queryFn: getBasicTeams,
    staleTime: 60 * 1000,
  });
  return data;
}

export function useAllLeagues() {
  const { data } = useSuspenseQuery({
    queryKey: ["leagues"],
    queryFn: getBasicLeagues,
    staleTime: 60 * 1000,
  });
  return data;
}

export async function getScorigami() {
  const resp = await fetch(API_BASE + `/scorigami`);
  const data = (await resp.json()) as ScorigamiEntry[];
  return data;
}

export async function getLocations() {
  const resp = await fetch(API_BASE + `/locations`);
  const data = (await resp.json()) as MapLocation[];
  return data;
}

export interface ScorigamiEntry {
  min: number;
  max: number;
  count: number;
  first: string;
}

export interface MapLocation {
  team: BasicTeam;
  location: { lat: number; long: number } | null;
}

// export function useTeams(ids: string[]) {
//   let ids2 = [...new Set(ids)];
//   const res = useSuspenseQueries({
//     queries: ids2.map((id) => ({
//       queryKey: ["team", id],
//       queryFn: async () => {
//         const res = await getBasicTeams();
//         return res[id];
//       },
//     })),
//   });
//   return Object.fromEntries(res.map((r) => [r.data.team_id, r.data]));
// }

// export async function useChronEntity<T>(
//   kind: string,
//   id: string
// ): UseQueryResult<ChronEntity<T>> {
//   return useQuery({
//     queryKey: [""],
//   });
// }

export interface ChronPaginatedResponse<T> {
  next_page: string | null;
  items: T[];
}

export interface ChronEntity<T> {
  entity_id: string;
  data: T;
}

export interface MmolbTeam {
  Location: string;
  Name: string;
  Emoji: string;
  Color: string;
  League: string;
  Players: MmolbRosterSlot[];
}

export interface MmolbRosterSlot {
  Slot: string;
  PlayerID: string;
  Position: string;
  PositionType: string;
  Number: number;
  FirstName: string;
  LastName: string;
}

export interface MmolbLeague {
  Name: string;
  Emoji: string;
}

export interface ChronGame {
  game_id: string;
  season: number;
  day: number;
  home_team_id: string;
  away_team_id: string;
  state: string;
  event_count: number;
  last_update: MmolbGameEvent | null;
}

export interface MmolbGameEvent {
  inning: number;
  away_score: number;
  home_score: number;
  inning_side: number;
  event: string;
  message: string;
}

export interface PlayerStatsEntry {
  player_id: string;
  team_id: string;
  stats: Record<string, number>;
}

export interface MmolbPlayer {
  FirstName: string;
  LastName: string;
  Number: number;
  Position: string;
  TeamID: string | null;
  LesserBoon: MmolbBoon | null;
  Modifications: MmolbBoon[];
  Equipment:
    | {
        Accessory: MmolbEquipment | null;
        Body: MmolbEquipment | null;
        Feet: MmolbEquipment | null;
        Hands: MmolbEquipment | null;
        Head: MmolbEquipment | null;
      }
    | undefined;
  Throws: string;
  Bats: string;
}

export interface MmolbEquipment {
  Effects: string[];
  Emoji: string;
  Name: string;
}

export interface MmolbBoon {
  Description: string;
  Emoji: string;
  Name: string;
}

export interface BasicTeam {
  team_id: string;
  league_id: string;
  name: string;
  location: string;
  full_location: string;
  emoji: string;
  color: string;
  abbreviation: string;
}

export interface BasicLeague {
  league_id: string;
  league_type: string;
  name: string;
  emoji: string;
  color: string;
}

export interface StatPercentile {
  league_id: string;
  season: number;
  percentile: number;
  fip_base: number;
  fip_const: number;
  era: number;
  ops: number;
}

export interface MmolbGame {
  AwaySP: string;
  HomeSP: string;
  AwayTeamID: string;
  HomeTeamID: string;
  EventLog: MmolbGameEvent[];
}
