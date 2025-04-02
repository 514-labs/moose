#! /usr/bin/env node

// The goal if this script is to get all the templates from the templates folder
// and package them in tar files to be uploaded to Google Cloud Storage

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const toml = require("@iarna/toml");

const TEMPLATE_PACKAGES_DIR = "template-packages";
const TEMPLATE_DIR = "templates";

// Create packages directory
fs.mkdirSync(TEMPLATE_PACKAGES_DIR, { recursive: true });

// First create the manifest
const manifest = {
  templates: {},
};

// Log base directories
console.log(
  "Template Packages Directory:",
  path.resolve(TEMPLATE_PACKAGES_DIR),
);
console.log("Templates Directory:", path.resolve(TEMPLATE_DIR));

// Process each template and create manifest
const templates = fs
  .readdirSync(TEMPLATE_DIR)
  .filter((dir) => fs.statSync(path.join(TEMPLATE_DIR, dir)).isDirectory());

console.log("\nFound templates:", templates);

templates.forEach((template) => {
  const configPath = path.join(TEMPLATE_DIR, template, "template.config.toml");
  console.log(`\nProcessing template: ${template}`);
  console.log("Config path:", path.resolve(configPath));

  if (fs.existsSync(configPath)) {
    try {
      const configContent = fs.readFileSync(configPath, "utf8");
      const config = toml.parse(configContent);

      // Add all fields from the config directly
      manifest.templates[template] = {
        ...config,
      };

      console.log(
        `Template ${template} fields:`,
        Object.keys(config).join(", "),
      );

      // Package the template
      const cwd = path.join(__dirname, "..", TEMPLATE_DIR, template);
      const outputFilePath = path.join(
        __dirname,
        "..",
        TEMPLATE_PACKAGES_DIR,
        `${template}.tgz`,
      );
      console.log("Template directory:", path.resolve(cwd));
      console.log("Output tar file:", path.resolve(outputFilePath));

      execFileSync(
        "tar",
        ["-czf", outputFilePath, "--exclude", "node_modules", "."],
        {
          cwd,
        },
      );
      console.log(`Successfully created tar file for ${template}`);
    } catch (error) {
      console.error(`Error processing ${template}:`, error.message);
    }
  } else {
    console.warn(`Warning: No template.config.toml found in ${template}`);
  }
});

// Write manifest file to the packages directory
try {
  const manifestPath = path.join(TEMPLATE_PACKAGES_DIR, "manifest.toml");
  console.log("\nWriting manifest to:", path.resolve(manifestPath));
  const manifestContent = toml.stringify(manifest);
  fs.writeFileSync(manifestPath, manifestContent);

  console.log("\nTemplates packaged successfully");
  console.log("Files in packages directory:");
  fs.readdirSync(TEMPLATE_PACKAGES_DIR).forEach((file) => {
    const fullPath = path.resolve(TEMPLATE_PACKAGES_DIR, file);
    console.log(`- ${file} (${fullPath})`);
  });
} catch (error) {
  console.error("Error writing manifest file:", error.message);
  process.exit(1);
}
