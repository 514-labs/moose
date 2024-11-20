import cluster from "node:cluster";
import { availableParallelism } from "node:os";
import { exit } from "node:process";

export class Cluster<C> {
  private shutdownInProgress: boolean = false;
  private hasCleanWorkerExit: boolean = true;

  private processStr = `${cluster.isPrimary ? "primary" : "worker"} process ${process.pid}`;

  private workerStart: () => Promise<C>;
  private workerStop: (c: C) => Promise<void>;

  private startOutput: C | undefined;

  constructor(options: {
    workerStart: () => Promise<C>;
    workerStop: (c: C) => Promise<void>;
  }) {
    this.workerStart = options.workerStart;
    this.workerStop = options.workerStop;
  }

  async start() {
    const cpuCount = availableParallelism();
    // Always use at least 1 CPU and leave some capacity for the other services.
    const usedCpuCount = Math.max(1, Math.round(Math.floor(cpuCount * 0.7)));

    process.on("SIGTERM", this.gracefulClusterShutdown("SIGTERM"));
    process.on("SIGINT", this.gracefulClusterShutdown("SIGINT"));

    if (cluster.isPrimary) {
      await this.bootWorkers(usedCpuCount);
    } else {
      this.startOutput = await this.workerStart();
    }
  }

  bootWorkers = async (numWorkers: number) => {
    console.info(`Setting ${numWorkers} workers...`);

    for (let i = 0; i < numWorkers; i++) {
      cluster.fork(); //create the workers
    }

    //Setting up lifecycle event listeners for worker processes
    cluster.on("online", (worker) => {
      console.info(`worker process ${worker.process.pid} is online`);
    });

    cluster.on("exit", (worker, code, signal) => {
      console.info(
        `worker ${worker.process.pid} exited with code ${code} and signal ${signal}`,
      );
      if (this.shutdownInProgress && code != 0) {
        this.hasCleanWorkerExit = false;
      }
    });

    cluster.on("disconnect", (worker) => {
      console.info(`worker process ${worker.process.pid} has disconnected`);
    });
  };

  gracefulClusterShutdown = (signal: NodeJS.Signals) => async () => {
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
        await this.shutdownWorkers(signal);
        console.info(`${this.processStr} - worker shutdown successful`);
      } else {
        await this.workerStop(this.startOutput!);
        console.info(`${this.processStr} shutdown successful`);
        this.hasCleanWorkerExit ? exit(0) : exit(1);
      }
    } catch (e) {
      console.error(`${this.processStr} - shutdown failed`, e);
      exit(1);
    }
  };

  shutdownWorkers = (signal: NodeJS.Signals) => {
    return new Promise<void>((resolve, reject) => {
      // This is only necessary for the primary process
      if (!cluster.isPrimary) {
        return resolve();
      }

      if (!cluster.workers) {
        return resolve();
      }

      const workerIds = Object.keys(cluster.workers);
      if (workerIds.length == 0) {
        return resolve();
      }

      let workersAlive = 0;
      let funcRun = 0;

      // Count the number of alive workers and keep looping until the number is zero.
      const cleanWorkers = () => {
        ++funcRun;
        workersAlive = 0;

        Object.values(cluster.workers || {})
          .filter((worker) => !!worker)
          .forEach((worker) => {
            if (!worker.isDead()) {
              ++workersAlive;
              if (funcRun == 1) {
                //On the first execution of the function, send the received signal to all the workers
                worker.kill(signal);
              }
            }
          });

        console.info(workersAlive + " workers alive");
        if (workersAlive == 0) {
          //Clear the interval when all workers are dead
          clearInterval(interval);
          return resolve();
        }
      };
      const interval = setInterval(cleanWorkers, 500);
    });
  };
}
