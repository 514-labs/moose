import { faker } from "@faker-js/faker";

export interface Tag {
    name: string;
    description: string;
}

export interface Author {
    userName: string, 
    firstName: string;
    lastName: string;
    email: string;
    avatar: string; //URL
    profileLink: string; //URL
}

export interface Contribution {
    commitHash: string;
    author: Author;
    dateTime: string; // DateTime
}

export enum ContraintType {
    Unique = 'Unique',
    Required = 'Required',
    Nullable = 'Nullable',
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

export enum PrimitiveType {
    Model = 'Model',
    Flow = 'Flow',
    Insight = 'Insight',
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

export interface Snippet {
    content: string;
    language: Language;
}

export interface Primitive {
    name: string;
    primitive_type: PrimitiveType;
    description: string;
    doc_link: string;
    version: string;
    console_path: string;
    count: number;
};

// Generate a primitive for each primitive type
const generatePrimitives = (): Primitive[] => {
    const primitiveTypes = Object.values(PrimitiveType);

    return primitiveTypes.map((primitiveType) => {
        return {
            name: primitiveType,
            primitive_type: primitiveType,
            description: faker.lorem.paragraph(),
            doc_link: faker.internet.url(),
            version: faker.system.semver(),
            console_path: faker.internet.url(),
            count: faker.number.int(),
        }
    })
}

export interface HomeMock {
    primitives: Primitive[];   
};

export const homeMock: HomeMock = {
    primitives: generatePrimitives(),
}