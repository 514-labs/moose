from python_worker_wrapper import start_worker, log

import sys
import asyncio


def main():
    log.info("Starting worker")
    # The root script where all the scripts are located
    script_root = sys.argv[1]

    asyncio.run(start_worker(script_root))

if __name__ == "__main__":
    main()