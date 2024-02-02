import { DataTable } from 'components/data-table';
import React from 'react';
import { Table, getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";
import Link from 'next/link';
import { Separator } from 'components/ui/separator';

async function TablesPage({searchParams}) {
    noStore();
    const data = await getCliData();

    let tables: Table[] = data.tables.filter(t => !t.name.includes('.inner'));

    if (searchParams.type === 'view') {
        tables = (tables.filter( t => t.engine === "MaterializedView"))
    } else if (searchParams.type === 'table') {
        tables = tables.filter( t => t.engine !== "MaterializedView")
    } else {
        tables = tables;
    }

    return (
        <section className="p-4 max-h-screen overflow-y-auto grow">
            <div className="py-10">
                <div className="text-6xl">
                    <Link className="text-muted-foreground" href={"/"}>overview/</Link>
                    {searchParams.type === 'view' ? 'Views' : 'Tables'}
                </div>
            </div>
            <div className="">
            <Separator />
            {tables.map((table, index) => (
                <Link key={index} href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`} >
                <div key={index} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"> 
                    <div className="py-2 flex flex-row">
                        <div>
                            <div>{table.name}</div>
                            <div className="text-muted-foreground">{table.database}</div>
                        </div>
                        <span className="flex-grow"/>
                    </div>
                    <Separator/>
                </div>
                </Link>
            ))}
        </div>      
        </section>
    );
}

export default TablesPage;