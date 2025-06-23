import { DataTablePagination } from "@/components/DataTablePagination";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  API_BASE,
  BasicLeague,
  BasicTeam,
  ChronEntity,
  ChronPaginatedResponse,
  getBasicLeagues,
  getBasicTeams,
  MmolbLeague,
  MmolbPlayer,
  MmolbTeam,
} from "@/lib/data";
import { useInfiniteQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  HeaderContext,
  Row,
  RowExpanding,
  SortingState,
  useReactTable,
} from "@tanstack/react-table";
import clsx from "clsx";
import { ChevronDown, ChevronUp } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

export const Route = createFileRoute("/players")({
  component: RouteComponent,
  loader: async () => {
    const [teams, leagues] = await Promise.all([
      getBasicTeams(),
      getBasicLeagues(),
    ]);
    return { teams, leagues };
  },
});

type RowData = MmolbPlayer & { id: string; teamName: string };

function SortableHeader(name: string) {
  return (props: HeaderContext<RowData, unknown>) => {
    return (
      <div
        className="flex items-center cursor-pointer"
        onClick={() => props.column.toggleSorting()}
      >
        {name}

        {props.column.getIsSorted() === "asc" && (
          <ChevronDown className="h-4 w-4 ml-0.5" />
        )}
        {props.column.getIsSorted() === "desc" && (
          <ChevronUp className="h-4 w-4 ml-0.5" />
        )}
      </div>
    );
  };
}

const columns: ColumnDef<RowData>[] = [
  {
    header: SortableHeader("ID"),
    accessorKey: "id",
    sortingFn: "text",
    cell: (props) => (
      <span className="font-mono">{props.getValue()?.toString()}</span>
    ),
  },
  { header: SortableHeader("#"), accessorKey: "Number" },
  { header: "Pos.", accessorKey: "Position" },
  {
    id: "name",
    header: SortableHeader("Name"),
    accessorFn: (row) => {
      return `${row.FirstName} ${row.LastName}`;
    },
    cell: (props) => {
      return (
        <a href={`https://mmolb.com/player/${props.row.original.id}`}>
          {props.getValue()?.toString()}
        </a>
      );
    },
  },
  {
    header: SortableHeader("Team"),
    accessorKey: "teamName",
    cell: (props) => {
      const teamId = props.row.original.TeamID;
      if (!teamId) return <span>Null Team</span>;

      const team = (props.table.options.meta as any).teams[teamId] as
        | BasicTeam
        | undefined;
      if (!team) return <span>(unknown {teamId})</span>;

      // const league = (props.table.options.meta as any).leagues[
      //   team.league_id
      // ] as BasicLeague;

      return (
        <a href={`https://mmolb.com/team/${teamId}`}>
          {team.emoji} {team.location} {team.name}
        </a>
      );
    },
  },
  { header: SortableHeader("Likes"), accessorKey: "Likes" },
  { header: SortableHeader("Dislikes"), accessorKey: "Dislikes" },
  { header: SortableHeader("Bats"), accessorKey: "Bats" },
  { header: SortableHeader("Throws"), accessorKey: "Throws" },
  {
    id: "mods",
    header: SortableHeader("Mods"),
    accessorFn: (row) => {
      const mods = [...row.Modifications];
      if (row.LesserBoon) mods.push(row.LesserBoon);
      return mods.map((x) => x.Emoji).join("");
    },
  },
  {
    id: "accessories",
    header: SortableHeader("Acc."),
    accessorFn: (row) => {
      return row.Equipment?.Accessory?.Emoji ?? "";
    },
  },
  {
    id: "body",
    header: SortableHeader("Body"),
    accessorFn: (row) => {
      return row.Equipment?.Body?.Emoji ?? "";
    },
  },
  {
    id: "feet",
    header: SortableHeader("Feet"),
    accessorFn: (row) => {
      return row.Equipment?.Feet?.Emoji ?? "";
    },
  },
  {
    id: "hands",
    header: SortableHeader("Hands"),
    accessorFn: (row) => {
      return row.Equipment?.Hands?.Emoji ?? "";
    },
  },
  {
    id: "head",
    header: SortableHeader("Head"),
    accessorFn: (row) => {
      return row.Equipment?.Head?.Emoji ?? "";
    },
  },
];

const fetchPlayers = async ({ pageParam }) => {
  const resp = await fetch(
    API_BASE +
      `/chron/v0/entities?kind=player_lite${pageParam ? "&page=" + pageParam : ""}&order=desc`
  );
  const data = (await resp.json()) as ChronPaginatedResponse<
    ChronEntity<MmolbPlayer>
  >;
  return data;
};

function RouteComponent() {
  const { teams, leagues } = Route.useLoaderData();

  const maxPages = 999;
  const {
    data,
    error,
    status,
    fetchNextPage,
    hasNextPage,
    isFetching,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["players"],
    queryFn: fetchPlayers,
    initialPageParam: null,
    getNextPageParam: (lastPage, _) => lastPage.next_page ?? undefined,
    maxPages,
    staleTime: 30 * 60 * 1000,
  });

  const shouldTriggerNextPage =
    !isFetchingNextPage && hasNextPage && (data?.pages.length ?? 0) < maxPages;
  useEffect(() => {
    if (shouldTriggerNextPage) {
      fetchNextPage();
    }
  }, [shouldTriggerNextPage]);

  const flattenedPlayers: RowData[] = useMemo(() => {
    return (data?.pages ?? [])
      .flatMap((x) => x.items)
      .filter((x) => !!x.data.TeamID)
      .map((x) => {
        const team = teams[x.data.TeamID ?? ""];
        const teamName = team ? `${team.location} ${team.name}` : "";
        return { ...x.data, id: x.entity_id, teamName: teamName };
      });
  }, [data]);

  const [sorting, setSorting] = useState<SortingState>([]);
  const [globalFilter, setGlobalFilter] = useState<any>([]);

  const table = useReactTable({
    data: flattenedPlayers,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onGlobalFilterChange: setGlobalFilter,

    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),

    initialState: {
      pagination: {
        pageSize: 100,
      },
    },
    meta: {
      leagues,
      teams,
    },
    state: {
      sorting,
      globalFilter,
    },
  });

  if (isFetchingNextPage || shouldTriggerNextPage || status === "pending") {
    const playerCount =
      data?.pages.map((x) => x.items.length).reduce((a, b) => a + b, 0) ?? 0;
    return (
      <div className="text-center m-4">
        Loading players... ({playerCount} so far)
      </div>
    );
  }

  return (
    <div className="container mx-auto mt-4">
      <div className="flex flex-row">
        <h1 className="py-4 flex-1 font-semibold">Player List (VERY wip)</h1>
        <div className="flex items-center py-4">
          <Input
            placeholder="Search..."
            onChange={(e) => table.setGlobalFilter(String(e.target.value))}
            className="max-w-sm"
          />
        </div>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead
                      key={header.id}
                      colSpan={header.colSpan}
                      className={clsx(
                        "font-semibold",
                        header.subHeaders.length === 0
                          ? "text-left"
                          : "text-center",
                        header.subHeaders.length > 1 ? "border-l" : ""
                      )}
                    >
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && "selected"}
                  className="border-0"
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell
                      key={cell.id}
                      className={
                        cell.column.id === "emoji" ? "p-0" : "border-b"
                      }
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <div className="m-2">
        <DataTablePagination table={table} />
      </div>
    </div>
  );
}
