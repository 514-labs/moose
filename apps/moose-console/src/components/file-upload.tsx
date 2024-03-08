import { Project } from 'app/db'
import { createReadStream } from 'fs'
import { writeFile } from 'fs/promises'
import { getClient } from 'lib/clickhouse'
import { join } from 'path'
import { Readable } from 'stream'


export default function FileUpload({ project }: { project: Project }) {
    async function upload(data: FormData) {
        'use server'

        const client = getClient(project, true)

        const file: File | null = data.get('file') as unknown as File
        if (!file) {
            throw new Error('No file uploaded')
        }
        

        const bytes = await file.arrayBuffer()
        const buffer = Buffer.from(bytes)

        const readable = Readable.from(buffer, { objectMode: false })



        client.insert({
            table: "UserEvent_0_0_trigger",
            format: "CSVWithNames",
            values: readable
        })

        // With the file data in the buffer, you can do whatever you want with it.
        // For this, we'll just write it to the filesystem in a new location

        // const path = join('/', 'tmp', file.name)
        // await writeFile(path, buffer)

        return { success: true }
    }

    return (
        <main>
            <h1>React Server Component: Upload</h1>
            <form action={upload}>
                <input type="file" name="file" />
                <input type="submit" value="Upload" />
            </form>
        </main>
    )
}