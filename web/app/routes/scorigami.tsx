import { getScorigami } from "@/lib/data";
import { createFileRoute } from "@tanstack/react-router";
import chroma from "chroma-js";
import colors from "tailwindcss/colors";

import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export const Route = createFileRoute("/scorigami")({
  component: RouteComponent,
  loader: async () => {
    const data = await getScorigami();
    return { data };
  },
});

function RouteComponent() {
  const { data } = Route.useLoaderData();

  let width = 0;
  let height = 0;
  let most = 0;

  for (let entry of data) {
    if (entry.max > width) width = entry.max;
    if (entry.min > height) height = entry.min;
    if (entry.count > most) most = entry.count;
  }

  let rows: number[][] = [];
  for (let i = 0; i <= height; i++) {
    rows.push(new Array(width - i).fill(0));
  }

  for (let entry of data) {
    rows[entry.min][entry.max - entry.min - 1] += entry.count;
  }

  const scale = chroma
    .scale("BuGn")
    .mode("lch")
    .gamma(0.5)
    .padding([0.2, 0])
    .domain([1, most]);

  return (
    <div className="container mx-auto p-4 flex flex-col items-center">
      <table>
        {rows.map((row, y) => {
          return (
            <tr key={y}>
              {y > 0 && <td colSpan={y}></td>}
              {row.map((count, x) => {
                return (
                  <Tooltip key={x}>
                    <TooltipTrigger asChild>
                      <td
                        className={"w-6 h-6 border"}
                        style={{
                          backgroundColor:
                            count > 0 ? scale(count).css() : undefined,
                        }}
                        key={x}
                      >
                        &nbsp;
                      </td>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p className="text-center">
                        {y}-{x}
                        <br />
                        <strong>{count}</strong>
                      </p>
                    </TooltipContent>
                  </Tooltip>
                );
              })}
            </tr>
          );
        })}
      </table>
    </div>
  );
}
