#! /usr/bin/env node

// The Goal of this script is to generate the version number for the current commit

// It should only be run in the context of github actions
// some of the variables are set by github actions

const { execSync } = require("child_process");

if (process.argv.length !== 3) {
  console.error("Expected only one argument!");
  process.exit(1);
}

const commit = process.argv[2];
const NO_RELEASE_COMMIT_MESSAGE = "[no-release]";
const MAJOR_COMMIT_MESSAGE = "[major-release]";
const MINOR_COMMIT_MESSAGE = "[minor-release]";

const printVersion = (major, minor, patch) => {
  console.log(`VERSION=${major}.${minor}.${patch}`);
};
// We check that the current commit is not tagged yet. If it is,
// we return its value as the version
execSync("git fetch --tags");
const tags = execSync(`git tag --points-at ${commit}`).toString().trim();

if (tags.length > 0 && tags.startsWith("v")) {
  console.log(`VERSION=${tags.slice(1)}`);
  process.exit(0);
}

// We retrieve the last release tag.
const latestTag = execSync("git describe --tags --abbrev=0").toString().trim();
// We parse the version number from the tag
const version = latestTag.match(/v(\d+)\.(\d+)\.(\d+)/);

const major = parseInt(version[1]);
const minor = parseInt(version[2]);
const patch = parseInt(version[3]);

const commitMessage = execSync(`git log -1 --pretty=%B ${commit}`)
  .toString()
  .trim();

if (commitMessage.includes(NO_RELEASE_COMMIT_MESSAGE)) {
  printVersion(major, minor, patch);
  process.exit(0);
}

if (commitMessage.includes(MAJOR_COMMIT_MESSAGE)) {
  printVersion(major + 1, 0, 0);
  process.exit(0);
}

if (commitMessage.includes(MINOR_COMMIT_MESSAGE)) {
  printVersion(major, minor + 1, 0);
  process.exit(0);
}

printVersion(major, minor, patch + 1);
