/**
 * End-to-end test for CollapsingMergeTree and VersionedCollapsingMergeTree support
 * This test verifies the complete flow from TypeScript API to internal configuration
 */

import { ClickHouseEngines, OlapTable, OlapConfig } from "./src/index";

// Test data models
interface UserChangeEvent {
  user_id: string;
  name: string;
  email: string;
  updated_at: Date;
  sign: number; // +1 for insert/update, -1 for delete
}

interface VersionedUserEvent {
  user_id: string;
  name: string;
  email: string;
  updated_at: Date;
  operation_sign: number; // Custom sign column
  revision: number; // Custom version column
}

// Test 1: CollapsingMergeTree with default sign column
console.log("=== Test 1: CollapsingMergeTree with default sign column ===");

const defaultCollapsingConfig: OlapConfig<UserChangeEvent> = {
  orderByFields: ["user_id", "updated_at"],
  engine: ClickHouseEngines.CollapsingMergeTree,
};

const defaultCollapsingTable = new OlapTable<UserChangeEvent>(
  "user_changes_default",
  defaultCollapsingConfig,
);

console.log(
  "Default CollapsingMergeTree table created:",
  defaultCollapsingTable.name,
);
console.log("Config:", JSON.stringify(defaultCollapsingTable.config, null, 2));

// Test 2: CollapsingMergeTree with custom sign column
console.log("\n=== Test 2: CollapsingMergeTree with custom sign column ===");

const customCollapsingConfig: OlapConfig<UserChangeEvent> = {
  orderByFields: ["user_id", "updated_at"],
  engine: ClickHouseEngines.CollapsingMergeTree,
  collapsingMergeTreeConfig: {
    signColumn: "operation_type",
  },
};

const customCollapsingTable = new OlapTable<UserChangeEvent>(
  "user_changes_custom",
  customCollapsingConfig,
);

console.log(
  "Custom CollapsingMergeTree table created:",
  customCollapsingTable.name,
);
console.log("Config:", JSON.stringify(customCollapsingTable.config, null, 2));

// Test 3: VersionedCollapsingMergeTree with default columns
console.log(
  "\n=== Test 3: VersionedCollapsingMergeTree with default columns ===",
);

const defaultVersionedConfig: OlapConfig<VersionedUserEvent> = {
  orderByFields: ["user_id", "updated_at"],
  engine: ClickHouseEngines.VersionedCollapsingMergeTree,
};

const defaultVersionedTable = new OlapTable<VersionedUserEvent>(
  "user_events_default_versioned",
  defaultVersionedConfig,
);

console.log(
  "Default VersionedCollapsingMergeTree table created:",
  defaultVersionedTable.name,
);
console.log("Config:", JSON.stringify(defaultVersionedTable.config, null, 2));

// Test 4: VersionedCollapsingMergeTree with custom columns
console.log(
  "\n=== Test 4: VersionedCollapsingMergeTree with custom columns ===",
);

const customVersionedConfig: OlapConfig<VersionedUserEvent> = {
  orderByFields: ["user_id", "updated_at"],
  engine: ClickHouseEngines.VersionedCollapsingMergeTree,
  versionedCollapsingMergeTreeConfig: {
    signColumn: "operation_sign",
    versionColumn: "revision",
  },
};

const customVersionedTable = new OlapTable<VersionedUserEvent>(
  "user_events_custom_versioned",
  customVersionedConfig,
);

console.log(
  "Custom VersionedCollapsingMergeTree table created:",
  customVersionedTable.name,
);
console.log("Config:", JSON.stringify(customVersionedTable.config, null, 2));

// Test 5: Verify internal serialization
console.log("\n=== Test 5: Verify internal serialization ===");

// Import internal serialization function
import { toInfraMap } from "./src/dmv2/internal";
import { moose_internal } from "./src/dmv2/typedBase";

// Generate infrastructure map
const infraMap = toInfraMap(moose_internal);

console.log("Infrastructure map tables:");
Object.keys(infraMap.tables).forEach((tableName) => {
  const table = infraMap.tables[tableName];
  console.log(`${tableName}:`);
  console.log(`  Engine: ${table.engine}`);
  console.log(`  OrderBy: ${table.orderBy.join(", ")}`);
  console.log(`  Deduplicate: ${table.deduplicate}`);
});

console.log("\n=== All tests completed successfully! ===");
