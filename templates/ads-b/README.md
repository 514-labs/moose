# Aircraft Tracking Data Processing

This project processes and transforms aircraft tracking data from various sources into a standardized format for analysis and visualization.

# Getting Started

1. Install dependencies: `npm install`
2. Add your Anthropic API Key to `/ads-b/.cursor/mcp.json`, and ensure your MCPs are running
3. Run Moose: `moose dev`
4. Run ingest workflow `moose workflow run military_aircraft_tracking`
5. Chat! and use your egress agent!

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

