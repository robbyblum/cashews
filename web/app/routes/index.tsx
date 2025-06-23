import { createFileRoute, useRouter } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  component: Home,
  loader: async () => {},
});

function Home() {
  const router = useRouter();
  const state = Route.useLoaderData();

  return <div>please imagine front page</div>;
}
