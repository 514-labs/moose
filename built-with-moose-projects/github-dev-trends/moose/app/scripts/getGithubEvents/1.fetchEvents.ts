import { TaskFunction, TaskDefinition, cliLog } from "@514labs/moose-lib";
import {
  EventType,
  EventResponseType,
  EventRequestOptions,
  FetchEventsOutput,
  createOctokit,
  loadEtag,
  saveEtag,
} from "../utils";

const fetchEvents: TaskFunction = async (input?: any) => {
  console.log("Fetching latest GitHub public events");

  const octokit = createOctokit();

  let allEvents: EventType = [];
  const maxPages = 3; // Limit to 3 pages (300 events) to avoid rate limits
  let noNewEvents = false;
  const etag = loadEtag();

  try {
    // Fetch events with pagination
    for (let page = 1; page <= maxPages; page++) {
      try {
        const requestOptions: EventRequestOptions = {
          per_page: 100, // Maximum allowed per page
          page: page,
        };

        if (page == 1 && etag) {
          requestOptions.headers = {
            "If-None-Match": etag,
          };
        }

        const response: EventResponseType =
          await octokit.rest.activity.listPublicEvents(requestOptions);

        // Save the new etag for future requests
        const newEtag = response.headers.etag as string;
        if (newEtag && newEtag !== etag) {
          saveEtag(newEtag);
        }

        // Add current page events to our collection
        if (page === 1) {
          allEvents = response.data;
        } else {
          allEvents = [...allEvents, ...response.data];
        }

        // Check for Link header to handle pagination
        const hasNextPage =
          response.headers.link?.includes('rel="next"') || false;
        if (!hasNextPage) {
          break; // Exit the loop if no more pages
        }
      } catch (error: any) {
        // Handle 304 Not Modified (only possible on the first page)
        if (page === 1 && error.status === 304) {
          console.log(
            "304 Not Modified - No new events available since last fetch",
          );
          noNewEvents = true;
          break; // Exit the loop, no need to fetch more pages
        } else {
          throw error; // Re-throw other errors
        }
      }
    }

    const pagesProcessed =
      allEvents.length > 0 ? Math.ceil(allEvents.length / 100) : 0;
    console.log(
      `${noNewEvents ? "No new events" : `Successfully fetched ${allEvents.length} events from ${pagesProcessed} pages`}`,
    );

    return {
      task: "fetchEvents",
      data: {
        events: noNewEvents ? [] : allEvents,
        fetchedAt: new Date().toISOString(),
        count: allEvents.length,
        noNewEvents,
      },
    } as { task: string; data: FetchEventsOutput };
  } catch (error) {
    console.error("Error fetching GitHub events:", error);
    throw error; // Let the task framework handle retries
  }
};

export default function createTask() {
  return {
    task: fetchEvents,
    config: {
      retries: 3,
    },
  } as TaskDefinition;
}
