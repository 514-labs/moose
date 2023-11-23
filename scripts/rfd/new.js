#! /usr/bin/env node

if (process.argv.length !== 3) {
  console.error("Expected only one argument!");
  process.exit(1);
}

const fs = require("fs");
const { execSync } = require("child_process");

// We check that we are currently on the main branch.
// We are not automatically switching to the main branch because
// we don't want to mess with the user's current branch
const currentBranch = execSync("git branch --show-current").toString().trim();

if (currentBranch !== "main") {
  console.error(
    "You are not on the main branch, please checkout the main branch before continuing."
  );
  process.exit(1);
}

const title = process.argv[2];

// We get the RFD number from the number of RFDs that already exist
const rfdNumber = (
  "" +
  (fs.readdirSync(__dirname + "/../../rfd").length + 1)
).padStart(4, "0");

// We check that there is not already a branch with the RFD number that
// has not been merged into main yet
const branches = execSync(`git branch -al "*rfd/${rfdNumber}"`)
  .toString()
  .trim();

if (branches.length > 0) {
  console.error(
    `The branch 'rfd/${rfdNumber}' already exist, please contact other maintainers to merge it into main before continuing.`
  );
  process.exit(1);
}

// We get author information from git config
const author = execSync("git config user.name").toString().trim();
if (!author) {
  console.error(
    "No Git author name configured!, please configure git config user.name"
  );
  process.exit(1);
}

const authorEmail = execSync("git config user.email").toString().trim();
if (!authorEmail) {
  console.error(
    "No Git author email configured!, please configure git config user.email"
  );
  process.exit(1);
}

// We create the branch
execSync(`git checkout -b "rfd/${rfdNumber}"`);

// We read the template and hydrate it with the RFD number, title and author
const template = fs.readFileSync(__dirname + "/template.mdx", "utf8");
const hydratedTemplate = template
  .replace("{{title}}", title)
  .replace("{{authors}}", `${author} <${authorEmail}>`);

// We create the RFD directory and write the hydrated template to it
fs.mkdirSync(__dirname + `/../../rfd/${rfdNumber}`);
fs.writeFileSync(
  __dirname + `/../../rfd/${rfdNumber}/README.mdx`,
  hydratedTemplate
);
