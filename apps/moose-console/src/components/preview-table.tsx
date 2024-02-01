
import { Table, TableCaption, TableHeader, TableRow, TableHead, TableBody, TableCell } from "./ui/table";

interface TableProps {
    rows: any[];
  }

export function PreviewTable({ rows }: TableProps) {
    // Get column headers (keys from the first object in the data array)
    const headers = rows.length > 0 ? Object.keys(rows[0]) : [];
  
    return (
      <Table>
        <TableCaption>A preview of the data in your table.</TableCaption>
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
  };