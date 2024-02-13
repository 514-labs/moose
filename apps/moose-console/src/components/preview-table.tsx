
import { Table, TableCaption, TableHeader, TableRow, TableHead, TableBody, TableCell } from "./ui/table";

interface TableProps {
    rows: any[];
    caption?: string;
  }

export function PreviewTable({ rows, caption }: TableProps) {
    // Get column headers (keys from the first object in the data array)
    const headers = rows.length > 0 ? Object.keys(rows[0]) : [];
  
    return (
      <Table>
        { caption && <TableCaption>{caption}</TableCaption> }
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
            <TableRow key={index}>
              {headers.map((value, index) => (
                <TableCell key={index}>{row[value]}</TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    );
  }