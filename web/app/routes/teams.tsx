import { createFileRoute, Link } from "@tanstack/react-router";

export const Route = createFileRoute("/teams")({
  component: RouteComponent,
});

import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { BasicLeague, BasicTeam, useAllLeagues, useAllTeams } from "@/lib/data";
import {
  CellContext,
  ColumnDef,
  ColumnFiltersState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  HeaderContext,
  SortingState,
  useReactTable,
} from "@tanstack/react-table";
import { ChevronDown, ChevronUp } from "lucide-react";
import { useMemo, useState } from "react";
import { DataTablePagination } from "@/components/DataTablePagination";

function SortableHeader(name: string) {
  return (props: HeaderContext<BasicTeam, unknown>) => {
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

function EmojiCell(props: CellContext<BasicTeam, unknown>) {
  return (
    <div
      suppressHydrationWarning
      className="text-xl p-1 text-center"
      style={{
        backgroundColor: "#" + props.row.original.color,
        borderBottom: "1px solid #" + props.row.original.color,
        // borderTop: "1px solid black",
        // transform: "translate(0, 0px)",
      }}
    >
      {props.getValue()?.toString()}
    </div>
  );
}

function IdCell(props: CellContext<BasicTeam, unknown>) {
  return <div className="font-mono">{props.getValue()?.toString()}</div>;
}

function LeagueCell(props: CellContext<BasicTeam, unknown>) {
  const team = props.row.original;
  const leagues = (props.table.options.meta as any).leagues as Record<
    string,
    BasicLeague
  >;
  const league = leagues[team.league_id];

  if (league) {
    return (
      <div>
        {league.emoji} {league.name} League
      </div>
    );
  } else {
    return <div>???</div>;
  }
}

function TeamNameCell(props: CellContext<BasicTeam, unknown>) {
  const team = props.row.original;
  return (
    <Link
      to={`/team/$id/stats`}
      params={{ id: team.team_id }}
      className="font-medium hover:underline"
      preload={false}
    >
      {props.getValue()?.toString()}
    </Link>
  );
}

const columns: ColumnDef<BasicTeam>[] = [
  {
    id: "emoji",
    header: SortableHeader(""),
    accessorKey: "emoji",
    cell: EmojiCell,
  },
  {
    id: "name",
    header: SortableHeader("Name"),
    accessorFn: (team) => `${team.location} ${team.name}`,
    cell: TeamNameCell,
  },
  {
    id: "id",
    accessorKey: "team_id",
    sortingFn: "text", // don't do "numeric sorting"
    header: SortableHeader("ID"),
    cell: IdCell,
  },
  {
    accessorKey: "league_id",
    cell: LeagueCell,
    header: SortableHeader("League"),
  },
];

function RouteComponent() {
  const leagues = useAllLeagues();
  const teams = useAllTeams();
  const teamsList = useMemo(() => {
    const values = Object.values(teams);
    values.sort((a, b) => a.team_id.localeCompare(b.team_id));
    return values;
  }, [teams]);

  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);
  const [globalFilter, setGlobalFilter] = useState<any>([]);

  const table = useReactTable({
    data: teamsList,
    columns,
    getCoreRowModel: getCoreRowModel(),
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    onColumnFiltersChange: setColumnFilters,
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onGlobalFilterChange: setGlobalFilter,
    initialState: {
      pagination: {
        pageSize: 100,
      },
    },
    state: {
      sorting,
      columnFilters,
      globalFilter,
    },
    meta: {
      leagues,
    },
  });

  return (
    <div className="container mx-auto">
      <div className="flex flex-row">
        <h1 className="py-4 flex-1">Team List</h1>
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
                    <TableHead key={header.id} className="font-semibold">
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
