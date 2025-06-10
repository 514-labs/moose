import { ConsumptionApi } from "@514labs/moose-lib";
import { config } from "dotenv";
import axios from "axios";

// Load environment variables from .env.local
config({ path: "../../../.env.local" });

const AI_PROMPT = `
You are a neuroscientist analyzing brain data. When you're provided a set of brain data 
you'll provide expert analysis in Markdown format.
`;

// This file is where you can define your API templates for consuming your data
// All query_params are passed in as strings, and are used within the sql tag to parameterize you queries
interface QueryParams {
  sessions: string; // sessionId|sessionLabel, sessionId|sessionLabel, ...
}

interface SessionInsightResponse {
  status: number;
  success: boolean;
  data: {
    insights: string | null;
    queryResult: any[];
  };
}

export const sessionInsightsApi = new ConsumptionApi<
  QueryParams,
  SessionInsightResponse
>("session-insights", async ({ sessions }, { client, sql }) => {
  const openAIKey = process.env.OPENAI_API_KEY;

  // Parse the sessions string into an array of sessionId and sessionLabel pairs
  const sessionData = sessions.split(",").map((session) => {
    const [sessionId, sessionLabel] = session.trim().split("|");
    return { sessionId, sessionLabel };
  });

  // Map through each sessionData and run the query
  const queryResults = await Promise.all(
    sessionData.map(async ({ sessionId, sessionLabel }) => {
      const result = await client.query.execute(sql`SELECT
          sessionId,
          ${sessionLabel} AS sessionLabel,
          SUM(sqrt((arrayElement(acc, 1) * arrayElement(acc, 1)) +
                   (arrayElement(acc, 2) * arrayElement(acc, 2)) +
                   (arrayElement(acc, 3) * arrayElement(acc, 3)))) AS acc_movement_score,
          SUM(sqrt((arrayElement(gyro, 1) * arrayElement(gyro, 1)) +
                   (arrayElement(gyro, 2) * arrayElement(gyro, 2)) +
                   (arrayElement(gyro, 3) * arrayElement(gyro, 3)))) AS gyro_movement_score,
          (SUM(sqrt((arrayElement(acc, 1) * arrayElement(acc, 1)) +
                    (arrayElement(acc, 2) * arrayElement(acc, 2)) +
                    (arrayElement(acc, 3) * arrayElement(acc, 3)))) +
           SUM(sqrt((arrayElement(gyro, 1) * arrayElement(gyro, 1)) +
                    (arrayElement(gyro, 2) * arrayElement(gyro, 2)) +
                    (arrayElement(gyro, 3) * arrayElement(gyro, 3))))) AS total_movement_score
      FROM
          Brain
      WHERE
          sessionId = ${sessionId}
      GROUP BY
          sessionId`);
      return result;
    }),
  );

  const formattedData = await Promise.all(
    queryResults.map(async (queryResult) => {
      const result = await queryResult.json();
      return result.map((row: any) => row);
    }),
  );

  let openAIResponse = null;

  if (openAIKey) {
    openAIResponse = await queryOpenAI(
      openAIKey,
      `${AI_PROMPT} ${JSON.stringify(formattedData)}`,
    );
  }

  return {
    status: 200,
    success: true,
    data: {
      insights: openAIResponse,
      queryResult: formattedData,
    },
  };
});

/**
 * Queries the OpenAI API with a given prompt.
 * @param apiKey - The OpenAI API key.
 * @param prompt - The prompt to send to OpenAI.
 * @returns The response from OpenAI.
 */
async function queryOpenAI(apiKey: string, prompt: string): Promise<any> {
  try {
    const response = await axios.post(
      "https://api.openai.com/v1/chat/completions",
      {
        model: "gpt-4o",
        store: true,
        messages: [{ role: "user", content: prompt }],
      },
      {
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
      },
    );
    return response.data.choices[0].message.content;
  } catch (error) {
    console.error("Error querying OpenAI:", error);
    throw error;
  }
}
