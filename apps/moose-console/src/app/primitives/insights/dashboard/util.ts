import { faker } from "@faker-js/faker";


enum EventType {
    Click = 'Click',
    Page = 'Page',
    Track = 'Track'
}

enum Language {
    English = 'english',
    Spanish = 'Spanish',
    French = 'French'
}
export interface Field {
    eventId: String,
    eventName: String,
    eventType: EventType,
    userId: String, // this should be a more universal way of identifying users and should not be the loggedIn userID
    isLoggedIn: Boolean,
    sessionId: String,
    ipAddress: String, //this should get parsed into country and geo codes during Flow transformation
    userAgent: String, // this should get parsed into device OS, OS version, device type, browser type, browser version during Flow
    language: String,
    timestamp: Date,
}

const generateField = (): Field => ({
    eventId: faker.string.uuid(),
    eventName: faker.vehicle.manufacturer(),
    eventType: faker.helpers.enumValue(EventType),
    userId: faker.string.uuid(),
    isLoggedIn: faker.datatype.boolean(),
    sessionId: faker.string.uuid(),
    ipAddress: faker.internet.ipv4(),
    userAgent: faker.internet.userAgent(),
    language: faker.helpers.enumValue(Language),
    timestamp: new Date(faker.date.recent({ days: 6 }).getTime())
});

export function runFaker() {
    for (let i = 0; i < 300; i++) {
        fetch('http://localhost:4000/ingest/UserEvent', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(
                generateField()
            )
        })
    }
}