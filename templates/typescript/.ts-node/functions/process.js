"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var models_1 = require("../ingest/models");
models_1.FooPipeline.stream.addTransform(
  models_1.BarPipeline.stream,
  function (foo) {
    var _a, _b;
    return {
      primaryKey: foo.primaryKey,
      utcTimestamp: new Date(foo.timestamp),
      textLength:
        (_b =
          (_a = foo.optionalText) === null || _a === void 0
            ? void 0
            : _a.length) !== null && _b !== void 0
          ? _b
          : 0,
      hasText: foo.optionalText !== null,
    };
  },
);
//# sourceMappingURL=process.js.map
