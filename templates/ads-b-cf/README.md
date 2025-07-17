# Aircraft Tracking System

This is a [Moose](https://docs.fiveonefour.com/moose) project that processes and transforms aircraft tracking data from multiple sources into a unified format for analysis and visualization.

This version of the ADS-B demo was created to highlight the new Connector Factory abstractions over connections.

## Getting Started

Prerequisites
* [Docker Desktop](https://www.docker.com/products/docker-desktop/)
* [Node](https://nodejs.org/en)
* [An Anthropic API Key](https://docs.anthropic.com/en/api/getting-started)
* [Cursor](https://www.cursor.com/) or [Claude Desktop](https://claude.ai/download)

1. Install Moose / Aurora: `bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose,aurora`
2. Create project `aurora init aircraft ads-b`
3. Install dependencies: `cd aircraft && npm install`
4. Run Moose: `moose dev`
5. Run Workflows: `moose workflow run laddAircraftETL & moose workflow run laddAircraftETL & wait`

You are ready to go!

## What you can do next?

* Add another source from [adsb.lol](https://api.adsb.lol/docs) to this project.
* You have an API you like? Modify `/app/datamodels/models.ts` with the data model you want to land; `/app/scripts/fetch_from_API` to point to the API you want, and to do any transformations you might need; edit `/app/functions/process_stream` to change any stream processing you need to do.
* try ask Aurora in Claude or Cursor about your data. Get it to create egress APIs for you.


## Architecture Overview

### Data Sources

1. **Military Aircraft API** (`https://api.adsb.lol/v2/mil`)
   - Military aircraft tracking data
   - Collected via ETL pipeline on-demand
   - Source identifier: `"Mil"`

2. **LADD Aircraft API** (`https://api.adsb.lol/v2/ladd`)
   - Limiting Aircraft Data Displayed - civilian aircraft with rich performance data
   - Enhanced fields: navigation, weather, performance metrics
   - Collected via ETL pipeline on-demand
   - Source identifier: `"LADD"`

### Data Flow & Transforms

```
┌─────────────────┐    ┌─────────────────┐
│ Military API    │    │ LADD API        │
│ /v2/mil         │    │ /v2/ladd        │
└─────────┬───────┘    └─────────┬───────┘
          │                      │
          ▼                      ▼
┌─────────────────┐    ┌─────────────────┐
│ ETL Pipeline    │    │ ETL Pipeline    │
│ +source:"Mil"   │    │ +source:"LADD"  │
└─────────┬───────┘    └─────────┬───────┘
          │                      │
          ▼                      ▼
┌─────────────────┐    ┌─────────────────┐
│ Unified Stream  │◄───┤ LADD Stream     │
│ AircraftTracking│    │ (rich data)     │
│ Data (w DLQ)    │    │ (w DLQ)         │
└─────────┬───────┘    └─────────────────┘
          │
          ▼
┌─────────────────┐
│ Stream Function │
│ +zorderCoord    │
│ +nav_mode_flags │
└─────────┬───────┘
          │
          ▼
┌─────────────────┐
│ Processed Table │
│ (ClickHouse)    │
└─────────────────┘
```

### Rate Limiting & Error Handling

The system implements a multi-layered approach to handle API rate limits and ensure reliable data collection:

#### APISource-Level Retries

Each API source has its own retry configuration with linear backoff:

**Military Aircraft API**
- **Max Retries**: 5 attempts
- **Backoff Strategy**: 1s, 2s, 3s, 4s, 5s (linear increments)
- **Total Max Delay**: 15 seconds

**LADD Aircraft API**  
- **Max Retries**: 5 attempts
- **Backoff Strategy**: 1.5s, 3s, 4.5s, 6s, 7.5s (linear increments)
- **Total Max Delay**: 22.5 seconds

#### Task-Level Retries

Both tasks have additional retry configuration:
- **Task Retries**: 3 attempts per task execution
- **Timeouts**: 5 minutes (Military), 10 minutes (LADD)

#### Connection Testing

Before each pipeline run, APISource tests the connection:
```typescript
const connectionTest = await apiSource.testConnection();
if (!connectionTest.success) {
  throw new Error(`Connection test failed: ${connectionTest.message}`);
}
```

This three-layer approach (APISource retries + workflow staggering + task retries) provides robust handling of rate limit scenarios and ensures continuous data collection.

### What's Built

#### 🔌 Ingestion APIs
- **Military Aircraft**: `POST http://localhost:4000/ingest/AircraftTrackingData`
- **LADD Aircraft**: `POST http://localhost:4000/ingest/LADDAircraftData`

#### 🌊 Data Streams
- **AircraftTrackingData**: Unified aircraft data stream
- **LADDAircraftData**: Rich LADD aircraft data stream  
- **AircraftTrackingProcessed**: Final processed data stream

#### ⚙️ Stream Functions
- **LADD→Unified Transform**: Converts LADD data to unified format
- **Processing Function**: Adds Z-order coordinates and navigation flags

#### 🗄️ Tables
- **AircraftTrackingProcessed**: Final analytical table in ClickHouse

#### 📊 Workflows
- **Military Aircraft Tracking**: Runs every 8 seconds
- **LADD Aircraft Tracking**: Runs every 12 seconds

## Sample Data at Each Stage

### 1. Raw Military Aircraft Data (API Response)
```json
{
  "hex": "ae6247",
  "flight": "WAKE13  ",
  "r": "77-0356",
  "lat": 63.851393,
  "lon": -151.486615,
  "alt_baro": 5300,
  "gs": 53.9,
  "track": 238.67,
  "squawk": "2273",
  "emergency": "none",
  "nav_modes": ["autopilot", "tcas"]
}
```

### 2. Raw LADD Aircraft Data (API Response)
```json
{
  "hex": "a08577",
  "flight": "N1323Y  ",
  "aircraft_type": "adsb_icao",
  "source": "LADD",
  "lat": 58.977868,
  "lon": -159.055797,
  "alt_baro": 5075,
  "gs": 151.4,
  "track": 143.05,
  "nav_modes": ["autopilot", "lnav"],
  "true_heading": 130.78,
  "indicated_airspeed": 145,
  "mach_number": 0.22,
  "outside_air_temp": -8.5,
  "wind_direction": 270,
  "wind_speed": 25
}
```

### 3. Unified Aircraft Stream Data
```json
{
  "hex": "ae57d4",
  "flight": "C2010   ",
  "source": "Mil",
  "lat": 61.143173,
  "lon": -150.360358,
  "alt_baro": 5925,
  "gs": 239.5,
  "track": 343,
  "nav_modes": ["autopilot", "tcas"],
  "timestamp": "2025-07-16T18:05:21.747Z"
}
```

### 4. Final Processed Table Data
```json
{
  "hex": "ae57d4",
  "flight": "C2010   ",
  "source": "Mil",
  "lat": 61.143173,
  "lon": -150.360358,
  "alt_baro": 5925,
  "gs": 239.5,
  "track": 343,
  "zorderCoordinate": 911678410,
  "autopilot": true,
  "approach": false,
  "althold": false,
  "lnav": false,
  "tcas": true,
  "timestamp": "2025-07-16T18:05:21.747Z"
}
```

## Data Statistics

- **Sources**: ~68% LADD civilian, 32% Military

## Learn More

To learn more about Moose, take a look at the following resources:

- [Moose Documentation](https://docs.fiveonefour.com/moose) - learn about Moose.
- [Aurora Documentation](https://docs.fiveonefour.com/aurora) - learn about Aurora, the MCP interface for data engineering.
- [Deploy on Boreal](https://www.fiveonefour.com/boreal)

You can check out [the Moose GitHub repository](https://github.com/514-labs/moose) - your feedback and contributions are welcome!

## Deploy on Boreal

The easiest way to deploy your Moose app is to use the [Boreal](https://www.fiveonefour.com/boreal) from Fiveonefour, the creators of Moose and Aurora.

[Sign up](https://www.boreal.cloud/sign-up).

## License

This template is MIT licensed.
