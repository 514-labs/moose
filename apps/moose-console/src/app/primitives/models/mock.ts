import { faker } from '@faker-js/faker';
import { Contribution, Snippet, Tag } from "app/mock";

faker.seed(0);

export enum ContraintType {
    Unique = 'Unique',
    Required = 'Required',
    Nullable = 'Nullable',
}


export enum Language {
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

export interface Field {
    name: string;
    type: string;
    description: string;
    constraints: ContraintType[]; 
    doc_link: string; // Type definition
    consolePath: string;
    rowCount: number;
    messageCount: number;
    lastContribution: Contribution;
    tags: Tag[];
    deployedVersions: string[]; // List of commit hashes/version strings
}


export const generateField = (): Field => ({
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
            firstName: faker.name.firstName(),
            lastName: faker.name.lastName(),
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

export interface Model {
    id: string;
    name: string;
    description: string;
    docLink: string;
    deployedVersions: string[]; // List of commit hashes/version strings
    fields: Field[];
    tags: Tag[];
    lastContributions: Contribution[];
    code: Snippet;
    queueIDs: string[];
    tableIDs: string[];
    viewIDs: string[];
    ingestionPointIDs: string[];
}

export interface ModelMock {
    models: Model[];
}

// Create a prisma model mock from an input model name
// This is a function that takes in a model name and returns a string that represents the prisma model code

const generatePrismaModel = (modelName: string, fields: Field[]): string => {
    const model = `
        model ${modelName} {
            ${fields.map(field => `${field.name} ${field.type}`).join('\n')}
        }
    `;
    return model;
}

export const modelMock: ModelMock = {
    models: Array.from({ length: 10 }, () => {
        const name = faker.commerce.productName();
        const fields = Array.from({ length: 10 }, generateField);
        
        return {
            id: faker.string.uuid(),
            name,
            description: faker.commerce.productDescription(),
            docLink: faker.internet.url(),
            deployedVersions: Array.from({ length: 5 }, () => faker.git.commitSha()),
            fields: Array.from({ length: 10 }, generateField),
            tags: Array.from({ length: 5 }, () => ({
                name: faker.lorem.word(),
                description: faker.lorem.sentence(),
            })),
            lastContributions: Array.from({ length: 5 }, () => ({
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
            })),
            code: {
                content: generatePrismaModel(name, fields),
                language: Language.Prisma,
            }, 
            queueIDs: Array.from({ length: 5 }, () => faker.string.uuid()),
            tableIDs: Array.from({ length: 5 }, () => faker.string.uuid()),
            viewIDs: Array.from({ length: 5 }, () => faker.string.uuid()),
            ingestionPointIDs: Array.from({ length: 5 }, () => faker.string.uuid()),
            }
    })
    ,
};