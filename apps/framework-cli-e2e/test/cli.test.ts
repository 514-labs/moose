import { exec, spawn, ExecException, ChildProcess } from "child_process";
import { expect } from "chai";
import * as fs from "fs";
import * as path from "path";
import { promisify } from "util";
import { createClient } from "@clickhouse/client";
import { randomUUID } from "crypto";

const execAsync = promisify(exec);
const setTimeoutAsync = promisify(setTimeout);
const CLI_PATH = path.resolve(__dirname, "../../../target/debug/moose-cli");
const TEST_PROJECT_DIR = path.join(__dirname, "test-project");
const MOOSE_LIB_PATH = path.resolve(
  __dirname,
  "../../../packages/ts-moose-lib",
);

describe("framework-cli", () => {
  let devProcess: ChildProcess | null = null;

  before(async function () {
    try {
      await fs.promises.access(CLI_PATH, fs.constants.F_OK);
    } catch (err) {
      console.error(
        `CLI not found at ${CLI_PATH}. It should be built in the pretest step.`,
      );
      throw err;
    }
  });

  const removeTestProj = () =>
    fs.rmSync(TEST_PROJECT_DIR, { recursive: true, force: true });

  after(async function () {
    if (devProcess && !devProcess.killed) {
      this.timeout(10_000);

      console.log("Stopping dev process...");
      devProcess.kill("SIGINT");

      // Wait for the devProcess to exit
      await new Promise<void>((resolve) => {
        devProcess!.on("exit", () => {
          console.log("Dev process has exited");
          resolve();
        });
      });
    }
    console.log("Stopping Docker containers and removing volumes...");
    try {
      await execAsync(
        "docker compose -f .moose/docker-compose.yml -p my-moose-app down -v",
        { cwd: TEST_PROJECT_DIR },
      );
      console.log("Docker containers stopped successfully");
    } catch (error) {
      console.error("Error stopping Docker containers:", error);
    }

    removeTestProj();
  });

  it("should return the dummy version in debug build", async () => {
    const { stdout } = await execAsync(`"${CLI_PATH}" --version`);
    const version = stdout.trim();
    const expectedVersion = "moose-cli 0.0.1";
    expect(version).to.equal(expectedVersion);
  });

  it("should init a project, install dependencies, run dev command, send a request, and stop it", async function () {
    this.timeout(120_000); // 2 minutes

    if (fs.existsSync(TEST_PROJECT_DIR)) {
      removeTestProj();
    }

    console.log("Initializing project...");
    await execAsync(
      `"${CLI_PATH}" init my-moose-app ts --location "${TEST_PROJECT_DIR}"`,
    );

    console.log("Updating package.json to use local moose-lib...");
    const packageJsonPath = path.join(TEST_PROJECT_DIR, "package.json");
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));

    packageJson.dependencies["@514labs/moose"] = `file:${MOOSE_LIB_PATH}`;

    fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2));

    console.log("Installing dependencies...");
    await new Promise<void>((resolve, reject) => {
      const npmInstall = spawn("npm", ["install"], {
        stdio: "inherit",
        cwd: TEST_PROJECT_DIR,
      });
      npmInstall.on("close", (code) => {
        console.log(`npm install exited with code ${code}`);
        code === 0
          ? resolve()
          : reject(new Error(`npm install failed with code ${code}`));
      });
    });

    console.log("Starting dev server...");
    devProcess = spawn(CLI_PATH, ["dev"], {
      stdio: "pipe",
      cwd: TEST_PROJECT_DIR,
    });

    await new Promise<void>((resolve, reject) => {
      let serverStarted = false;
      devProcess!.stdout?.on("data", async (data) => {
        const output = data.toString();
        console.log("Dev server output:", output);

        if (
          !serverStarted &&
          output.includes(
            "Your local development server is running at: http://localhost:4000/ingest",
          )
        ) {
          resolve();
          serverStarted = true;
        }
      });

      devProcess!.stderr?.on("data", (data) => {
        console.error("Dev server stderr:", data.toString());
      });

      devProcess!.on("exit", (code) => {
        console.log(`Dev process exited with code ${code}`);
        expect(code).to.equal(0);
      });

      (async () => {
        await setTimeoutAsync(60_000);
        if (devProcess!.killed) return;
        console.error("Dev server did not start or complete in time");
        devProcess!.kill("SIGINT");
        reject(new Error("Dev server timeout"));
      })();
    });

    console.log("Server started, waiting before sending test request...");

    await setTimeoutAsync(2000);

    console.log("Sending test request...");
    const eventId = randomUUID();
    try {
      const response = await fetch(
        "http://localhost:4000/ingest/UserActivity",
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            eventId: eventId,
            timestamp: "2019-01-01 00:00:01",
            userId: "123456",
            activity: "click",
          }),
        },
      );

      if (response.ok) {
        console.log("Test request sent successfully");
      } else {
        console.error("Response code:", response.status);
        console.error(`Test request failed:`, await response.text());
      }
    } catch (error) {
      console.error("Error sending test request:", error);
    }

    // Wait for data to be processed
    await setTimeoutAsync(5000);

    // Query the database
    const client = createClient({
      url: "http://localhost:18123",
      username: "panda",
      password: "pandapass",
      database: "local",
    });

    try {
      const result = await client.query({
        query: "SELECT * FROM ParsedActivity_0_0",
        format: "JSONEachRow",
      });
      const rows: any[] = await result.json();
      console.log("ParsedActivity data:", rows);

      expect(rows).to.have.lengthOf(
        1,
        "Expected exactly one row in ParsedActivity",
      );

      expect(rows[0].eventId).to.equal(
        eventId,
        "EventId in ParsedActivity should match the generated UUID",
      );
    } catch (error) {
      console.error("Error querying ClickHouse:", error);
      throw error; // Re-throw the error to fail the test
    } finally {
      await client.close();
    }
  });
});
