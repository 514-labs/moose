#! /usr/bin/env node

// The goal if this script is to get all the templates from the templates folder
// and package them in tar files to be uploaded to Google Cloud Storage

const fs = require("fs");
const process = require("child_process");

const TEMPLATE_PACKAGES_DIR = "template-packages";
const TEMPLATE_DIR = "templates";

fs.mkdirSync(TEMPLATE_PACKAGES_DIR, { recursive: true });

const templates = fs.readdirSync(TEMPLATE_DIR);
templates.forEach((template) => {
  // We tar the template with native tools
  const cwd = `${__dirname}/../${TEMPLATE_DIR}/${template}`;
  process.execSync(
    `tar -czf ../../${TEMPLATE_PACKAGES_DIR}/${template}.tgz .`,
    {
      cwd,
    }
  );
});

console.log("Templates packaged successfully");
fs.readdirSync(TEMPLATE_PACKAGES_DIR).forEach((file) => {
  console.log(`- ${file}`);
});
