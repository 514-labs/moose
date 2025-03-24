import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";
import * as XLSX from "xlsx";
import * as fs from "fs";
import * as path from "path";
import { v4 as uuidv4 } from "uuid";
import axios from "axios";
import { TransportSchedule } from "../../datamodels/TransportSchedule";

/**
 * Interface for the input parameters of the ImportTransportSchedule task
 */
interface ImportTransportScheduleInput {
  filePath?: string; // Path to the Excel file containing transport schedule data
  sheetName?: string; // Name of the sheet containing the data
}

/**
 * Task to import transport schedule data from Excel file into the TransportSchedule data model
 *
 * @param input - The input parameters for the task
 * @returns Object containing task name and imported data
 */
const importTransportSchedule: TaskFunction = async (
  input: ImportTransportScheduleInput,
) => {
  console.log("ImportTransportSchedule: Starting import task");

  try {
    // Set default values if not provided
    const filePath =
      input.filePath || "./data/Transport_Schedule(Transport) (1).xlsx";
    const sheetName = input.sheetName || "Sheet 1 - Transport_Schedule(Tr";

    console.log(
      `Starting import of transport schedule from ${filePath}, sheet: ${sheetName}`,
    );

    // Check if file exists
    if (!fs.existsSync(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }

    // Read the Excel file
    const workbook = XLSX.readFile(filePath);

    // Check if the specified sheet exists
    if (!workbook.SheetNames.includes(sheetName)) {
      throw new Error(
        `Sheet '${sheetName}' not found in workbook. Available sheets: ${workbook.SheetNames.join(", ")}`,
      );
    }

    // Get the worksheet
    const worksheet = workbook.Sheets[sheetName];

    // Convert the worksheet to JSON with headers
    const rawData = XLSX.utils.sheet_to_json(worksheet, { header: 1 });

    if (rawData.length <= 1) {
      console.warn("No data found in the Excel sheet or only headers present");
      return {
        task: "ImportTransportSchedule",
        data: {
          success: false,
          message: "No data found in the Excel sheet or only headers present",
          records: [],
        },
      };
    }

    // Extract headers from the first row
    const headers = rawData[0] as string[];

    console.log(`Found ${rawData.length - 1} records in the Excel sheet`);

    // Process and validate the data
    const transportScheduleData: TransportSchedule[] = [];
    const errors: string[] = [];

    // Map Excel column indices to our data model fields
    const fieldMapping: Record<string, string> = {
      "Tranport Ring Name": "transportRingName",
      Priority: "priority",
      "ISP Ready Date if Different from Field Ops Date": "ispReadyDate",
      "Field Ops Date": "fieldOpsDate",
      "inCUR Region - HUB A": "hubARegion",
      "inCUR Market - HUB A": "hubAMarket",
      "inCUR Location - HUB A": "hubALocation",
      "HUB A": "hubAName",
      "HUB A CLLI": "hubACLLI",
      "CITY ": "hubACity",
      STATE: "hubAState",
      "inCUR Region- HUB B": "hubBRegion",
      "inCUR Market - HUB B": "hubBMarket",
      "inCUR Location - HUB B": "hubBLocation",
      "HUB B": "hubBName",
      "HUB B CLLI": "hubBCLLI",
      CITY: "hubBCity",
      STATE2: "hubBState",
      "HUB A-B DISTANCE (KM)": "distanceKM",
      "Hub A - Hub B": "hubAToHubB",
      "Hub A Cilli - Hub B Cilli": "hubACLLIToHubBCLLI",
      Comments: "comments",
      "ISP Actual Date Complete": "ispActualDateComplete",
    };

    // Convert header indices to column mapping
    const columnMap: Record<number, string> = {};
    headers.forEach((header, index) => {
      if (fieldMapping[header]) {
        columnMap[index] = fieldMapping[header];
      }
    });

    // Process each data row
    for (let i = 1; i < rawData.length; i++) {
      try {
        const row = rawData[i] as any[];

        // Skip empty rows
        if (!row || row.length === 0) continue;

        // Create a transport schedule item
        const item: Partial<TransportSchedule> = {
          id: uuidv4(), // Generate a unique ID
        };

        // Map data from Excel columns to our data model fields
        Object.entries(columnMap).forEach(([columnIndex, fieldName]) => {
          const value = row[parseInt(columnIndex)];

          // Skip undefined or null values
          if (value === undefined || value === null) return;

          // Handle different field types
          switch (fieldName) {
            case "ispReadyDate":
            case "fieldOpsDate":
              // Handle Excel date values
              if (typeof value === "string") {
                try {
                  item[fieldName] = new Date(value);
                } catch (e) {
                  // Skip if we can't parse the date
                  console.warn(`Could not parse date value: ${value}`);
                }
              } else if (typeof value === "number") {
                // Excel stores dates as days since 1/1/1900
                const excelEpoch = new Date(1899, 11, 30);
                const date = new Date(excelEpoch);
                date.setDate(excelEpoch.getDate() + value);
                item[fieldName] = date;
              }
              break;
            case "priority":
            case "distanceKM":
              item[fieldName] =
                typeof value === "number" ? value : parseFloat(value);
              break;
            default:
              item[fieldName] = String(value);
          }
        });

        // Validate the transformed item - add minimal validation to ensure it has required fields
        if (!item.transportRingName || !item.hubAName || !item.hubBName) {
          errors.push(
            `Row ${i + 1}: Missing required fields (transportRingName, hubAName, or hubBName)`,
          );
          continue;
        }

        transportScheduleData.push(item as TransportSchedule);
      } catch (error) {
        errors.push(
          `Error processing row ${i + 1}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }

    // Log any errors
    if (errors.length > 0) {
      console.warn(`Encountered ${errors.length} errors during import:`);
      errors.slice(0, 5).forEach((error) => console.warn(error));
      if (errors.length > 5) {
        console.warn(`... and ${errors.length - 5} more errors`);
      }
    }

    console.log(
      `Successfully processed ${transportScheduleData.length} transport schedule records`,
    );

    // Chunk the data into batches to avoid hitting request size limits
    const batchSize = 100;
    const batches: TransportSchedule[][] = [];

    for (let i = 0; i < transportScheduleData.length; i += batchSize) {
      batches.push(transportScheduleData.slice(i, i + batchSize));
    }

    console.log(
      `Sending data in ${batches.length} batches of up to ${batchSize} records each`,
    );

    // Send each batch to the ingest API endpoint
    const ingestResults: Array<{
      batch: number;
      success: boolean;
      records: number;
      response?: any;
      error?: string;
    }> = [];

    for (let i = 0; i < batches.length; i++) {
      try {
        console.log(
          `Sending batch ${i + 1} of ${batches.length} (${batches[i].length} records)`,
        );

        // Send data to the ingest API endpoint - adjust URL as needed
        const response = await axios.post(
          "http://localhost:4000/ingest/TransportSchedule",
          batches[i],
          {
            headers: {
              "Content-Type": "application/json",
            },
          },
        );

        ingestResults.push({
          batch: i + 1,
          success: true,
          records: batches[i].length,
          response: response.data,
        });

        console.log(`Successfully sent batch ${i + 1} to ingest API`);
      } catch (error) {
        console.error(
          `Failed to send batch ${i + 1} to ingest API: ${error instanceof Error ? error.message : String(error)}`,
        );

        ingestResults.push({
          batch: i + 1,
          success: false,
          records: batches[i].length,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    // Return the processed data summary
    return {
      task: "ImportTransportSchedule",
      data: {
        success: true,
        message: `Imported ${transportScheduleData.length} transport schedule records with ${errors.length} errors`,
        totalRecords: transportScheduleData.length,
        errorCount: errors.length,
        sampleErrors: errors.slice(0, 5),
        batches: batches.length,
        ingestResults,
        timestamp: new Date().toISOString(),
      },
    };
  } catch (error) {
    console.error(
      `Failed to import transport schedule: ${error instanceof Error ? error.message : String(error)}`,
    );

    // Return error information
    return {
      task: "ImportTransportSchedule",
      data: {
        success: false,
        message: `Failed to import transport schedule: ${error instanceof Error ? error.message : String(error)}`,
        records: 0,
        errors: [error instanceof Error ? error.message : String(error)],
        timestamp: new Date().toISOString(),
      },
    };
  }
};

/**
 * Creates and returns the task definition for the ImportTransportSchedule task
 */
export default function createTask(): TaskDefinition {
  return {
    task: importTransportSchedule,
    config: {
      retries: 3, // Retry up to 3 times if the task fails
    },
  };
}
