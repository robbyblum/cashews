import { getEntities, getEntity, MmolbPlayer, MmolbTeam } from "@/lib/data";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/team/$id/roster")({
  component: RouteComponent,
  loader: async ({ params }) => {
    const teamId = params.id;

    const team = await getEntity<MmolbTeam>("team", teamId);

    const playerIds = team.data.Players.map((x) => x.PlayerID).filter(
      (x) => x != "#"
    );
    const players = await getEntities<MmolbPlayer>("player", playerIds);
    return { team: team.data, players };
  },
});

function RouteComponent() {
  // const { team, players } =
  return <div>Hello "/team/$id/roster"!</div>;
}
