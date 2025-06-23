import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { getBasicTeams, getEntities, getGames, MmolbTeam } from "@/lib/data";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/team/$id/games")({
  component: RouteComponent,
  loader: async ({ params }) => {
    const teamId = params.id;
    const games = await getGames({ season: 1, team: teamId });
    const teams = await getEntities<MmolbTeam>(
      "team",
      games.items.flatMap((x) => [x.away_team_id, x.home_team_id])
    );

    return { games, teams };
  },
});

function RouteComponent() {
  const { games, teams } = Route.useLoaderData();
  const { id: teamId } = Route.useParams();

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="font-semibold text-left">Day</TableHead>
            <TableHead className="font-semibold text-left">Score</TableHead>
            <TableHead className="font-semibold text-left">Opponent</TableHead>
            <TableHead className="font-semibold text-left">SP</TableHead>
            <TableHead className="font-semibold text-left">RPs</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {games.items.map((game) => {
            const otherTeamId =
              game.away_team_id === teamId
                ? game.home_team_id
                : game.away_team_id;
            const otherTeam = teams[otherTeamId];
            const isAwayTeam = game.away_team_id === teamId;

            return (
              <TableRow>
                <TableCell>{game.day}</TableCell>
                <TableCell>
                  {game.last_update?.away_score}-{game.last_update?.home_score}
                </TableCell>
                <TableCell>
                  {isAwayTeam ? "@" : "vs."} {otherTeam.Emoji}{" "}
                  {otherTeam.Location} {otherTeam.Name}
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </div>
  );
}
