const xlsx = require("xlsx");
const path = require("path");
const fs = require("fs");

function describeExcel(filePath) {
  try {
    console.log(`Analyzing file: ${filePath}`);
    const workbook = xlsx.readFile(filePath);
    const result = {};

    workbook.SheetNames.forEach((sheetName) => {
      const worksheet = workbook.Sheets[sheetName];
      const jsonData = xlsx.utils.sheet_to_json(worksheet, { header: 1 });

      // Get column headers (first row)
      const headers = jsonData[0] || [];

      // Calculate stats
      const rowCount = jsonData.length - 1; // Excluding header row

      result[sheetName] = {
        headers,
        rowCount,
        sampleData: jsonData.slice(1, Math.min(4, jsonData.length)),
      };
    });

    return result;
  } catch (error) {
    return { error: error.message, stack: error.stack };
  }
}

// Define file paths
const dataDir = path.join(__dirname, "..", "data");
const files = [
  path.join(dataDir, "Transport_Schedule(Transport) (1).xlsx"),
  path.join(
    dataDir,
    "incur_site_osp_fiber_panel_report_02-21-2025_2119PM (1).xlsx",
  ),
];

files.forEach((file) => {
  console.log(`\n==== File: ${path.basename(file)} ====`);
  const data = describeExcel(file);
  console.log(JSON.stringify(data, null, 2));
});
