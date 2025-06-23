import ColorPreview from "@/components/ColorPreview";
import StatsTable, { StatDisplay } from "@/components/StatsTable";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { defaultScale, hotCold } from "@/lib/colors";
import {
  getEntities,
  getEntity,
  getGames,
  getLeagueAggregates, getLeagueAverages,
  getTeamStats,
  MmolbGame,
  MmolbGameEvent,
  MmolbPlayer,
  MmolbTeam,
} from "@/lib/data";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useTheme } from "next-themes";
import { useState } from "react";
import { z } from "zod";

const defaultSeason = 2;
const stateSchema = z.object({
  season: z.number().catch(1).optional(),
});

type StateParams = z.infer<typeof stateSchema>;

async function getLineupOrder(teamId: string): Promise<(string | null)[]> {
  // TODO: this is awful code :p
  // need to not waterfall as hard

  const [games, team] = await Promise.all([
    getGames({ season: 2, team: teamId }),
    getEntity<MmolbTeam>("team", teamId),
  ]);
  // lol
  // also, todo: handle pagination?
  games.items.sort((x) => x.season * 1000 + x.day);
  const lastGame = games.items[games.items.length - 1];

  const gameData = await getEntity<MmolbGame>("game", lastGame.game_id);

  let lineupEvent: MmolbGameEvent | undefined = undefined;
  if (gameData.data.AwayTeamID === teamId) {
    lineupEvent = gameData.data.EventLog.find((x) => x.event === "AwayLineup");
  } else {
    lineupEvent = gameData.data.EventLog.find((x) => x.event === "HomeLineup");
  }

  if (lineupEvent) {
    const lineupLines = lineupEvent.message.split("<br>");
    const batters = team.data.Players.filter(
      (x) => x.PositionType === "Batter"
    );

    const lineupIds: (string | null)[] = [];
    for (let line of lineupLines) {
      const batter = batters.find((b) =>
        line.includes(`${b.FirstName} ${b.LastName}`)
      );
      lineupIds.push(batter?.PlayerID ?? null);
    }
    return lineupIds;
  } else {
    return [];
  }
}

export const Route = createFileRoute("/team/$id/stats")({
  component: RouteComponent,
  validateSearch: (search) => stateSchema.parse(search),
  loaderDeps: ({ search: { season } }) => ({ season }),
  loader: async ({ params, deps }) => {
    const [stats, pcts, aggs, lineupOrder] = await Promise.all([
      getTeamStats(params.id, deps.season ?? defaultSeason),
      getLeagueAggregates(deps.season ?? defaultSeason),
      getLeagueAverages(deps.season ?? defaultSeason),
      getLineupOrder(params.id),
    ]);

    const playerIds = [...new Set(stats.map((x) => x.player_id))];
    const teamIds = [...new Set(stats.map((x) => x.team_id))];

    const [players, teams] = await Promise.all([
      getEntities<MmolbPlayer>("player_lite", playerIds),
      getEntities<MmolbTeam>("team", teamIds),
    ]);

    const thisTeam = teams[params.id]!;

    return { stats, players, teams, pcts, aggs, lineupOrder };
  },
});

function RouteComponent() {
  const { stats, players, teams, pcts, aggs, lineupOrder } = Route.useLoaderData();
  const { season } = Route.useSearch();
  const navigate = useNavigate({ from: Route.fullPath });

  const seasons = [2, 1, 0];

  const [display, setDisplay] = useState<StatDisplay>("stat");
  const scale = defaultScale;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-row place-content-end">
        <Select
          value={(season ?? defaultSeason).toString()}
          onValueChange={(val) => {
            navigate({
              search: (prev) => ({ ...prev, season: parseInt(val) ?? 0 }),
            });
          }}
        >
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Season..."></SelectValue>
          </SelectTrigger>
          <SelectContent>
            {seasons.map((s) => (
              <SelectItem value={s.toString()}>Season {s}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div>
        <h2 className="mb-2 font-medium text-lg">Batting</h2>
        <StatsTable
          data={stats}
          players={players}
          teams={teams}
          pcts={pcts}
          aggs={aggs}
          display={display}
          lineupOrder={lineupOrder}
          type="batting"
        />
      </div>

      <div>
        <h2 className="mb-2 font-medium text-lg">Pitching</h2>
        <StatsTable
          data={stats}
          players={players}
          teams={teams}
          pcts={pcts}
          aggs={aggs}
          display={display}
          lineupOrder={lineupOrder}
          type="pitching"
        />
      </div>

      <div>
        <h2 className="mb-2 font-medium text-lg">Color Scale (percentiles)</h2>
        <ColorPreview scale={scale} />
      </div>
    </div>
  );
}
