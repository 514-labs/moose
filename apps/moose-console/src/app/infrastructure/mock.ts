import { faker } from "@faker-js/faker";
import { Field, Language, Snippet, generateField } from "app/mock";

interface InfrastuctureMock {
    tables: Table[];
    views: View[];
    queues: Queue[];
    ingestionPoints: IngestionPoint[];
}

enum QueueStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
    Error = 'Error',
}

interface Queue {
    id: string;
    name: string;
    clusterId: string;
    modelId: string;
    description: string;
    status: QueueStatus;
    connectionUrl: string;
    docLink: string;
    version: string; // Provider version number
    consolePath: string;
    lastUpdated: string;
    lastUpdatedBy: string; // As defined by the infra itself and its auth system
    errors: string[];
}

const generateQueue = (): Queue => ({
    id: faker.datatype.uuid(),
    name: faker.commerce.productName(),
    clusterId: faker.datatype.uuid(),
    modelId: faker.datatype.uuid(),
    description: faker.commerce.productDescription(),
    status: faker.helpers.arrayElement(Object.values(QueueStatus)),
    connectionUrl: faker.internet.url(),
    docLink: faker.internet.url(),
    version: faker.system.semver(),
    consolePath: faker.system.directoryPath(),
    lastUpdated: faker.date.recent().toISOString(),
    lastUpdatedBy: faker.internet.userName(),
    errors: [],
});


enum IngestionPointStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
    Error = 'Error',
}

interface SdkInfo {
}

interface IngestionPoint {
    id: string;
    name: string;
    description: string;
    status: IngestionPointStatus;
    connectionURL: string;
    docLink: string;
    version: string; // Provider version number
    consolePath: string;
    lastUpdated: string;
    lastUpdatedBy: string; // As defined by the infra itself and its auth system
    snippets: Snippet[];
    sdks: SdkInfo[];
    serverId: string;
    modelId: string;
    errors: string[];
}

const generateIngestionPoint = (): IngestionPoint => ({
    id: faker.datatype.uuid(),
    name: faker.commerce.productName(),
    description: faker.commerce.productDescription(),
    status: faker.helpers.arrayElement(Object.values(IngestionPointStatus)),
    connectionURL: faker.internet.url(),
    docLink: faker.internet.url(),
    version: faker.system.semver(),
    consolePath: faker.system.directoryPath(),
    lastUpdated: faker.date.recent().toISOString(),
    lastUpdatedBy: faker.internet.userName(),
    snippets: Array.from({ length: 5 }, () => ({
        content: faker.lorem.paragraph(),
        language: faker.helpers.arrayElement(Object.values(Language)),
    })),
    sdks: Array.from({ length: 5 }, () => ({
        language: faker.helpers.arrayElement(Object.values(Language)),
        version: faker.system.semver(),
    })),
    serverId: faker.datatype.uuid(),
    modelId: faker.datatype.uuid(),
    errors: [],
});

enum DatabaseStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
    Error = 'Error',
}

interface Database {
    id: string;
    name: string;
    description: string;
    status: DatabaseStatus;
    connectionURL: string;
    version: string;
    tables: Table[];
    views: View[];
    docsLink: string;
    consolePath: string;
    tableCount: number;
    viewCount: number;
    lastUpdated: string;
    lastUpdatedBy: string;
    modelIds: string[];
    errors: string[];
}

const generateDatabase = (): Database => {
    const tables = Array.from({ length: 5 }, generateTable);
    // generate a view that references a table
    const views = tables.map(table => {
        const { status, ...sharedFields } = table;

        return {
            parentTable: table.id,
            status: faker.helpers.arrayElement(Object.values(ViewStatus)),
            ...sharedFields,
        }});    

    return {
        id: faker.string.uuid(),
        name: faker.commerce.productName(),
        description: faker.commerce.productDescription(),
        status: faker.helpers.arrayElement(Object.values(DatabaseStatus)),
        connectionURL: faker.internet.url(),
        docsLink: faker.internet.url(),
        version: faker.system.semver(),
        tables: tables,
        views: views,
        consolePath: faker.system.directoryPath(),
        tableCount: faker.datatype.number(),
        viewCount: faker.datatype.number(),
        lastUpdated: faker.date.recent().toISOString(),
        lastUpdatedBy: faker.internet.userName(),
        modelIds: Array.from({ length: 5 }, () => faker.datatype.uuid()),
        errors: [],
    }
    
};

enum TableStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
}

interface Value {
    field: Field;
    value: string;
}

interface Row {
    values: Value[];
}

interface Table {
    id: string;
    name: string;
    description: string;
    status: TableStatus;
    docLink: string;
    version: string;
    consolePath: string;
    fields: Field[];
    fieldCount: number;
    rowCount: number;
    lastUpdated: string;
    lastUpdatedBy: string;
    samples: Row[];
    modelId: string;
    errors: string[];
}

const generateTable = (): Table => ({
    id: faker.string.uuid(),
    name: faker.commerce.productName(),
    description: faker.commerce.productDescription(),
    status: faker.helpers.arrayElement(Object.values(TableStatus)),
    docLink: faker.internet.url(),
    version: faker.system.semver(),
    consolePath: faker.system.directoryPath(),
    fields: Array.from({ length: 5 }, generateField),
    fieldCount: faker.datatype.number(),
    rowCount: faker.datatype.number(),
    lastUpdated: faker.date.recent().toISOString(),
    lastUpdatedBy: faker.internet.userName(),
    samples: Array.from({ length: 5 }, () => ({
        values: Array.from({ length: 5 }, () => ({
            field: generateField(),
            value: faker.lorem.sentence(),
        })),
    })),
    modelId: faker.datatype.uuid(),
    errors: [],
});

enum ViewStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
}

interface View {
    id: string;
    name: string;
    parentTable: string;
    description: string;
    status: ViewStatus;
    docLink: string;
    version: string;
    consolePath: string;
    fields: Field[];
    fieldCount: number;
    rowCount: number;
    lastUpdated: string;
    lastUpdatedBy: string;
    samples: Row[];
    modelId: string;
    errors: string[];
}

const database = generateDatabase();


export const infrastructureMock: InfrastuctureMock = {
    tables: database.tables, 
    views: database.views,
    queues: Array.from({ length: 10 }, generateQueue),
    ingestionPoints: Array.from({ length: 10 }, generateIngestionPoint),
};