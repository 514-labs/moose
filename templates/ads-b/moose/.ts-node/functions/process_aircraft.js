"use strict";
var __assign =
  (this && this.__assign) ||
  function () {
    __assign =
      Object.assign ||
      function (t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
          s = arguments[i];
          for (var p in s)
            if (Object.prototype.hasOwnProperty.call(s, p)) t[p] = s[p];
        }
        return t;
      };
    return __assign.apply(this, arguments);
  };
Object.defineProperty(exports, "__esModule", { value: true });
var models_1 = require("../datamodels/models");
function calculateZOrder(lat, lon) {
  // Normalize lat/lon to integers between 0 and 2^20
  var latInt = Math.floor(((lat + 90.0) * (1 << 20)) / 180.0);
  var lonInt = Math.floor(((lon + 180.0) * (1 << 20)) / 360.0);
  // Interleave bits
  var result = 0;
  for (var i = 0; i < 20; i++) {
    result |= ((latInt & (1 << i)) << i) | ((lonInt & (1 << i)) << (i + 1));
  }
  return result;
}
/**
 * Converts NavModes array to boolean flags
 * @param navModes Array of navigation modes
 * @returns Object containing boolean flags for each nav mode
 */
function parseNavModes(navModes) {
  var _a, _b, _c, _d, _e;
  return {
    approach:
      (_a =
        navModes === null || navModes === void 0
          ? void 0
          : navModes.includes("approach")) !== null && _a !== void 0
        ? _a
        : false,
    autopilot:
      (_b =
        navModes === null || navModes === void 0
          ? void 0
          : navModes.includes("autopilot")) !== null && _b !== void 0
        ? _b
        : false,
    althold:
      (_c =
        navModes === null || navModes === void 0
          ? void 0
          : navModes.includes("althold")) !== null && _c !== void 0
        ? _c
        : false,
    lnav:
      (_d =
        navModes === null || navModes === void 0
          ? void 0
          : navModes.includes("lnav")) !== null && _d !== void 0
        ? _d
        : false,
    tcas:
      (_e =
        navModes === null || navModes === void 0
          ? void 0
          : navModes.includes("tcas")) !== null && _e !== void 0
        ? _e
        : false,
  };
}
if (
  models_1.AircraftTrackingDataPipeline.stream &&
  models_1.AircraftTrackingProcessedPipeline.stream
) {
  models_1.AircraftTrackingDataPipeline.stream.addTransform(
    models_1.AircraftTrackingProcessedPipeline.stream,
    function (record) {
      var zorderCoordinate = calculateZOrder(record.lat, record.lon);
      var _a = parseNavModes(record.nav_modes),
        approach = _a.approach,
        autopilot = _a.autopilot,
        althold = _a.althold,
        lnav = _a.lnav,
        tcas = _a.tcas;
      return __assign(__assign({}, record), {
        zorderCoordinate: zorderCoordinate,
        approach: approach,
        autopilot: autopilot,
        althold: althold,
        lnav: lnav,
        tcas: tcas,
      });
    },
  );
}
//# sourceMappingURL=process_aircraft.js.map
