"use server"
import { createClient } from "@clickhouse/client"

const clickhouseClient = createClient({
    host: "http://localhost:18123",
    username: "panda",
    password: "pandapass",
    database: "local",
});


export const getData = async (query: string):Promise<object[]> => {
    if (!query) {
        return [];
    }
    const resultSet = await clickhouseClient.query({
        query,
        format: "JSONEachRow",
    });

    return await resultSet.json();
};