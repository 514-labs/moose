// Example flow function: Converts local timestamps in UserActivity data to UTC.

// Imports: Source (UserActivity) and Destination (ParsedActivity) data models.
import { ParsedActivity } from "../../../../.moose/internal-app-sdk/ParsedActivity.ts";
import { UserActivity } from "../../../../.moose/internal-app-sdk/UserActivity.ts";

// The 'run' function transforms UserActivity data to ParsedActivity format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default function run(event: UserActivity): ParsedActivity {
  // Convert local timestamp to UTC and return new ParsedActivity object.
  return {
    eventId: event.eventId, // Retain original event ID.
    userId: "puid" + event.userId, // Example: Prefix user ID.
    activity: event.activity, // Copy activity unchanged.
    timestamp: new Date(event.timestamp.toUTCString()), // Convert timestamp to UTC.
  };
}
