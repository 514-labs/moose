import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";
import * as XLSX from "xlsx";
import * as fs from "fs";
import * as path from "path";
import { v4 as uuidv4 } from "uuid";
import axios from "axios";
import { FiberPanel } from "../../datamodels/FiberPanel";
import { FiberPanelConnection } from "../../datamodels/FiberPanelConnection";

/**
 * Input interface for the ImportFiberPanelData task
 */
interface ImportFiberPanelDataInput {
  filePath?: string; // Path to the Excel file containing fiber panel data
}

/**
 * Interface for connection ingest result
 */
interface ConnectionIngestResult {
  batch: number;
  success: boolean;
  records: number;
  result?: any;
  error?: string;
}

/**
 * Task to import fiber panel data from Excel file into the FiberPanel and FiberPanelConnection data models
 *
 * @param input - The input parameters for the task
 * @returns Object containing task name and imported data
 */
const importFiberPanelData: TaskFunction = async (
  input: ImportFiberPanelDataInput,
) => {
  console.log("ImportFiberPanelData: Starting import task");

  try {
    // Set default values if not provided
    const filePath =
      input.filePath ||
      "./data/incur_site_osp_fiber_panel_report_02-21-2025_2119PM (1).xlsx";

    console.log(`Starting import of fiber panel data from ${filePath}`);

    // Check if file exists
    if (!fs.existsSync(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }

    // Read the Excel file
    const workbook = XLSX.readFile(filePath);

    console.log(
      `Available sheets in the workbook: ${workbook.SheetNames.join(", ")}`,
    );

    // Process the Site Summary sheet to extract fiber panel information
    if (!workbook.SheetNames.includes("Site Summary (OSP)")) {
      throw new Error("Site Summary (OSP) sheet not found in the workbook");
    }

    // Get the Site Summary worksheet
    const summarySheet = workbook.Sheets["Site Summary (OSP)"];

    // Convert the worksheet to JSON with headers
    const summaryData = XLSX.utils.sheet_to_json(summarySheet, { header: 1 });

    if (summaryData.length <= 1) {
      console.warn(
        "No data found in the Site Summary sheet or only headers present",
      );
      return {
        task: "ImportFiberPanelData",
        data: {
          success: false,
          message: "No data found in the Site Summary sheet",
          panels: [],
          connections: [],
        },
      };
    }

    // Extract site location from the first row
    const siteLocationRow =
      summaryData[0] && summaryData[0][0] ? String(summaryData[0][0]) : "";
    // Parse location parts (format is typically Region/Market/Location/CLLI/Main)
    const siteLocation = siteLocationRow.split("/").slice(0, -1).join("/");

    console.log(`Found site location: ${siteLocation}`);

    // Locate the headers row in the summary sheet (usually index 3, which is the 4th row)
    const headerRowIndex = summaryData.findIndex(
      (row) =>
        Array.isArray(row) &&
        row.length >= 2 &&
        row[0] === "Rack Location" &&
        row[1] === "Name",
    );

    if (headerRowIndex === -1) {
      throw new Error("Could not find header row in Site Summary sheet");
    }

    // Process panel summary data
    const fiberPanels: FiberPanel[] = [];
    const errors: string[] = [];

    // Start from the row after the headers
    for (let i = headerRowIndex + 1; i < summaryData.length; i++) {
      const row = summaryData[i] as any[];

      // Skip empty rows or rows without enough data
      if (!row || row.length < 4) continue;

      // Extract panel data
      const rackLocation = String(row[0] || "");
      const panelName = String(row[1] || "");
      const connectedPorts = Number(row[2] || 0);
      const totalPorts = Number(row[3] || 0);

      // Skip if missing required data
      if (!rackLocation || !panelName) {
        errors.push(
          `Row ${i + 1}: Missing required fields (rackLocation or panelName)`,
        );
        continue;
      }

      // Create fiber panel record
      const panel: FiberPanel = {
        id: uuidv4(),
        siteLocation,
        rackLocation,
        panelName,
        connectedPorts,
        totalPorts,
      };

      fiberPanels.push(panel);

      console.log(
        `Processed panel: ${panelName} at ${rackLocation} (${connectedPorts}/${totalPorts} ports connected)`,
      );
    }

    console.log(`Extracted ${fiberPanels.length} fiber panels`);

    // Now process each panel sheet to extract connection details
    const fiberConnections: FiberPanelConnection[] = [];

    for (const panel of fiberPanels) {
      // Look for a sheet named after the rack location
      if (!workbook.SheetNames.includes(panel.rackLocation)) {
        console.warn(`No sheet found for rack location ${panel.rackLocation}`);
        continue;
      }

      console.log(
        `Processing connections for panel ${panel.panelName} at ${panel.rackLocation}`,
      );

      // Get the worksheet
      const panelSheet = workbook.Sheets[panel.rackLocation];

      // Convert the worksheet to JSON with headers
      const panelData = XLSX.utils.sheet_to_json(panelSheet, { header: 1 });

      if (panelData.length <= 1) {
        console.warn(`No data found in the ${panel.rackLocation} sheet`);
        continue;
      }

      // Process each row
      for (let rowIndex = 2; rowIndex < panelData.length; rowIndex++) {
        const row = panelData[rowIndex] as any[];

        // Skip empty rows
        if (!row || row.length === 0) continue;

        // Each column in the row represents a port
        for (let colIndex = 0; colIndex < row.length; colIndex++) {
          const portInfo = String(row[colIndex] || "");

          // Parse port data (format is typically "1: Yes (AM123 Forward)" or "2: No")
          const portMatch = portInfo.match(
            /^(\d+):\s+(Yes|No)(?:\s+\((.*)\))?$/,
          );

          if (!portMatch) continue;

          const portNumber = parseInt(portMatch[1]);
          const isConnected = portMatch[2] === "Yes";
          const circuitId = portMatch[3] || "";

          // Parse connection type from circuit ID
          let connectionType = "";
          if (circuitId) {
            if (circuitId.includes("Forward")) {
              connectionType = "Forward";
            } else if (circuitId.includes("Return")) {
              connectionType = "Return";
            } else if (circuitId.includes("A/B")) {
              connectionType = "A/B";
            } else if (circuitId.includes("A/B/C/D")) {
              connectionType = "A/B/C/D";
            } else if (circuitId.includes("BC Circuit")) {
              connectionType = "BC Circuit";
            } else if (circuitId.includes("Unknown")) {
              connectionType = "Unknown";
            }
          }

          // Only create records for connected ports
          if (isConnected) {
            const connection: FiberPanelConnection = {
              id: uuidv4(),
              siteLocation: panel.siteLocation,
              rackLocation: panel.rackLocation,
              panelName: panel.panelName,
              portNumber,
              isConnected,
              circuitId,
              connectionType,
            };

            fiberConnections.push(connection);
          }
        }
      }
    }

    console.log(`Extracted ${fiberConnections.length} fiber panel connections`);

    // Send fiber panel data to the ingest API
    console.log(`Sending ${fiberPanels.length} fiber panels to ingest API`);

    const panelIngestResult = await sendToIngestAPI("FiberPanel", fiberPanels);

    // Send fiber panel connection data to the ingest API
    console.log(
      `Sending ${fiberConnections.length} fiber panel connections to ingest API`,
    );

    const connectionBatchSize = 100;
    const connectionBatches: FiberPanelConnection[][] = [];

    for (let i = 0; i < fiberConnections.length; i += connectionBatchSize) {
      connectionBatches.push(
        fiberConnections.slice(i, i + connectionBatchSize),
      );
    }

    const connectionIngestResults: ConnectionIngestResult[] = [];

    for (let i = 0; i < connectionBatches.length; i++) {
      console.log(
        `Sending connection batch ${i + 1} of ${connectionBatches.length} (${connectionBatches[i].length} records)`,
      );

      try {
        const result = await sendToIngestAPI(
          "FiberPanelConnection",
          connectionBatches[i],
        );
        connectionIngestResults.push({
          batch: i + 1,
          success: true,
          records: connectionBatches[i].length,
          result,
        });
      } catch (error) {
        console.error(
          `Failed to send connection batch ${i + 1}: ${error instanceof Error ? error.message : String(error)}`,
        );
        connectionIngestResults.push({
          batch: i + 1,
          success: false,
          records: connectionBatches[i].length,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    // Return the processed data summary
    return {
      task: "ImportFiberPanelData",
      data: {
        success: true,
        message: `Imported ${fiberPanels.length} fiber panels and ${fiberConnections.length} connections with ${errors.length} errors`,
        panels: {
          count: fiberPanels.length,
          ingestResult: panelIngestResult,
        },
        connections: {
          count: fiberConnections.length,
          batches: connectionBatches.length,
          ingestResults: connectionIngestResults,
        },
        errors: errors.slice(0, 10),
        timestamp: new Date().toISOString(),
      },
    };
  } catch (error) {
    console.error(
      `Failed to import fiber panel data: ${error instanceof Error ? error.message : String(error)}`,
    );

    // Return error information
    return {
      task: "ImportFiberPanelData",
      data: {
        success: false,
        message: `Failed to import fiber panel data: ${error instanceof Error ? error.message : String(error)}`,
        panels: { count: 0 },
        connections: { count: 0 },
        errors: [error instanceof Error ? error.message : String(error)],
        timestamp: new Date().toISOString(),
      },
    };
  }
};

/**
 * Helper function to send data to the ingest API
 *
 * @param modelName - The name of the data model
 * @param data - The data to send
 * @returns The API response
 */
async function sendToIngestAPI<T>(modelName: string, data: T[]): Promise<any> {
  try {
    const response = await axios.post(
      `http://localhost:4000/ingest/${modelName}`,
      data,
      {
        headers: {
          "Content-Type": "application/json",
        },
      },
    );

    return response.data;
  } catch (error) {
    console.error(
      `Error sending to ingest API for ${modelName}: ${error instanceof Error ? error.message : String(error)}`,
    );
    throw error;
  }
}

/**
 * Creates and returns the task definition for the ImportFiberPanelData task
 */
export default function createTask(): TaskDefinition {
  return {
    task: importFiberPanelData,
    config: {
      retries: 3, // Retry up to 3 times if the task fails
    },
  };
}
