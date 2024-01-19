import { faker } from '@faker-js/faker';
import { Contribution, Field, Language, Snippet, Tag, generateField } from "app/mock";

enum ContraintType {
    Unique = 'Unique',
    Required = 'Required',
    Nullable = 'Nullable',
}


interface Model {
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

interface ModelMock {
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