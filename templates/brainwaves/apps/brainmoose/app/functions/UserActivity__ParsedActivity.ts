// Example streaming function: Converts local timestamps in UserActivity data to UTC.

// Imports: Source (UserActivity) and Destination (ParsedActivity) data models.
import { ParsedActivity, UserActivity } from "datamodels/models";

// The 'run' function transforms UserActivity data to ParsedActivity format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(source: UserActivity): ParsedActivity {
  // Convert local timestamp to UTC and return new ParsedActivity object.
  return {
    eventId: source.eventId, // Retain original event ID.
    userId: "puid" + source.userId, // Example: Prefix user ID.
    activity: source.activity, // Copy activity unchanged.
    timestamp: new Date(source.timestamp), // Convert timestamp to UTC.
  };
}
