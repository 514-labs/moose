from python_worker_wrapper import start_worker, log

import sys
import asyncio


def main():
    log.info("Starting worker")
    # The root script where all the scripts are located
    script_root = sys.argv[1]

    try:
        asyncio.run(start_worker(script_root))
    except KeyboardInterrupt:
        # Ignore error messages when user force kills the program
        # In the future, we might want to terminate all running workflows
        pass

if __name__ == "__main__":
    main()