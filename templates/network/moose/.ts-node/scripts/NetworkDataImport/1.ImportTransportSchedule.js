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
 * Task to import transport schedule data from Excel file into the TransportSchedule data model
 *
 * @param input - The input parameters for the task
 * @returns Object containing task name and imported data
 */
var importTransportSchedule = function (input) {
  return __awaiter(void 0, void 0, void 0, function () {
    var filePath,
      sheetName,
      workbook,
      worksheet,
      rawData,
      headers,
      transportScheduleData,
      errors,
      fieldMapping_1,
      columnMap_1,
      _loop_1,
      i,
      batchSize,
      batches,
      i,
      ingestResults,
      i,
      response,
      error_1,
      error_2;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          console.log("ImportTransportSchedule: Starting import task");
          _a.label = 1;
        case 1:
          _a.trys.push([1, 8, , 9]);
          filePath =
            input.filePath || "./data/Transport_Schedule(Transport) (1).xlsx";
          sheetName = input.sheetName || "Sheet 1 - Transport_Schedule(Tr";
          console.log(
            "Starting import of transport schedule from "
              .concat(filePath, ", sheet: ")
              .concat(sheetName),
          );
          // Check if file exists
          if (!fs.existsSync(filePath)) {
            throw new Error("File not found: ".concat(filePath));
          }
          workbook = XLSX.readFile(filePath);
          // Check if the specified sheet exists
          if (!workbook.SheetNames.includes(sheetName)) {
            throw new Error(
              "Sheet '"
                .concat(
                  sheetName,
                  "' not found in workbook. Available sheets: ",
                )
                .concat(workbook.SheetNames.join(", ")),
            );
          }
          worksheet = workbook.Sheets[sheetName];
          rawData = XLSX.utils.sheet_to_json(worksheet, { header: 1 });
          if (rawData.length <= 1) {
            console.warn(
              "No data found in the Excel sheet or only headers present",
            );
            return [
              2 /*return*/,
              {
                task: "ImportTransportSchedule",
                data: {
                  success: false,
                  message:
                    "No data found in the Excel sheet or only headers present",
                  records: [],
                },
              },
            ];
          }
          headers = rawData[0];
          console.log(
            "Found ".concat(rawData.length - 1, " records in the Excel sheet"),
          );
          transportScheduleData = [];
          errors = [];
          fieldMapping_1 = {
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
          columnMap_1 = {};
          headers.forEach(function (header, index) {
            if (fieldMapping_1[header]) {
              columnMap_1[index] = fieldMapping_1[header];
            }
          });
          _loop_1 = function (i) {
            try {
              var row_1 = rawData[i];
              // Skip empty rows
              if (!row_1 || row_1.length === 0) return "continue";
              // Create a transport schedule item
              var item_1 = {
                id: (0, uuid_1.v4)(), // Generate a unique ID
              };
              // Map data from Excel columns to our data model fields
              Object.entries(columnMap_1).forEach(function (_a) {
                var columnIndex = _a[0],
                  fieldName = _a[1];
                var value = row_1[parseInt(columnIndex)];
                // Skip undefined or null values
                if (value === undefined || value === null) return;
                // Handle different field types
                switch (fieldName) {
                  case "ispReadyDate":
                  case "fieldOpsDate":
                    // Handle Excel date values
                    if (typeof value === "string") {
                      try {
                        item_1[fieldName] = new Date(value);
                      } catch (e) {
                        // Skip if we can't parse the date
                        console.warn(
                          "Could not parse date value: ".concat(value),
                        );
                      }
                    } else if (typeof value === "number") {
                      // Excel stores dates as days since 1/1/1900
                      var excelEpoch = new Date(1899, 11, 30);
                      var date = new Date(excelEpoch);
                      date.setDate(excelEpoch.getDate() + value);
                      item_1[fieldName] = date;
                    }
                    break;
                  case "priority":
                  case "distanceKM":
                    item_1[fieldName] =
                      typeof value === "number" ? value : parseFloat(value);
                    break;
                  default:
                    item_1[fieldName] = String(value);
                }
              });
              // Validate the transformed item - add minimal validation to ensure it has required fields
              if (
                !item_1.transportRingName ||
                !item_1.hubAName ||
                !item_1.hubBName
              ) {
                errors.push(
                  "Row ".concat(
                    i + 1,
                    ": Missing required fields (transportRingName, hubAName, or hubBName)",
                  ),
                );
                return "continue";
              }
              transportScheduleData.push(item_1);
            } catch (error) {
              errors.push(
                "Error processing row "
                  .concat(i + 1, ": ")
                  .concat(
                    error instanceof Error ? error.message : String(error),
                  ),
              );
            }
          };
          // Process each data row
          for (i = 1; i < rawData.length; i++) {
            _loop_1(i);
          }
          // Log any errors
          if (errors.length > 0) {
            console.warn(
              "Encountered ".concat(errors.length, " errors during import:"),
            );
            errors.slice(0, 5).forEach(function (error) {
              return console.warn(error);
            });
            if (errors.length > 5) {
              console.warn(
                "... and ".concat(errors.length - 5, " more errors"),
              );
            }
          }
          console.log(
            "Successfully processed ".concat(
              transportScheduleData.length,
              " transport schedule records",
            ),
          );
          batchSize = 100;
          batches = [];
          for (i = 0; i < transportScheduleData.length; i += batchSize) {
            batches.push(transportScheduleData.slice(i, i + batchSize));
          }
          console.log(
            "Sending data in "
              .concat(batches.length, " batches of up to ")
              .concat(batchSize, " records each"),
          );
          ingestResults = [];
          i = 0;
          _a.label = 2;
        case 2:
          if (!(i < batches.length)) return [3 /*break*/, 7];
          _a.label = 3;
        case 3:
          _a.trys.push([3, 5, , 6]);
          console.log(
            "Sending batch "
              .concat(i + 1, " of ")
              .concat(batches.length, " (")
              .concat(batches[i].length, " records)"),
          );
          return [
            4 /*yield*/,
            axios_1.default.post(
              "http://localhost:4000/ingest/TransportSchedule",
              batches[i],
              {
                headers: {
                  "Content-Type": "application/json",
                },
              },
            ),
          ];
        case 4:
          response = _a.sent();
          ingestResults.push({
            batch: i + 1,
            success: true,
            records: batches[i].length,
            response: response.data,
          });
          console.log(
            "Successfully sent batch ".concat(i + 1, " to ingest API"),
          );
          return [3 /*break*/, 6];
        case 5:
          error_1 = _a.sent();
          console.error(
            "Failed to send batch "
              .concat(i + 1, " to ingest API: ")
              .concat(
                error_1 instanceof Error ? error_1.message : String(error_1),
              ),
          );
          ingestResults.push({
            batch: i + 1,
            success: false,
            records: batches[i].length,
            error: error_1 instanceof Error ? error_1.message : String(error_1),
          });
          return [3 /*break*/, 6];
        case 6:
          i++;
          return [3 /*break*/, 2];
        case 7:
          // Return the processed data summary
          return [
            2 /*return*/,
            {
              task: "ImportTransportSchedule",
              data: {
                success: true,
                message: "Imported "
                  .concat(
                    transportScheduleData.length,
                    " transport schedule records with ",
                  )
                  .concat(errors.length, " errors"),
                totalRecords: transportScheduleData.length,
                errorCount: errors.length,
                sampleErrors: errors.slice(0, 5),
                batches: batches.length,
                ingestResults: ingestResults,
                timestamp: new Date().toISOString(),
              },
            },
          ];
        case 8:
          error_2 = _a.sent();
          console.error(
            "Failed to import transport schedule: ".concat(
              error_2 instanceof Error ? error_2.message : String(error_2),
            ),
          );
          // Return error information
          return [
            2 /*return*/,
            {
              task: "ImportTransportSchedule",
              data: {
                success: false,
                message: "Failed to import transport schedule: ".concat(
                  error_2 instanceof Error ? error_2.message : String(error_2),
                ),
                records: 0,
                errors: [
                  error_2 instanceof Error ? error_2.message : String(error_2),
                ],
                timestamp: new Date().toISOString(),
              },
            },
          ];
        case 9:
          return [2 /*return*/];
      }
    });
  });
};
/**
 * Creates and returns the task definition for the ImportTransportSchedule task
 */
function createTask() {
  return {
    task: importTransportSchedule,
    config: {
      retries: 3, // Retry up to 3 times if the task fails
    },
  };
}
//# sourceMappingURL=1.ImportTransportSchedule.js.map
