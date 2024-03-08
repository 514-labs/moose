"use client"

import { ColumnDef } from "@tanstack/react-table"
import { UserEvent } from '../../../../moose/.moose/moose-sdk/UserEvent'
// This type is used to define the shape of our data.
// You can use a Zod schema here if you want.

export function createColumns<T extends {}>(object: T): ColumnDef<T>[] {
    return Object.keys(object).map(key => ({ accessorKey: key, header: key }))
}

export const columns: ColumnDef<UserEvent>[] = [
    {
        accessorKey: "event_time",
        header: "Time",
    },
    {
        accessorKey: "price",
        header: "Price",
    },
    {
        accessorKey: "event_type",
        header: "Event Type",
    },
    {
        accessorKey: "user_session",
        header: "Session",
    },
    {
        accessorKey: "brand",
        header: "Brand",
    },
    { accessorKey: "user_session", header: "Session" }
]

