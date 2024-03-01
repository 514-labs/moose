"use client";

import {
  Table,
  TableCaption,
  TableHeader,
  TableRow,
  TableHead,
  TableBody,
  TableCell,
} from "./ui/table";

interface TableProps<T> {
  rows: T[];
  caption?: string;
  onRowClick?: (row: T) => void;
}

function PreviewTable<T>({ rows, caption, onRowClick }: TableProps<T>) {
  // Get column headers (keys from the first object in the data array)
  const headers = rows.length > 0 ? Object.keys(rows[0]) : [];
  return (
    <Table>
      {caption && <TableCaption>{caption}</TableCaption>}
      <TableHeader>
        <TableRow>
          {headers.map((header, index) => (
            <TableHead key={index} className="font-medium">
              {header}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row, index) => (
          <TableRow
            className={onRowClick ? "cursor-pointer" : ""}
            key={index}
            onClick={() => onRowClick(row)}
          >
            {headers.map((value, index) => (
              <TableCell key={index}>{row[value]}</TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

export { PreviewTable };
