"use strict";
var __importDefault =
  (this && this.__importDefault) ||
  function (mod) {
    return mod && mod.__esModule ? mod : { default: mod };
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.AircraftTrackingProcessedPipeline =
  exports.AircraftTrackingDataPipeline = void 0;
var typia_1 = __importDefault(require("typia"));
var moose_lib_1 = require("@514labs/moose-lib");
exports.AircraftTrackingDataPipeline = new moose_lib_1.IngestPipeline(
  "AircraftTrackingData",
  {
    table: false,
    stream: true,
    ingest: true,
  },
  {
    version: "3.1",
    components: {
      schemas: {
        AircraftTrackingData: {
          type: "object",
          properties: {
            hex: {
              type: "string",
            },
            transponder_type: {
              type: "string",
            },
            flight: {
              type: "string",
            },
            r: {
              type: "string",
            },
            aircraft_type: {
              type: "string",
            },
            dbFlags: {
              type: "number",
            },
            lat: {
              type: "number",
            },
            lon: {
              type: "number",
            },
            alt_baro: {
              type: "number",
            },
            alt_baro_is_ground: {
              type: "boolean",
            },
            alt_geom: {
              type: "number",
            },
            gs: {
              type: "number",
            },
            track: {
              type: "number",
            },
            baro_rate: {
              type: "number",
            },
            geom_rate: {
              type: "number",
            },
            squawk: {
              type: "string",
            },
            emergency: {
              type: "string",
            },
            category: {
              type: "string",
            },
            nav_qnh: {
              type: "number",
            },
            nav_altitude_mcp: {
              type: "number",
            },
            nav_heading: {
              type: "number",
            },
            nav_modes: {
              type: "array",
              items: {
                type: "string",
              },
            },
            nic: {
              type: "number",
            },
            rc: {
              type: "number",
            },
            seen_pos: {
              type: "number",
            },
            version: {
              type: "number",
            },
            nic_baro: {
              type: "number",
            },
            nac_p: {
              type: "number",
            },
            nac_v: {
              type: "number",
            },
            sil: {
              type: "number",
            },
            sil_type: {
              type: "string",
            },
            gva: {
              type: "number",
            },
            sda: {
              type: "number",
            },
            alert: {
              type: "number",
            },
            spi: {
              type: "number",
            },
            mlat: {
              type: "array",
              items: {
                type: "string",
              },
            },
            tisb: {
              type: "array",
              items: {
                type: "string",
              },
            },
            messages: {
              type: "number",
            },
            seen: {
              type: "number",
            },
            rssi: {
              type: "number",
            },
            timestamp: {
              type: "string",
              format: "date-time",
            },
          },
          required: [
            "hex",
            "transponder_type",
            "flight",
            "r",
            "dbFlags",
            "lat",
            "lon",
            "alt_baro",
            "alt_baro_is_ground",
            "alt_geom",
            "gs",
            "track",
            "baro_rate",
            "squawk",
            "emergency",
            "category",
            "nic",
            "rc",
            "seen_pos",
            "version",
            "nic_baro",
            "nac_p",
            "nac_v",
            "sil",
            "sil_type",
            "gva",
            "sda",
            "alert",
            "spi",
            "mlat",
            "tisb",
            "messages",
            "seen",
            "rssi",
            "timestamp",
          ],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/AircraftTrackingData",
      },
    ],
  },
  JSON.parse(
    '[{"name":"hex","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"transponder_type","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"flight","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"r","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"aircraft_type","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"dbFlags","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"lat","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"lon","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alt_baro","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alt_baro_is_ground","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alt_geom","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"gs","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"track","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"baro_rate","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"geom_rate","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"squawk","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"emergency","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"category","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nav_qnh","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nav_altitude_mcp","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nav_heading","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nav_modes","data_type":{"elementNullable":false,"elementType":"String"},"primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nic","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"rc","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"seen_pos","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"version","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nic_baro","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nac_p","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nac_v","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"sil","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"sil_type","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"gva","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"sda","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alert","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"spi","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"mlat","data_type":{"elementNullable":false,"elementType":"String"},"primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"tisb","data_type":{"elementNullable":false,"elementType":"String"},"primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"messages","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"seen","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"rssi","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"timestamp","data_type":"DateTime","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]}]',
  ),
);
exports.AircraftTrackingProcessedPipeline = new moose_lib_1.IngestPipeline(
  "AircraftTrackingProcessed",
  {
    table: true,
    stream: true,
    ingest: false,
  },
  {
    version: "3.1",
    components: {
      schemas: {
        AircraftTrackingProcessed: {
          type: "object",
          properties: {
            zorderCoordinate: {
              type: "number",
            },
            approach: {
              type: "boolean",
            },
            autopilot: {
              type: "boolean",
            },
            althold: {
              type: "boolean",
            },
            lnav: {
              type: "boolean",
            },
            tcas: {
              type: "boolean",
            },
            hex: {
              type: "string",
            },
            transponder_type: {
              type: "string",
            },
            flight: {
              type: "string",
            },
            r: {
              type: "string",
            },
            aircraft_type: {
              type: "string",
            },
            dbFlags: {
              type: "number",
            },
            lat: {
              type: "number",
            },
            lon: {
              type: "number",
            },
            alt_baro: {
              type: "number",
            },
            alt_baro_is_ground: {
              type: "boolean",
            },
            alt_geom: {
              type: "number",
            },
            gs: {
              type: "number",
            },
            track: {
              type: "number",
            },
            baro_rate: {
              type: "number",
            },
            geom_rate: {
              type: "number",
            },
            squawk: {
              type: "string",
            },
            emergency: {
              type: "string",
            },
            category: {
              type: "string",
            },
            nav_qnh: {
              type: "number",
            },
            nav_altitude_mcp: {
              type: "number",
            },
            nav_heading: {
              type: "number",
            },
            nav_modes: {
              type: "array",
              items: {
                type: "string",
              },
            },
            nic: {
              type: "number",
            },
            rc: {
              type: "number",
            },
            seen_pos: {
              type: "number",
            },
            version: {
              type: "number",
            },
            nic_baro: {
              type: "number",
            },
            nac_p: {
              type: "number",
            },
            nac_v: {
              type: "number",
            },
            sil: {
              type: "number",
            },
            sil_type: {
              type: "string",
            },
            gva: {
              type: "number",
            },
            sda: {
              type: "number",
            },
            alert: {
              type: "number",
            },
            spi: {
              type: "number",
            },
            mlat: {
              type: "array",
              items: {
                type: "string",
              },
            },
            tisb: {
              type: "array",
              items: {
                type: "string",
              },
            },
            messages: {
              type: "number",
            },
            seen: {
              type: "number",
            },
            rssi: {
              type: "number",
            },
            timestamp: {
              type: "string",
              format: "date-time",
            },
          },
          required: [
            "zorderCoordinate",
            "approach",
            "autopilot",
            "althold",
            "lnav",
            "tcas",
            "hex",
            "transponder_type",
            "flight",
            "r",
            "dbFlags",
            "lat",
            "lon",
            "alt_baro",
            "alt_baro_is_ground",
            "alt_geom",
            "gs",
            "track",
            "baro_rate",
            "squawk",
            "emergency",
            "category",
            "nic",
            "rc",
            "seen_pos",
            "version",
            "nic_baro",
            "nac_p",
            "nac_v",
            "sil",
            "sil_type",
            "gva",
            "sda",
            "alert",
            "spi",
            "mlat",
            "tisb",
            "messages",
            "seen",
            "rssi",
            "timestamp",
          ],
        },
      },
    },
    schemas: [
      {
        $ref: "#/components/schemas/AircraftTrackingProcessed",
      },
    ],
  },
  JSON.parse(
    '[{"name":"zorderCoordinate","data_type":"Float","primary_key":true,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"approach","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"autopilot","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"althold","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"lnav","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"tcas","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"hex","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"transponder_type","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"flight","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"r","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"aircraft_type","data_type":"String","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"dbFlags","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"lat","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"lon","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alt_baro","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alt_baro_is_ground","data_type":"Boolean","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alt_geom","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"gs","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"track","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"baro_rate","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"geom_rate","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"squawk","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"emergency","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"category","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nav_qnh","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nav_altitude_mcp","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nav_heading","data_type":"Float","primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nav_modes","data_type":{"elementNullable":false,"elementType":"String"},"primary_key":false,"required":false,"unique":false,"default":null,"annotations":[]},{"name":"nic","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"rc","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"seen_pos","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"version","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nic_baro","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nac_p","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"nac_v","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"sil","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"sil_type","data_type":"String","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"gva","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"sda","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"alert","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"spi","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"mlat","data_type":{"elementNullable":false,"elementType":"String"},"primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"tisb","data_type":{"elementNullable":false,"elementType":"String"},"primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"messages","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"seen","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"rssi","data_type":"Float","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]},{"name":"timestamp","data_type":"DateTime","primary_key":false,"required":true,"unique":false,"default":null,"annotations":[]}]',
  ),
);
//# sourceMappingURL=models.js.map
