import { BaseResultSet, Row, createClient } from '@clickhouse/client-web'

async function getTable(tableName: string): Promise<any> {
    const client = createClient({
        host: "http://localhost:18123",
        username: "panda",
        password: "pandapass",
        database: "local"
    })

    const resultSet = await client.query({
        query: `SELECT * FROM ${tableName} LIMIT 10`,
        format: 'JSONEachRow'
    })
    
    
    return resultSet.json()
}


interface TableProps {
    data: any[];
  }
  
  export const Table = ({ data }: TableProps) => {
    // Get column headers (keys from the first object in the data array)
    const headers = data.length > 0 ? Object.keys(data[0]) : [];
  
    return (
      <table>
        <thead>
          <tr>
            {headers.map((header, index) => (
              <th key={index}>{header}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.map((row, rowIndex) => (
            <tr key={rowIndex}>
              {headers.map((header, cellIndex) => (
                <td key={cellIndex}>{row[header]}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    );
  };






export default async function Page({ params }: { params: { tableName: string } }) {
    const tableData = await getTable(params.tableName);

    return <div>
        <Table data={tableData} />
    </div>
  }