import cluster from "node:cluster";
import { availableParallelism } from "node:os";
import { exit } from "node:process";
import { Worker } from "node:cluster";

/**
 * Class for managing a cluster of worker processes
 * C represents the type of output from worker startup
 */
export class Cluster<C> {
  // Tracks if shutdown is currently in progress
  private shutdownInProgress: boolean = false;
  // Tracks if workers exited cleanly during shutdown
  private hasCleanWorkerExit: boolean = true;

  // String identifying if this is primary or worker process
  private processStr = `${cluster.isPrimary ? "primary" : "worker"} process ${process.pid}`;

  // Functions for starting and stopping workers
  private workerStart: (w: Worker, paralelism: number) => Promise<C>;
  private workerStop: (c: C) => Promise<void>;

  // Result from starting worker, needed for cleanup
  private startOutput: C | undefined;
  private maxCpuUsageRatio: number;
  private usedCpuCount: number;

  /**
   * Creates a new cluster manager
   * @param options Object containing worker lifecycle functions
   */
  constructor(options: {
    workerStart: (w: Worker, paralelism: number) => Promise<C>;
    workerStop: (c: C) => Promise<void>;
    maxCpuUsageRatio?: number;
    maxWorkerCount?: number;
  }) {
    this.workerStart = options.workerStart;
    this.workerStop = options.workerStop;
    if (
      options.maxCpuUsageRatio &&
      (options.maxCpuUsageRatio > 1 || options.maxCpuUsageRatio < 0)
    ) {
      throw new Error("maxCpuUsageRatio must be between 0 and 1");
    }
    this.maxCpuUsageRatio = options.maxCpuUsageRatio || 0.7;
    this.usedCpuCount = this.computeCPUUsageCount(
      this.maxCpuUsageRatio,
      options.maxWorkerCount,
    );
  }

  computeCPUUsageCount(cpuUsageRatio: number, maxWorkerCount?: number) {
    const cpuCount = availableParallelism();
    const maxWorkers = maxWorkerCount || cpuCount;
    // Always use at least 1 CPU and leave some capacity for the other services.
    // As long as all the services are running on the same machine, we should
    // move this to the rust side. The Rust side could be doing the resource control.
    // Or we could have a separate service that manages the resources.
    // This is a temporary solution.
    return Math.min(
      maxWorkers,
      Math.max(1, Math.floor(cpuCount * cpuUsageRatio)),
    );
  }

  /**
   * Starts the cluster by spawning worker processes
   */
  async start() {
    // Set up signal handlers for graceful shutdown
    process.on("SIGTERM", this.gracefulClusterShutdown("SIGTERM"));
    process.on("SIGINT", this.gracefulClusterShutdown("SIGINT"));

    if (cluster.isPrimary) {
      const parentPid = process.ppid; // Parent process ID

      // Kill onself if parent process is dead
      setInterval(() => {
        try {
          // Check if the process is still alive
          // This won't acutally kill the process, just check if it's alive
          process.kill(parentPid, 0);
        } catch (e) {
          console.log("Parent process has exited.");
          this.gracefulClusterShutdown("SIGTERM")();
        }
      }, 1000);

      await this.bootWorkers(this.usedCpuCount);
    } else {
      if (!cluster.worker) {
        throw new Error(
          "Worker is not defined, it should be defined in worker process",
        );
      }

      this.startOutput = await this.workerStart(
        cluster.worker,
        this.usedCpuCount,
      );
    }
  }

  /**
   * Spawns the requested number of worker processes and sets up event handlers
   */
  bootWorkers = async (numWorkers: number) => {
    console.info(`Setting ${numWorkers} workers...`);

    // Create the worker processes
    for (let i = 0; i < numWorkers; i++) {
      cluster.fork();
    }

    // Set up event handlers for worker lifecycle events
    cluster.on("online", (worker) => {
      console.info(`worker process ${worker.process.pid} is online`);
    });

    cluster.on("exit", (worker, code, signal) => {
      console.info(
        `worker ${worker.process.pid} exited with code ${code} and signal ${signal}`,
      );

      if (!this.shutdownInProgress) {
        // Unexpected worker death - restart it in a 10 seconds
        setTimeout(() => cluster.fork(), 10000);
      }

      // Track if any workers exit with non-zero code during shutdown
      if (this.shutdownInProgress && code != 0) {
        this.hasCleanWorkerExit = false;
      }
    });

    cluster.on("disconnect", (worker) => {
      console.info(`worker process ${worker.process.pid} has disconnected`);
    });
  };

  /**
   * Creates a handler function for graceful shutdown on a given signal
   */
  gracefulClusterShutdown = (signal: NodeJS.Signals) => async () => {
    // Prevent multiple concurrent shutdowns
    if (this.shutdownInProgress) {
      return;
    }

    this.shutdownInProgress = true;
    this.hasCleanWorkerExit = true;

    console.info(
      `Got ${signal} on ${this.processStr}. Graceful shutdown start at ${new Date().toISOString()}`,
    );

    try {
      if (cluster.isPrimary) {
        // Primary process shuts down all workers
        await this.shutdownWorkers(signal);
        console.info(`${this.processStr} - worker shutdown successful`);
      } else {
        // Worker process runs cleanup and exits
        await this.workerStop(this.startOutput!);
        console.info(`${this.processStr} shutdown successful`);
        this.hasCleanWorkerExit ? exit(0) : exit(1);
      }
    } catch (e) {
      console.error(`${this.processStr} - shutdown failed`, e);
      exit(1);
    }
  };

  /**
   * Gracefully shuts down all worker processes
   */
  shutdownWorkers = (signal: NodeJS.Signals) => {
    return new Promise<void>((resolve, reject) => {
      // This is only necessary for the primary process
      if (!cluster.isPrimary) {
        return resolve();
      }

      // Nothing to do if no workers exist
      if (!cluster.workers) {
        return resolve();
      }

      const workerIds = Object.keys(cluster.workers);
      if (workerIds.length == 0) {
        return resolve();
      }

      // Track number of workers still alive and times function has run
      let workersAlive = 0;
      let funcRun = 0;

      // Function to check worker status and terminate if needed
      const cleanWorkers = () => {
        ++funcRun;
        workersAlive = 0;

        Object.values(cluster.workers || {})
          .filter((worker) => !!worker)
          .forEach((worker) => {
            if (!worker.isDead()) {
              ++workersAlive;
              if (funcRun == 1) {
                // Only send kill signal on first run
                worker.kill(signal);
              }
            }
          });

        console.info(workersAlive + " workers alive");
        if (workersAlive == 0) {
          // All workers terminated, resolve promise
          clearInterval(interval);
          return resolve();
        }
      };

      // Check worker status every 500ms
      const interval = setInterval(cleanWorkers, 500);
    });
  };
}
