'use client'

import { table } from "console"
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from "./ui/resizable"
import { Textarea } from "./ui/textarea"
import { Table } from "app/db"
import { Button } from "./ui/button"
import { useState } from "react"
import { createClient } from "@clickhouse/client-web"
import { PreviewTable } from "./preview-table"

function getClient() {
    const CLICKHOUSE_HOST = process.env.CLICKHOUSE_HOST || "localhost";
    // Environment variables are always strings
    const CLICKHOUSE_PORT = process.env.CLICKHOUSE_PORT || "18123";
    const CLICKHOUSE_USERNAME = process.env.CLICKHOUSE_USERNAME || "panda";
    const CLICKHOUSE_PASSWORD = process.env.CLICKHOUSE_PASSWORD || "pandapass";
    const CLICKHOUSE_DB = "default";

    const client = createClient({
        host: `http://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
        username: CLICKHOUSE_USERNAME,
        password: CLICKHOUSE_PASSWORD,
        database: CLICKHOUSE_DB,
    });

    return client;
}

async function runQuery(queryString: string): Promise<any> {
    const client = getClient();
  
    const resultSet = await client.query({
      query: queryString,
      format: "JSONEachRow",
    });
  
    return resultSet.json();
  }


interface QueryInterfaceProps {
    table: Table
}

export default function QueryInterface({ table }: QueryInterfaceProps) {
    const[value, setValue] = useState(`SELECT * FROM ${table.database}.${table.name};`)
    const [results, setResults] = useState<any[]>([])

    return (
        <ResizablePanelGroup
            className="h-full"
            direction="vertical"
        >
            <ResizablePanel defaultSize={20}>
            <ResizablePanelGroup direction="horizontal">
                <ResizablePanel defaultSize={75}>
                <div className="flex h-full">
                    <Textarea className="border-0 h-full font-mono" placeholder="type your query here" value={value} onChange={(e) => setValue(e.target.value)}/>
                </div>
                </ResizablePanel>
                <ResizableHandle withHandle />
                <ResizablePanel defaultSize={25}>
                <div className="flex h-full  p-6">
                    <span className="font-semibold">Autocomplete objects like fields</span>
                </div>
                </ResizablePanel>
            </ResizablePanelGroup>
            </ResizablePanel>
            <ResizableHandle withHandle />
            <ResizablePanel defaultSize={80}>
            <div className="flex h-full flex-col py-4">
                <div className="flex flex-row items-center">
                    <span className="">Results</span>
                    <span className="flex-grow"/>
                    <span className="px-2">ctrl/cmd + enter</span>                
                    <Button variant="default" onClick={
                        async () => {
                            const results = await runQuery(value)
                            setResults(results)
                        }
                    
                    }>Run</Button>
                </div>
                <div>
                    {results ? <PreviewTable rows={results} /> : ""}
                </div>
            </div>
            </ResizablePanel>
        </ResizablePanelGroup>
    )
}