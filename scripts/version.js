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
const NO_RELEASE_COMMIT_MESSAGE = "[no-release]"
const MAJOR_COMMIT_MESSAGE = "[major-release]"
const MINOR_COMMIT_MESSAGE = "[minor-release]"


// We check that the current commit is not tagged yet. If it is, 
// we return its value as the version
execSync("git fetch --tags");
const tags = execSync(`git tag --points-at ${commit}`).toString().trim();

if (tags.length > 0) {
    console.log(tags)
    process.exit(0)
}

// We retrieve the last release tag.
const latestTag = execSync("git describe --tags --abbrev=0").toString().trim();
// We parse the version number from the tag
const version = latestTag.match(/v(\d+)\.(\d+)\.(\d+)/);

const commitMessage = execSync(`git log -1 --pretty=%B ${commit}`).toString().trim();

if (commitMessage.contains(NO_RELEASE_COMMIT_MESSAGE)){
    console.log(latestTag)
    process.exit(0)
}

if (commitMessage.contains(MAJOR_COMMIT_MESSAGE)){
    console.log(`v${version[1] + 1}.0.0`)
    process.exit(0)
}

if (commitMessage.contains(MINOR_COMMIT_MESSAGE)){
    console.log(`v${version[1]}.${version[2] + 1}.0`)
    process.exit(0)
}

console.log(`v${version[1]}.${version[2]}.${version[3] + 1}`)