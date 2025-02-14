import {
  makeTelemetryFilterString,
  DefaultLogger,
  Runtime,
} from "@temporalio/worker";

class LoggerSingleton {
  private static instance: DefaultLogger | null = null;

  private constructor() {}

  public static initializeLogger(): DefaultLogger {
    if (!LoggerSingleton.instance) {
      LoggerSingleton.instance = new DefaultLogger(
        "DEBUG",
        ({ level, message }) => {
          console.log(`${level} | ${message}`);
        },
      );

      Runtime.install({
        logger: LoggerSingleton.instance,
        telemetryOptions: {
          logging: {
            filter: makeTelemetryFilterString({ core: "INFO", other: "INFO" }),
            forward: {},
          },
        },
      });
    }

    return LoggerSingleton.instance;
  }

  public static getInstance(): DefaultLogger {
    return LoggerSingleton.instance!;
  }
}

export const initializeLogger = LoggerSingleton.initializeLogger;
