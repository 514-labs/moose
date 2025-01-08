from python_wrapper import run_workflow
import os
import sys
import json
import asyncio

def main():
    script_path = sys.argv[1]
    input_data = (
        json.loads(os.environ["MOOSE_INPUT"]) if "MOOSE_INPUT" in os.environ else None
    )
    asyncio.run(run_workflow(script_path, input_data))

if __name__ == "__main__":
    main()