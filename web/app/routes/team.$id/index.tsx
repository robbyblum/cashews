import {
  ChartConfig,
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart";
import { ChronGame, getBasicTeams, getEntities, getGames } from "@/lib/data";
import { createFileRoute } from "@tanstack/react-router";
import {
  Bar,
  BarChart,
  Cell,
  ReferenceLine,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

interface GameWinLoss {
  day: number;
  cumulative_wl: number;
  is_win: boolean;
  cumulative_run_diff: number;
  run_diff: number;
  game_id: string;
  other_team: string;
}

function extractWinLoss(teamId: string, games: ChronGame[]) {
  const wls: GameWinLoss[] = [];

  // look, i didn't shorten it to cum
  let cumulative_wl = 0;
  let cumulative_run_diff = 0;
  for (let game of games) {
    const lu = game.last_update;
    if (!lu) continue;
    if (game.state != "Complete") continue;

    const run_diff =
      game.away_team_id === teamId
        ? lu.away_score - lu.home_score
        : lu.home_score - lu.away_score;
    cumulative_run_diff += run_diff;

    const is_win =
      game.away_team_id === teamId
        ? lu.away_score > lu.home_score
        : lu.home_score > lu.away_score;
    cumulative_wl += is_win ? 1 : -1;
    wls.push({
      game_id: game.game_id,
      other_team:
        game.away_team_id === teamId ? game.home_team_id : game.away_team_id,
      day: game.day,
      is_win: is_win,
      run_diff: run_diff,
      cumulative_run_diff: cumulative_run_diff,
      cumulative_wl: cumulative_wl,
    });
  }
  return wls;
}

export const Route = createFileRoute("/team/$id/")({
  component: RouteComponent,
  loader: async ({ params }) => {
    const teamId = params.id;

    // todo: paginate, as well
    const season = 0;
    const games = await getGames({
      season, // todo: season selector
      team: teamId,
    });

    const wls = extractWinLoss(teamId, games.items);
    const teams = getEntities(
      "team",
      wls.map((x) => x.other_team)
    );

    return { games: games.items, teams, season };
  },
});

const config: ChartConfig = {
  cumulative_wl: {
    label: "Wins/Losses",
  },
};

function WinLossGraph(props: {
  season: number;
  wls: GameWinLoss[];
  type: "wl" | "rd";
}) {
  const totalGames = props.season == 0 ? 120 : 240;
  const gap = props.season == 0 ? 1 : 2;

  const dataKey = props.type == "wl" ? "cumulative_wl" : "cumulative_run_diff";
  return (
    <ChartContainer config={config} className="w-[60rem] h-[30rem] mx-auto">
      <BarChart data={props.wls} barCategoryGap={0}>
        <Bar dataKey={dataKey}>
          {props.wls.map((wl, idx) => {
            const thisValue =
              props.type == "wl" ? wl.cumulative_wl : wl.cumulative_run_diff;

            const nextVal = 1;
            idx == props.wls.length - 1
              ? 0
              : props.type == "wl"
                ? wl.cumulative_wl
                : wl.cumulative_run_diff;
            const prevVal =
              idx == 0
                ? 0
                : props.type == "wl"
                  ? wl.cumulative_wl
                  : wl.cumulative_run_diff;

            const color =
              thisValue >= 0
                ? "var(--color-green-500)"
                : "var(--color-red-500)";
            const radius = 2;
            const roundLeft = Math.abs(prevVal) < Math.abs(thisValue);
            const roundRight = Math.abs(nextVal) < Math.abs(thisValue);
            return (
              <Cell
                fill={color}
                stroke={color}
                radius={
                  [roundLeft ? radius : 0, roundRight ? radius : 0, 0, 0] as any
                }
              />
            );
          })}
        </Bar>
        <XAxis
          label={{ value: "Day", position: "bottom", offset: -10 }}
          dataKey="day"
          min={0}
          max={totalGames}
        />
        <YAxis
          label={{
            value: props.type == "wl" ? "W-L" : "Run Differential",
            angle: -90,
          }}
          dataKey={dataKey}
        />
        <ReferenceLine
          y={0}
          stroke="light-dark(var(--color-gray-800), var(--color-gray-200))"
        />
      </BarChart>
    </ChartContainer>
  );
}

function RouteComponent() {
  const { season, games } = Route.useLoaderData();
  const { id: teamId } = Route.useParams();

  const wls = extractWinLoss(teamId, games);

  return (
    <div className="container mx-auto">
      <WinLossGraph type="wl" season={season} wls={wls} />
      <WinLossGraph type="rd" season={season} wls={wls} />
    </div>
  );
}
