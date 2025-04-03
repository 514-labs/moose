# Aircraft Tracking Data Processing

This project processes and transforms aircraft tracking data from various sources into a standardized format for analysis and visualization.

# Getting Started

1. Install Moose / Aurora: `bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose,aurora`
2. Install dependencies: `npm install`
3. Add your Anthropic API Key to `/ads-b/.cursor/mcp.json`, and ensure your MCPs are running
4. Run Moose: `moose dev`
5. In a new terminal, navigate to the project directory and run ingest workflow `moose workflow run military_aircraft_tracking`
6. Chat! and use your egress agent!

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

