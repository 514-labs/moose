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

// Process each template and create manifest
const templates = fs
  .readdirSync(TEMPLATE_DIR)
  .filter((dir) => fs.statSync(path.join(TEMPLATE_DIR, dir)).isDirectory());

templates.forEach((template) => {
  const configPath = path.join(TEMPLATE_DIR, template, "template.config.toml");

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
      execFileSync("tar", ["-czf", outputFilePath, "."], { cwd });
    } catch (error) {
      console.error(`Error processing ${template}:`, error.message);
    }
  } else {
    console.warn(`Warning: No template.config.toml found in ${template}`);
  }
});

// Write manifest file to the packages directory
try {
  const manifestContent = toml.stringify(manifest);
  fs.writeFileSync(
    path.join(TEMPLATE_PACKAGES_DIR, "manifest.toml"),
    manifestContent,
  );
  console.log("Templates packaged successfully");
  console.log("Manifest created successfully");
  fs.readdirSync(TEMPLATE_PACKAGES_DIR).forEach((file) => {
    console.log(`- ${file}`);
  });
} catch (error) {
  console.error("Error writing manifest file:", error.message);
  process.exit(1);
}
