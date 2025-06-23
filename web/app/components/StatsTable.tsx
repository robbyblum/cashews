"use client";

import {
  MmolbPlayer,
  MmolbRosterSlot,
  MmolbTeam,
  PercentileResponse,
  AveragesResponse,
  PlayerStatsEntry,
  StatPercentile,
} from "@/lib/data";
import { AdvancedStats, calculateAdvancedStats } from "@/lib/stats";
import {
  CellContext,
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  HeaderContext,
  SortingState,
  useReactTable,
} from "@tanstack/react-table";
import {
  Table,
  TableBody,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from "./ui/table";
import { useMemo, useState } from "react";
import {
  ArrowDown,
  ArrowUp,
  ChevronDown,
  ChevronUp,
  MoveHorizontal,
} from "lucide-react";
import { findPercentile } from "@/lib/percentile";
import clsx from "clsx";
import { defaultScale, scales } from "@/lib/colors";

export type StatDisplay = "percentile" | "stat" | "vibes";

interface StatsTableProps {
  data: PlayerStatsEntry[];
  players: Record<string, MmolbPlayer>;
  teams: Record<string, MmolbTeam>;
  type: "batting" | "pitching";
  pcts: PercentileResponse;
  aggs: AveragesResponse[];
  display: StatDisplay;
  lineupOrder: (string | null)[];
}

type RowData = {
  id: string;
  name: string;
  position: string | null;
  team: string;
  league: string;
  rosterIndex: number;
  lineupIndex: number | undefined;
} & AdvancedStats;

function PercentileVibes(props: { percentile: number }) {
  // this is a joke
  const cls = "flex flex-row place-content-end";

  const UpArrow = () => <ArrowUp size={16} className="mr-[-4px]" />;
  const DownArrow = () => <ArrowDown size={16} className="mr-[-4px]" />;
  const MidArrow = () => <MoveHorizontal size={16} />;

  if (props.percentile > 0.9)
    return (
      <span className={cls + " text-green-700"}>
        <UpArrow />
        <UpArrow />
        <UpArrow />
      </span>
    );
  if (props.percentile > 0.75)
    return (
      <span className={cls + " text-green-600"}>
        <UpArrow />
        <UpArrow />
      </span>
    );
  if (props.percentile > 0.6)
    return (
      <span className={cls + " text-green-500"}>
        <UpArrow />
      </span>
    );

  if (props.percentile > 0.4)
    return (
      <span className={cls + " text-gray-500"}>
        <MidArrow />
      </span>
    );

  if (props.percentile > 0.25)
    return (
      <span className={cls + " text-red-400"}>
        <DownArrow />
      </span>
    );

  if (props.percentile > 0.1)
    return (
      <span className={cls + " text-red-600"}>
        <DownArrow />
        <DownArrow />
      </span>
    );

  return (
    <span className={cls + " text-red-800"}>
      <DownArrow />
      <DownArrow />
      <DownArrow />
    </span>
  );
}

function StatCell(
  digits: number,
  aggKey: string | null = null,
  inverse: boolean = false
) {
  return (props: CellContext<RowData, unknown>) => {
    const data = props.getValue() as number;
    const orig = props.row.original;

    const pcts = (props.table.options.meta as any).pcts as PercentileResponse;
    const display = (props.table.options.meta as any).display as StatDisplay;

    let percentile: number | undefined = undefined;
    if (aggKey && pcts.leagues[orig.league][aggKey]) {
      percentile = findPercentile(
        pcts.leagues[orig.league][aggKey],
        data,
        inverse
      );
    }

    const scale = defaultScale;

    const lightColor = scale.light(percentile ?? 0);
    const darkColor = scale.dark(percentile ?? 0);

    return (
      <div
        className={clsx(
          `text-right tabular-nums p-2`,
          aggKey && "font-semibold dark:font-medium"
        )}
        style={{
          color: aggKey
            ? `light-dark(${lightColor.css()}, ${darkColor.css()})`
            : undefined,
        }}
      >
        {data === undefined || isNaN(data) ? (
          "-"
        ) : display === "percentile" && percentile !== undefined ? (
          (percentile * 100).toFixed(0) + "%"
        ) : display === "vibes" && percentile !== undefined ? (
          <PercentileVibes percentile={percentile} />
        ) : (
          data.toFixed(digits)
        )}
      </div>
    );
  };
}

function NormalCell() {
  return (props: CellContext<RowData, unknown>) => {
    return <div className="p-2">{props.getValue()?.toString()}</div>;
  };
}

function StatFooter() {
  return (props: HeaderContext<RowData, unknown>) => {
    return <div className="p-2">todo</div>;
  };
}

function InningsCell() {
  return (props: CellContext<RowData, unknown>) => {
    const data = props.getValue() as number;

    const innings = Math.floor(data);
    const outs = Math.floor(data * 3) % 3;
    return (
      <div className="tabular-nums p-2 font-medium text-right">
        {innings}.{outs}
      </div>
    );
  };
}

function SortableHeader(name: string, alignRight: boolean = false) {
  return (props: HeaderContext<RowData, unknown>) => {
    return (
      <div
        className={clsx(
          "flex items-center cursor-pointer gap-1",
          alignRight ? "flex-row-reverse" : "flex-row"
        )}
        onClick={() => props.column.toggleSorting()}
      >
        <span>{name}</span>
        {props.column.getIsSorted() === "asc" && (
          <ChevronUp className="h-4 w-4" />
        )}
        {props.column.getIsSorted() === "desc" && (
          <ChevronDown className="h-4 w-4" />
        )}
        {props.column.getIsSorted() !== "asc" &&
          props.column.getIsSorted() !== "desc" && (
            <div className="h-4 w-4"></div>
          )}
      </div>
    );
  };
}

function IdCell(props: CellContext<RowData, unknown>) {
  return <div className="p-2 font-mono">{props.getValue()?.toString()}</div>;
}

function NameCell(props: CellContext<RowData, unknown>) {
  const player = props.row.original;
  return (
    <a
      href={`https://mmolb.com/player/${player.id}`}
      className="p-2 hover:underline font-semibold"
    >
      {props.getValue()?.toString()}
    </a>
  );
}

const columnsBase: ColumnDef<RowData>[] = [
  // {
  //   header: SortableHeader("ID"),
  //   accessorKey: "id",
  //   cell: IdCell,
  // },
  {
    header: SortableHeader("Name"),
    accessorKey: "name",
    cell: NameCell,
  },
  {
    header: SortableHeader("Pos."),
    accessorKey: "position",
    sortingFn: (a, b) => {
      return a.original.rosterIndex > b.original.rosterIndex ? 1 : a.original.rosterIndex < b.original.rosterIndex ? -1 : 0;
    },
    cell: NormalCell(),
  },
];

const preColumnsBatting: ColumnDef<RowData>[] = [
  {
    header: SortableHeader("#"),
    accessorKey: "lineupIndex",
    sortDescFirst: false,
    sortUndefined: 'last',
    cell: NormalCell(),
  },
];

const columnsBatting: ColumnDef<RowData>[] = [
  {
    header: SortableHeader("PAs", true),
    accessorKey: "plate_appearances",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("ABs", true),
    accessorKey: "at_bats",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("H", true),
    accessorKey: "hits",
    cell: StatCell(0),
    // footer: StatFooter(),
  },
  {
    header: SortableHeader("2B", true),
    accessorKey: "doubles",
    cell: StatCell(0),
    // footer: StatFooter(),
  },
  {
    header: SortableHeader("3B", true),
    accessorKey: "triples",
    cell: StatCell(0),
    // footer: StatFooter(),
  },
  {
    header: SortableHeader("HR", true),
    accessorKey: "home_runs",
    cell: StatCell(0),
    // footer: StatFooter(),
  },
  {
    header: SortableHeader("BB", true),
    accessorKey: "walked",
    cell: StatCell(0),
    // footer: StatFooter(),
  },
  {
    header: SortableHeader("K", true),
    accessorKey: "struck_out",
    cell: StatCell(0),
    // footer: StatFooter(),
  },
  {
    header: SortableHeader("BA", true),
    accessorKey: "ba",
    cell: StatCell(3, "ba"),
    // footer: StatFooter(),
  },
  {
    header: SortableHeader("OBP", true),
    accessorKey: "obp",
    cell: StatCell(3, "obp"),
  },
  {
    header: SortableHeader("SLG", true),
    accessorKey: "slg",
    cell: StatCell(3, "slg"),
  },
  {
    header: SortableHeader("OPS", true),
    accessorKey: "ops",
    cell: StatCell(3, "ops"),
  },
  {
    header: SortableHeader("OPS+", true),
    accessorKey: "ops_plus",
    cell: StatCell(0, null),
  },
  {
    header: SortableHeader("SB", true),
    accessorKey: "stolen_bases",
    cell: StatCell(0, null),
  },
  {
    header: SortableHeader("CS", true),
    accessorKey: "caught_stealing",
    cell: StatCell(0, null),
  },
  {
    id: "sb_success",
    header: SortableHeader("SB%", true),
    accessorFn: (data) =>
      isNaN(data.sb_success) ? undefined : data.sb_success,
    cell: StatCell(2, "sb_success"),
  },
];

const columnsPitching: ColumnDef<RowData>[] = [
  {
    header: SortableHeader("G", true),
    accessorKey: "appearances",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("GS", true),
    accessorKey: "starts",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("IP", true),
    accessorKey: "ip",
    cell: InningsCell(),
  },
  {
    header: SortableHeader("W", true),
    accessorKey: "wins",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("L", true),
    accessorKey: "losses",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("H", true),
    accessorKey: "hits_allowed",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("HR", true),
    accessorKey: "home_runs_allowed",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("K", true),
    accessorKey: "strikeouts",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("BB", true),
    accessorKey: "walks",
    cell: StatCell(0),
  },
  {
    header: SortableHeader("ERA", true),
    accessorKey: "era",
    cell: StatCell(2, "era", true),
  },
  {
    header: SortableHeader("ERA-", true),
    accessorKey: "era_minus",
    cell: StatCell(0, null),
  },
  {
    header: SortableHeader("FIP", true),
    accessorKey: "fip",
    cell: StatCell(2, null, true),
  },
  {
    header: SortableHeader("FIP-", true),
    accessorKey: "fip_minus",
    cell: StatCell(0, null),
  },
  {
    header: SortableHeader("WHIP", true),
    accessorKey: "whip",
    cell: StatCell(2, "whip", true),
  },
  {
    header: SortableHeader("H/9", true),
    accessorKey: "h9",
    cell: StatCell(2, "h9", true),
  },
  {
    header: SortableHeader("HR/9", true),
    accessorKey: "hr9",
    cell: StatCell(2, "hr9", true),
  },
  {
    header: SortableHeader("K/9", true),
    accessorKey: "k9",
    cell: StatCell(2, "k9"),
  },
  {
    header: SortableHeader("BB/9", true),
    accessorKey: "bb9",
    cell: StatCell(2, "bb9", true),
  },
];

export default function StatsTable(props: StatsTableProps) {
  const data = useMemo(() => {
    return processStats(props);
  }, [props.data, props.players, props.teams]);

  const columns = useMemo(() => {
    if (props.type === "batting")
      return [...preColumnsBatting, ...columnsBase, ...columnsBatting];
    if (props.type === "pitching") return [...columnsBase, ...columnsPitching];
    return columnsBase;
  }, [props.type]);

  const [sorting, setSorting] = useState<SortingState>([
    { desc: false, id: "position" },
  ]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    enableSortingRemoval: false,
    state: {
      sorting,
    },
    meta: {
      aggs: props.aggs,
      pcts: props.pcts,
      display: props.display,
    },
  });

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => {
                return (
                  <TableHead
                    key={header.id}
                    className="font-semibold text-left"
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
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id} className="p-0">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No results.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
        <TableFooter>
          {table.getFooterGroups().map((footerGroup) => (
            <TableRow key={footerGroup.id}>
              {footerGroup.headers.map((header) => {
                return (
                  <TableHead key={header.id} className="font-semibold">
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.footer,
                          header.getContext()
                        )}
                  </TableHead>
                );
              })}
            </TableRow>
          ))}
        </TableFooter>
      </Table>
    </div>
  );
}

function findSlot(
  team: MmolbTeam,
  playerId: string
): { slot: MmolbRosterSlot | undefined; index: number } {
  const index = team.Players.findIndex((x) => x.PlayerID === playerId);
  return { slot: team.Players[index], index: index };
}

function processStats(props: StatsTableProps): RowData[] {
  const data: RowData[] = [];
  for (let row of props.data) {
    const team_data = props.teams[row.team_id];

    const stats = calculateAdvancedStats(
      row,
      props.aggs.find(x => x.league_id == team_data.League)
    );
    if (props.type == "batting" && stats.plate_appearances == 0) continue;
    if (props.type == "pitching" && stats.ip == 0) continue;

    const player = props.players[row.player_id]!;
    const name = player.FirstName + " " + player.LastName;
    let slot: MmolbRosterSlot | undefined, slotIndex: number;
    ({slot, index: slotIndex} = findSlot(team_data, row.player_id));
    if ((props.type == "pitching" && slotIndex < 9)) slotIndex += 999;
    const position = slot?.Slot ?? player.Position;
    const id = row.player_id;

    const arrayIndex = props.lineupOrder.findIndex((x) => x === id)
    const lineupIndex = arrayIndex < 0 ? undefined : arrayIndex + 1

    data.push({
      ...stats,
      name,
      position,
      rosterIndex: slotIndex,
      id,
      team: row.team_id,
      league: team_data.League,
      lineupIndex,
    });
  }
  return data;
}
