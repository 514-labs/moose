import { faker } from "@faker-js/faker";
import {  Field, Snippet,  } from "app/mock";

enum ContraintType {
    Unique = 'Unique',
    Required = 'Required',
    Nullable = 'Nullable',
}

enum Language {
    Typescript = 'Typescript',
    Javascript = 'Javascript',
    Python = 'Python',
    Rust = 'Rust',
    Go = 'Go',
    Java = 'Java',
    CPP = 'C++',
    CSharp = 'C#',
    Swift = 'Swift',
    Kotlin = 'Kotlin',
    Fortran = 'Fortran',
    Cobol = 'Cobol',
    Curl = 'curl',
    Bash = 'bash',
    Scala = 'scala',
    Prisma = 'prisma',
}

const generateField = (): Field => ({
    name: faker.database.column(),
    type: faker.database.type(),
    description: faker.lorem.sentence(),
    constraints: [ContraintType.Unique, ContraintType.Required],
    doc_link: faker.internet.url(),
    consolePath: faker.system.directoryPath(),
    rowCount: faker.number.int(),
    messageCount: faker.number.int(),
    lastContribution: {
        commitHash: faker.git.commitSha(),
        author: {
            userName: faker.internet.userName(),
            firstName: faker.person.firstName(),
            lastName: faker.person.lastName(),
            email: faker.internet.email(),
            avatar: faker.internet.avatar(),
            profileLink: faker.internet.url(),
        },
        dateTime: faker.date.recent().toISOString(),
    },
    tags: Array.from({ length: 5 }, () => ({
        name: faker.lorem.word(),
        description: faker.lorem.sentence(),
    })),
    deployedVersions: Array.from({ length: 5 }, () => faker.git.commitSha()),
});


export interface InfrastuctureMock {
    databases: Database[];
    queues: Queue[];
    ingestionPoints: IngestionPoint[];
}

enum QueueStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
    Error = 'Error',
}

export interface Queue {
    id: string;
    name: string;
    clusterId: string;
    messageCount: number;
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
    environment: string;
}

const generateQueue = (): Queue => ({
    id: faker.string.uuid(),
    name: faker.commerce.productName(),
    clusterId: faker.string.uuid(),
    messageCount: faker.number.int(),
    modelId: faker.string.uuid(),
    description: faker.commerce.productDescription(),
    status: faker.helpers.arrayElement(Object.values(QueueStatus)),
    connectionUrl: faker.internet.url(),
    docLink: faker.internet.url(),
    version: faker.system.semver(),
    consolePath: faker.system.directoryPath(),
    lastUpdated: faker.date.recent().toISOString(),
    lastUpdatedBy: faker.internet.userName(),
    errors: [],
    environment: faker.helpers.arrayElement(Object.values(Environment)),
});


enum IngestionPointStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
    Error = 'Error',
}

interface SdkInfo {
}

export interface IngestionPoint {
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
    environment: string;
    
}

const generateIngestionPoint = (): IngestionPoint => ({
    id: faker.string.uuid(),
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
    serverId: faker.string.uuid(),
    modelId: faker.string.uuid(),
    errors: [],
    environment: faker.helpers.arrayElement(Object.values(Environment)),
});

enum DatabaseStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
    Error = 'Error',
}

export interface Database {
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
    environment: string;
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
        tableCount: faker.number.int(),
        viewCount: faker.number.int(),
        lastUpdated: faker.date.recent().toISOString(),
        lastUpdatedBy: faker.internet.userName(),
        modelIds: Array.from({ length: 5 }, () => faker.string.uuid()),
        errors: [],
        environment: faker.helpers.arrayElement(Object.values(Environment)),
    }
    
};

enum TableStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
}

export interface Value {
    field: Field;
    value: string;
}

export interface Row {
    values: Value[];
}

export interface Table {
    id: string;
    databaseId?: string;
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
    environment: string;
}

enum Environment {
    Production = 'Production',
    Staging = 'Staging',
    Development = 'Development',
    Test = 'Test',
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
    fieldCount: faker.number.int(),
    rowCount: faker.number.int(),
    lastUpdated: faker.date.recent().toISOString(),
    lastUpdatedBy: faker.internet.userName(),
    samples: Array.from({ length: 5 }, () => ({
        values: Array.from({ length: 5 }, () => ({
            field: generateField(),
            value: faker.lorem.sentence(),
        })),
    })),
    modelId: faker.string.uuid(),
    errors: [],
    environment: faker.helpers.arrayElement(Object.values(Environment)),
});

enum ViewStatus {
    Active = 'Active',
    Inactive = 'Inactive',
    Deprecated = 'Deprecated',
}

export interface View {
    id: string;
    name: string;
    databaseId?: string;
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
    environment: string;
}



export const infrastructureMock: InfrastuctureMock = {
    databases: Array.from({ length: 10 }, generateDatabase),
    queues: Array.from({ length: 10 }, generateQueue),
    ingestionPoints: Array.from({ length: 10 }, generateIngestionPoint),
};