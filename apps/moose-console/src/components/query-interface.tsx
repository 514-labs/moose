"use client";

import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "./ui/resizable";
import { Textarea } from "./ui/textarea";
import { Table } from "app/db";
import { Button } from "./ui/button";
import {
  Dispatch,
  MutableRefObject,
  SetStateAction,
  useEffect,
  useRef,
  useState,
} from "react";
import { createClient } from "@clickhouse/client-web";
import { PreviewTable } from "./preview-table";
import { Badge } from "./ui/badge";
import { Card, CardContent } from "./ui/card";

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

const sqlKeyWords = [
  "SELECT",
  "FROM",
  "WHERE",
  "GROUP BY",
  "ORDER BY",
  "LIMIT",
  "AND",
  "OR",
  "AS",
  "JOIN",
  "LEFT JOIN",
  "RIGHT JOIN",
  "INNER JOIN",
  "OUTER JOIN",
  "ON",
  "IS",
  "NOT",
  "NULL",
  "LIKE",
  "IN",
  "BETWEEN",
  "EXISTS",
  "UNION",
  "ALL",
  "ANY",
  "CASE",
  "WHEN",
  "THEN",
  "ELSE",
  "END",
  "CREATE",
  "TABLE",
  "DROP",
  "ALTER",
  "INDEX",
  "VIEW",
  "SEQUENCE",
  "TRIGGER",
  "PROCEDURE",
  "FUNCTION",
  "PACKAGE",
  "LOCK",
  "EXPLAIN",
  "DESCRIBE",
  "ANALYZE",
  "GRANT",
  "REVOKE",
  "PRIVILEGES",
  "COMMIT",
  "ROLLBACK",
  "SAVEPOINT",
  "SET",
  "SESSION",
  "SYSTEM",
  "USER",
  "DATABASE",
  "SCHEMA",
  "DOMAIN",
  "TYPE",
  "CHARACTER",
  "COLLATION",
  "TRANSLATION",
  "SERVER",
  "CONNECTION",
  "STATEMENT",
  "PREPARE",
  "EXECUTE",
  "DEALLOCATE",
  "WORK",
  "ISOLATION",
  "LEVEL",
  "READ",
  "WRITE",
  "ONLY",
  "CALL",
  "RETURN",
  "HANDLER",
  "CONDITION",
  "SIGNAL",
  "RESIGNAL",
  "ITERATE",
  "LEAVE",
  "LOOP",
  "REPEAT",
  "UNTIL",
  "OPEN",
  "CLOSE",
  "FETCH",
  "DECLARE",
  "CURSOR",
  "CONTINUE",
  "EXIT",
  "GET",
  "DIAGNOSTICS",
  "STACKED",
  "DYNAMIC",
  "STATIC",
  "SENSITIVE",
  "PRIOR",
  "SQLSTATE",
  "SQLCODE",
  "SQLERROR",
  "SQLWARNING",
  "SQLNOTFOUND",
  "SQLROWCOUNT",
  "SQLFOUND",
  "SQL",
];

async function runQuery(queryString: string): Promise<any> {
  const client = getClient();

  const resultSet = await client.query({
    query: queryString,
    format: "JSONEachRow",
  });

  return resultSet.json();
}

interface QueryInterfaceProps {
  table: Table;
  related: Table[];
}

const insertSomeText = (
  insert: string,
  originalValue: string,
  ref: MutableRefObject<HTMLTextAreaElement>,
  setter: Dispatch<SetStateAction<string>>
) => {
  const selectionStart = ref.current.selectionStart;
  const selectionEnd = ref.current.selectionEnd;

  const newValue =
    originalValue.substring(0, selectionStart) +
    insert +
    originalValue.substring(selectionEnd, originalValue.length);
  setter(newValue);
};

export default function QueryInterface({
  table,
  related,
}: QueryInterfaceProps) {
  // Create a ref to the textarea
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const tables = related
    .filter((t) => !t.name.includes(".inner"))
    .map((t) => `${t.database}.${t.name}`);

  const [value, setValue] = useState(
    `SELECT * FROM ${table.database}.${table.name} LIMIT 50;`
  );
  const [results, setResults] = useState<any[]>();
  const [sqlKeyWordCount, setSqlKeyWordCount] = useState(12);
  const [tableCount, setTableCount] = useState(12);

  useEffect(() => {
    (async () => {
      const data = await runQuery(value);
      setResults(data);
    })();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
        runQuery(value).then((results) => {
          e.preventDefault();
          setResults(results);
        });
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [value]);

  return (
    <ResizablePanelGroup className="h-full" direction="vertical">
      <ResizablePanel defaultSize={20}>
        <ResizablePanelGroup direction="horizontal">
          <ResizablePanel defaultSize={75}>
            <div className="p-4 pt-0 pl-0 h-full">
              <Card className="h-full p-0 rounded-3xl">
                <CardContent className="h-full p-1 rounded-3xl ">
                  <div className="h-full rounded-3xl ">
                    <Textarea
                      ref={textareaRef}
                      className="border-0 h-full rounded-3xl font-mono resize-none p-4"
                      placeholder="type your query here"
                      value={value}
                      onChange={(e) => setValue(e.target.value)}
                    />
                  </div>
                </CardContent>
              </Card>
            </div>
          </ResizablePanel>
          <ResizableHandle withHandle className="bg-border-0" />
          <ResizablePanel defaultSize={25}>
            <div className="p-4 pt-0 pr-0 h-full ">
              <Card className="h-full overflow-y-auto rounded-3xl ">
                <CardContent className="py-4">
                  <div className="flex h-full w-full overflow-y-auto">
                    <div className="grow">
                      <div className="text-xs font-mono flex flex-row items-center">
                        <span className="py-2">Tables</span>{" "}
                        <span className="flex-grow" />
                        {tableCount < tables.length ? (
                          <Button
                            variant="ghost"
                            onClick={() => setTableCount(tables.length)}
                          >
                            more
                          </Button>
                        ) : tables.length < tableCount ? (
                          ""
                        ) : (
                          <Button
                            variant="ghost"
                            onClick={() => setTableCount(12)}
                          >
                            less
                          </Button>
                        )}
                      </div>
                      <div className="flex flex-row space-x-1 py-2 flex-wrap">
                        {tables.slice(0, tableCount).map((word, index) => (
                          <Badge
                            onClick={(_e: any) => {
                              insertSomeText(
                                word,
                                value,
                                textareaRef,
                                setValue
                              );
                            }}
                            className="text-nowrap my-1"
                            variant="outline"
                            key={index}
                          >
                            {word}
                          </Badge>
                        ))}
                      </div>
                      <div className="text-xs font-mono flex flex-row items-center">
                        <span>SQL</span> <span className="flex-grow" />
                        {sqlKeyWordCount < sqlKeyWords.length ? (
                          <Button
                            variant="ghost"
                            onClick={() =>
                              setSqlKeyWordCount(sqlKeyWords.length)
                            }
                          >
                            more
                          </Button>
                        ) : (
                          <Button
                            variant="ghost"
                            onClick={() => setSqlKeyWordCount(12)}
                          >
                            less
                          </Button>
                        )}
                      </div>
                      <div className="flex flex-row space-x-1 py-2 flex-wrap">
                        {sqlKeyWords
                          .slice(0, sqlKeyWordCount)
                          .map((word, index) => (
                            <Badge
                              onClick={(_e: any) => {
                                insertSomeText(
                                  word,
                                  value,
                                  textareaRef,
                                  setValue
                                );
                              }}
                              className="text-nowrap my-1"
                              variant="outline"
                              key={index}
                            >
                              {word}
                            </Badge>
                          ))}
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </ResizablePanel>
      <ResizableHandle withHandle className="bg-border-0" />
      <ResizablePanel defaultSize={80} className=" overflow-y-auto">
        <div className="pt-4">
          <Card className="rounded-3xl ">
            <CardContent>
              <div className="flex h-full flex-col py-4">
                <div className="flex flex-row items-center">
                  <span className="">Results</span>
                  <span className="flex-grow" />
                  <span className="px-2">ctrl/cmd + enter</span>
                  <Button
                    variant="default"
                    onClick={async () => {
                      const results = await runQuery(value);
                      setResults(results);
                    }}
                  >
                    Run
                  </Button>
                </div>
                {results ? (
                  <PreviewTable rows={results} caption="query results" />
                ) : (
                  "query results will appear here"
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}
