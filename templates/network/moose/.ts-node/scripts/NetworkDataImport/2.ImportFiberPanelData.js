"use strict";
var __createBinding =
  (this && this.__createBinding) ||
  (Object.create
    ? function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        var desc = Object.getOwnPropertyDescriptor(m, k);
        if (
          !desc ||
          ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)
        ) {
          desc = {
            enumerable: true,
            get: function () {
              return m[k];
            },
          };
        }
        Object.defineProperty(o, k2, desc);
      }
    : function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        o[k2] = m[k];
      });
var __setModuleDefault =
  (this && this.__setModuleDefault) ||
  (Object.create
    ? function (o, v) {
        Object.defineProperty(o, "default", { enumerable: true, value: v });
      }
    : function (o, v) {
        o["default"] = v;
      });
var __importStar =
  (this && this.__importStar) ||
  (function () {
    var ownKeys = function (o) {
      ownKeys =
        Object.getOwnPropertyNames ||
        function (o) {
          var ar = [];
          for (var k in o)
            if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
          return ar;
        };
      return ownKeys(o);
    };
    return function (mod) {
      if (mod && mod.__esModule) return mod;
      var result = {};
      if (mod != null)
        for (var k = ownKeys(mod), i = 0; i < k.length; i++)
          if (k[i] !== "default") __createBinding(result, mod, k[i]);
      __setModuleDefault(result, mod);
      return result;
    };
  })();
var __awaiter =
  (this && this.__awaiter) ||
  function (thisArg, _arguments, P, generator) {
    function adopt(value) {
      return value instanceof P
        ? value
        : new P(function (resolve) {
            resolve(value);
          });
    }
    return new (P || (P = Promise))(function (resolve, reject) {
      function fulfilled(value) {
        try {
          step(generator.next(value));
        } catch (e) {
          reject(e);
        }
      }
      function rejected(value) {
        try {
          step(generator["throw"](value));
        } catch (e) {
          reject(e);
        }
      }
      function step(result) {
        result.done
          ? resolve(result.value)
          : adopt(result.value).then(fulfilled, rejected);
      }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
  };
var __generator =
  (this && this.__generator) ||
  function (thisArg, body) {
    var _ = {
        label: 0,
        sent: function () {
          if (t[0] & 1) throw t[1];
          return t[1];
        },
        trys: [],
        ops: [],
      },
      f,
      y,
      t,
      g = Object.create(
        (typeof Iterator === "function" ? Iterator : Object).prototype,
      );
    return (
      (g.next = verb(0)),
      (g["throw"] = verb(1)),
      (g["return"] = verb(2)),
      typeof Symbol === "function" &&
        (g[Symbol.iterator] = function () {
          return this;
        }),
      g
    );
    function verb(n) {
      return function (v) {
        return step([n, v]);
      };
    }
    function step(op) {
      if (f) throw new TypeError("Generator is already executing.");
      while ((g && ((g = 0), op[0] && (_ = 0)), _))
        try {
          if (
            ((f = 1),
            y &&
              (t =
                op[0] & 2
                  ? y["return"]
                  : op[0]
                    ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                    : y.next) &&
              !(t = t.call(y, op[1])).done)
          )
            return t;
          if (((y = 0), t)) op = [op[0] & 2, t.value];
          switch (op[0]) {
            case 0:
            case 1:
              t = op;
              break;
            case 4:
              _.label++;
              return { value: op[1], done: false };
            case 5:
              _.label++;
              y = op[1];
              op = [0];
              continue;
            case 7:
              op = _.ops.pop();
              _.trys.pop();
              continue;
            default:
              if (
                !((t = _.trys), (t = t.length > 0 && t[t.length - 1])) &&
                (op[0] === 6 || op[0] === 2)
              ) {
                _ = 0;
                continue;
              }
              if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                _.label = op[1];
                break;
              }
              if (op[0] === 6 && _.label < t[1]) {
                _.label = t[1];
                t = op;
                break;
              }
              if (t && _.label < t[2]) {
                _.label = t[2];
                _.ops.push(op);
                break;
              }
              if (t[2]) _.ops.pop();
              _.trys.pop();
              continue;
          }
          op = body.call(thisArg, _);
        } catch (e) {
          op = [6, e];
          y = 0;
        } finally {
          f = t = 0;
        }
      if (op[0] & 5) throw op[1];
      return { value: op[0] ? op[1] : void 0, done: true };
    }
  };
var __importDefault =
  (this && this.__importDefault) ||
  function (mod) {
    return mod && mod.__esModule ? mod : { default: mod };
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = createTask;
var XLSX = __importStar(require("xlsx"));
var fs = __importStar(require("fs"));
var uuid_1 = require("uuid");
var axios_1 = __importDefault(require("axios"));
/**
 * Task to import fiber panel data from Excel file into the FiberPanel and FiberPanelConnection data models
 *
 * @param input - The input parameters for the task
 * @returns Object containing task name and imported data
 */
var importFiberPanelData = function (input) {
  return __awaiter(void 0, void 0, void 0, function () {
    var filePath,
      workbook,
      summarySheet,
      summaryData,
      siteLocationRow,
      siteLocation,
      headerRowIndex,
      fiberPanels,
      errors,
      i,
      row,
      rackLocation,
      panelName,
      connectedPorts,
      totalPorts,
      panel,
      fiberConnections,
      _i,
      fiberPanels_1,
      panel,
      panelSheet,
      panelData,
      rowIndex,
      row,
      colIndex,
      portInfo,
      portMatch,
      portNumber,
      isConnected,
      circuitId,
      connectionType,
      connection,
      panelIngestResult,
      connectionBatchSize,
      connectionBatches,
      i,
      connectionIngestResults,
      i,
      result,
      error_1,
      error_2;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          console.log("ImportFiberPanelData: Starting import task");
          _a.label = 1;
        case 1:
          _a.trys.push([1, 9, , 10]);
          filePath =
            input.filePath ||
            "./data/incur_site_osp_fiber_panel_report_02-21-2025_2119PM (1).xlsx";
          console.log(
            "Starting import of fiber panel data from ".concat(filePath),
          );
          // Check if file exists
          if (!fs.existsSync(filePath)) {
            throw new Error("File not found: ".concat(filePath));
          }
          workbook = XLSX.readFile(filePath);
          console.log(
            "Available sheets in the workbook: ".concat(
              workbook.SheetNames.join(", "),
            ),
          );
          // Process the Site Summary sheet to extract fiber panel information
          if (!workbook.SheetNames.includes("Site Summary (OSP)")) {
            throw new Error(
              "Site Summary (OSP) sheet not found in the workbook",
            );
          }
          summarySheet = workbook.Sheets["Site Summary (OSP)"];
          summaryData = XLSX.utils.sheet_to_json(summarySheet, { header: 1 });
          if (summaryData.length <= 1) {
            console.warn(
              "No data found in the Site Summary sheet or only headers present",
            );
            return [
              2 /*return*/,
              {
                task: "ImportFiberPanelData",
                data: {
                  success: false,
                  message: "No data found in the Site Summary sheet",
                  panels: [],
                  connections: [],
                },
              },
            ];
          }
          siteLocationRow =
            summaryData[0] && summaryData[0][0]
              ? String(summaryData[0][0])
              : "";
          siteLocation = siteLocationRow.split("/").slice(0, -1).join("/");
          console.log("Found site location: ".concat(siteLocation));
          headerRowIndex = summaryData.findIndex(function (row) {
            return (
              Array.isArray(row) &&
              row.length >= 2 &&
              row[0] === "Rack Location" &&
              row[1] === "Name"
            );
          });
          if (headerRowIndex === -1) {
            throw new Error("Could not find header row in Site Summary sheet");
          }
          fiberPanels = [];
          errors = [];
          // Start from the row after the headers
          for (i = headerRowIndex + 1; i < summaryData.length; i++) {
            row = summaryData[i];
            // Skip empty rows or rows without enough data
            if (!row || row.length < 4) continue;
            rackLocation = String(row[0] || "");
            panelName = String(row[1] || "");
            connectedPorts = Number(row[2] || 0);
            totalPorts = Number(row[3] || 0);
            // Skip if missing required data
            if (!rackLocation || !panelName) {
              errors.push(
                "Row ".concat(
                  i + 1,
                  ": Missing required fields (rackLocation or panelName)",
                ),
              );
              continue;
            }
            panel = {
              id: (0, uuid_1.v4)(),
              siteLocation: siteLocation,
              rackLocation: rackLocation,
              panelName: panelName,
              connectedPorts: connectedPorts,
              totalPorts: totalPorts,
            };
            fiberPanels.push(panel);
            console.log(
              "Processed panel: "
                .concat(panelName, " at ")
                .concat(rackLocation, " (")
                .concat(connectedPorts, "/")
                .concat(totalPorts, " ports connected)"),
            );
          }
          console.log("Extracted ".concat(fiberPanels.length, " fiber panels"));
          fiberConnections = [];
          for (
            _i = 0, fiberPanels_1 = fiberPanels;
            _i < fiberPanels_1.length;
            _i++
          ) {
            panel = fiberPanels_1[_i];
            // Look for a sheet named after the rack location
            if (!workbook.SheetNames.includes(panel.rackLocation)) {
              console.warn(
                "No sheet found for rack location ".concat(panel.rackLocation),
              );
              continue;
            }
            console.log(
              "Processing connections for panel "
                .concat(panel.panelName, " at ")
                .concat(panel.rackLocation),
            );
            panelSheet = workbook.Sheets[panel.rackLocation];
            panelData = XLSX.utils.sheet_to_json(panelSheet, { header: 1 });
            if (panelData.length <= 1) {
              console.warn(
                "No data found in the ".concat(panel.rackLocation, " sheet"),
              );
              continue;
            }
            // Process each row
            for (rowIndex = 2; rowIndex < panelData.length; rowIndex++) {
              row = panelData[rowIndex];
              // Skip empty rows
              if (!row || row.length === 0) continue;
              // Each column in the row represents a port
              for (colIndex = 0; colIndex < row.length; colIndex++) {
                portInfo = String(row[colIndex] || "");
                portMatch = portInfo.match(
                  /^(\d+):\s+(Yes|No)(?:\s+\((.*)\))?$/,
                );
                if (!portMatch) continue;
                portNumber = parseInt(portMatch[1]);
                isConnected = portMatch[2] === "Yes";
                circuitId = portMatch[3] || "";
                connectionType = "";
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
                  connection = {
                    id: (0, uuid_1.v4)(),
                    siteLocation: panel.siteLocation,
                    rackLocation: panel.rackLocation,
                    panelName: panel.panelName,
                    portNumber: portNumber,
                    isConnected: isConnected,
                    circuitId: circuitId,
                    connectionType: connectionType,
                  };
                  fiberConnections.push(connection);
                }
              }
            }
          }
          console.log(
            "Extracted ".concat(
              fiberConnections.length,
              " fiber panel connections",
            ),
          );
          // Send fiber panel data to the ingest API
          console.log(
            "Sending ".concat(
              fiberPanels.length,
              " fiber panels to ingest API",
            ),
          );
          return [4 /*yield*/, sendToIngestAPI("FiberPanel", fiberPanels)];
        case 2:
          panelIngestResult = _a.sent();
          // Send fiber panel connection data to the ingest API
          console.log(
            "Sending ".concat(
              fiberConnections.length,
              " fiber panel connections to ingest API",
            ),
          );
          connectionBatchSize = 100;
          connectionBatches = [];
          for (i = 0; i < fiberConnections.length; i += connectionBatchSize) {
            connectionBatches.push(
              fiberConnections.slice(i, i + connectionBatchSize),
            );
          }
          connectionIngestResults = [];
          i = 0;
          _a.label = 3;
        case 3:
          if (!(i < connectionBatches.length)) return [3 /*break*/, 8];
          console.log(
            "Sending connection batch "
              .concat(i + 1, " of ")
              .concat(connectionBatches.length, " (")
              .concat(connectionBatches[i].length, " records)"),
          );
          _a.label = 4;
        case 4:
          _a.trys.push([4, 6, , 7]);
          return [
            4 /*yield*/,
            sendToIngestAPI("FiberPanelConnection", connectionBatches[i]),
          ];
        case 5:
          result = _a.sent();
          connectionIngestResults.push({
            batch: i + 1,
            success: true,
            records: connectionBatches[i].length,
            result: result,
          });
          return [3 /*break*/, 7];
        case 6:
          error_1 = _a.sent();
          console.error(
            "Failed to send connection batch "
              .concat(i + 1, ": ")
              .concat(
                error_1 instanceof Error ? error_1.message : String(error_1),
              ),
          );
          connectionIngestResults.push({
            batch: i + 1,
            success: false,
            records: connectionBatches[i].length,
            error: error_1 instanceof Error ? error_1.message : String(error_1),
          });
          return [3 /*break*/, 7];
        case 7:
          i++;
          return [3 /*break*/, 3];
        case 8:
          // Return the processed data summary
          return [
            2 /*return*/,
            {
              task: "ImportFiberPanelData",
              data: {
                success: true,
                message: "Imported "
                  .concat(fiberPanels.length, " fiber panels and ")
                  .concat(fiberConnections.length, " connections with ")
                  .concat(errors.length, " errors"),
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
            },
          ];
        case 9:
          error_2 = _a.sent();
          console.error(
            "Failed to import fiber panel data: ".concat(
              error_2 instanceof Error ? error_2.message : String(error_2),
            ),
          );
          // Return error information
          return [
            2 /*return*/,
            {
              task: "ImportFiberPanelData",
              data: {
                success: false,
                message: "Failed to import fiber panel data: ".concat(
                  error_2 instanceof Error ? error_2.message : String(error_2),
                ),
                panels: { count: 0 },
                connections: { count: 0 },
                errors: [
                  error_2 instanceof Error ? error_2.message : String(error_2),
                ],
                timestamp: new Date().toISOString(),
              },
            },
          ];
        case 10:
          return [2 /*return*/];
      }
    });
  });
};
/**
 * Helper function to send data to the ingest API
 *
 * @param modelName - The name of the data model
 * @param data - The data to send
 * @returns The API response
 */
function sendToIngestAPI(modelName, data) {
  return __awaiter(this, void 0, void 0, function () {
    var response, error_3;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          _a.trys.push([0, 2, , 3]);
          return [
            4 /*yield*/,
            axios_1.default.post(
              "http://localhost:4000/ingest/".concat(modelName),
              data,
              {
                headers: {
                  "Content-Type": "application/json",
                },
              },
            ),
          ];
        case 1:
          response = _a.sent();
          return [2 /*return*/, response.data];
        case 2:
          error_3 = _a.sent();
          console.error(
            "Error sending to ingest API for "
              .concat(modelName, ": ")
              .concat(
                error_3 instanceof Error ? error_3.message : String(error_3),
              ),
          );
          throw error_3;
        case 3:
          return [2 /*return*/];
      }
    });
  });
}
/**
 * Creates and returns the task definition for the ImportFiberPanelData task
 */
function createTask() {
  return {
    task: importFiberPanelData,
    config: {
      retries: 3, // Retry up to 3 times if the task fails
    },
  };
}
//# sourceMappingURL=2.ImportFiberPanelData.js.map
