from python_worker_wrapper import start_worker, log

import sys
import asyncio


def main():
    log.info("Starting worker")
    temporal_url = sys.argv[1]
    # The root script where all the scripts are located
    script_root = sys.argv[2]
    # Connection configs
    client_cert = sys.argv[3]
    client_key = sys.argv[4]
    api_key = sys.argv[5]

    try:
        asyncio.run(start_worker(temporal_url, script_root, client_cert, client_key, api_key))
    except KeyboardInterrupt:
        # Ignore error messages when user force kills the program
        # In the future, we might want to terminate all running workflows
        pass

if __name__ == "__main__":
    main()