import cluster from "node:cluster";
import { availableParallelism } from "node:os";
import { exit } from "node:process";

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
  private workerStart: () => Promise<C>;
  private workerStop: (c: C) => Promise<void>;

  // Result from starting worker, needed for cleanup
  private startOutput: C | undefined;

  /**
   * Creates a new cluster manager
   * @param options Object containing worker lifecycle functions
   */
  constructor(options: {
    workerStart: () => Promise<C>;
    workerStop: (c: C) => Promise<void>;
  }) {
    this.workerStart = options.workerStart;
    this.workerStop = options.workerStop;
  }

  /**
   * Starts the cluster by spawning worker processes
   */
  async start() {
    const cpuCount = availableParallelism();
    // Always use at least 1 CPU and leave some capacity for the other services.
    const usedCpuCount = Math.max(1, Math.round(Math.floor(cpuCount * 0.7)));

    // Set up signal handlers for graceful shutdown
    process.on("SIGTERM", this.gracefulClusterShutdown("SIGTERM"));
    process.on("SIGINT", this.gracefulClusterShutdown("SIGINT"));

    if (cluster.isPrimary) {
      await this.bootWorkers(usedCpuCount);
    } else {
      this.startOutput = await this.workerStart();
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
