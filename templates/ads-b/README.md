# Aircraft Tracking Data Processing

This project processes and transforms aircraft tracking data from various sources into a standardized format for analysis and visualization.

# Getting Started

1. Install Moose / Aurora: `bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose,aurora`
2. Create project `aurora init aircraft ads-b`
3. Install dependencies: `npm install`
4. Add your Anthropic API Key to `/ads-b/.cursor/mcp.json`, and ensure your MCPs are running
5. Run Moose: `moose dev`
6. In a new terminal, navigate to the project directory and run ingest workflow `moose workflow run military_aircraft_tracking`

You are ready to go!

## Overview

The system takes raw aircraft tracking data (such as ADS-B data) and transforms it into a flattened data model that's optimized for storage and querying. 

It is currently only pulling from military aircraft.

## Data Models

### Input Data Models

- `AircraftTrackingData_altBaroString`: Raw aircraft data where altitude may be represented as a string (e.g., "Ground" or numeric values)

### Output Data Models

- `FlattenedAircraftTrackingData`: Standardized data model with normalized fields and additional computed values

## Processing Functions

- `AircraftTrackingData__FlattenedAircraftTrackingData.ts`: Transforms the raw data into the flattened format, handling special cases like "Ground" altitude



## License

