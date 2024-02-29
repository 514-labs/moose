
import { getCliData } from 'app/db';
import ChartExplorer from './chart-explorer';
import { getClient } from 'lib/clickhouse';


export default async function Page() {
    const data = await getCliData();
    const client = getClient(data.project)
    const resultSet = await client.query({
        query: "SELECT * FROM UserEvent_trigger;",
        format: "JSONEachRow",
    });

    const rows = await resultSet.json() as object[]


    //  deserializeSpec(nah)




    return (
        <section className="p-4 grow overflow-y-scroll">

            <ChartExplorer data={rows} />
        </section>
    )
}