# Live Heart Rate Leaderboard

This project is inspired by work done for the F45 global gym chain. It demonstrates how to build a fully featured real-time data pipeline using Moose. 

You'll find a simple Next.js frontend that displays the live heart rate data in a graph, and a data generator script that simulates ANT+ data. 

If you have either a Polar HR device or Whoop product that can emmit data over BLE, you can use the `bt-hr-server` script to collect data from the device and see the pipeline transform the data in a unified model. 

# Step 1: Set up Moose

1. Clone this repository
2. Create a virtual environment:
```
python -m venv venv
source venv/bin/activate
```
3. Go into the moose project directory & install dependencies:
```
cd moose-backend
pip install -e .
```
4. Run `moose dev` to start the Moose the project

This spins up a local running version of all the project's services.

- Redpanda
- Temporal
- Postgres
- Clickhouse
- Moose built Rust Ingestion / Consumption Server


The `moose dev` command will also start a `dev terminal` which will show the logs of your Moose project.

# Step 2: Send data to Moose

Create a new terminal and from the root of the project run (`moose-backend`), trigger the worflow that will send ANT+ data to Moose:

```
moose-cli workflow run ant-data-generator
```

This synthetic data generator is running inside a temporal workflow. You can see the workflow in the `app/scripts/ant-data-generator.py` file. The moose scripts primitives is a wrapper around Temporal, and makes it easy to run fault tolerant workflows. 

# Step 3: Consume data from Moose
1. Change to the `sample-nextjs-app` directory and install dependencies:
```
cd ../sample-nextjs-app
npm install
```

2. Run the Next.js app:
```
npm run dev
```

3. Open your browser and you should see live heart rate data in the graph. 

Default: http://localhost:5173/


## System Architecture

The system consists of three main components:

1. **Backend (Moose Server)**
   - Handles data ingestion from heart rate monitors
   - Processes and aggregates heart rate data
   - Provides REST APIs for frontend consumption

2. **Frontend (Next.js)**
   - Real-time heart rate visualization
   - Interactive leaderboard
   - User profile display
   - Metric selection and filtering

3. **Data Generators/Collectors**
   - ANT+ data simulator
    - Mocks the ANT+ protocol in 8 byte packets, where rollover logic is implemented
   - Bluetooth device connector
    - Connects to a BLE device and reads heart rate data

TODO: Add diagram from figma


## Prerequisites

- Python 3.8+
- Node.js 18+
- Optional: Bluetooth-enabled device (Polar or Whoop)

