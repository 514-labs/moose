import { createClient } from "@clickhouse/client-web"

const client = createClient({
    host: "http://localhost:18123",
    username: "panda",
    password: "pandapass",
    database: "local",
});


export const getData = async () => {
    const resultSet = await client.query({
        query: "SELECT * FROM UserActivity_trigger;",
        format: "JSONEachRow",
    });

    return await resultSet.json();
};
