# Aircraft Tracking Data Processing

This project processes and transforms aircraft tracking data from various sources into a standardized format for analysis and visualization.

## Overview

The system takes raw aircraft tracking data (such as ADS-B data) and transforms it into a flattened data model that's optimized for storage and querying. It handles different input formats, including cases where altitude data might be represented as a string with special values like "Ground".

## Key Features

- Transforms complex aircraft tracking data into a standardized format
- Handles special cases like ground-level aircraft
- Calculates Z-order curve values for geospatial indexing
- Parses navigation modes into boolean flags for easier querying
- Normalizes altitude data from different sources

## Data Models

### Input Data Models

- `AircraftTrackingData_altBaroString`: Raw aircraft data where altitude may be represented as a string (e.g., "Ground" or numeric values)

### Output Data Models

- `FlattenedAircraftTrackingData`: Standardized data model with normalized fields and additional computed values

## Processing Functions

- `AircraftTrackingData__FlattenedAircraftTrackingData.ts`: Transforms the raw data into the flattened format, handling special cases like "Ground" altitude

## Technical Details

The project uses TypeScript and the @514labs/moose-lib framework for data modeling and processing. Key technical features include:

- Z-order curve calculation for efficient geospatial indexing
- Type-safe data transformations
- Handling of special string values in numeric fields
- Boolean flag normalization for navigation modes

## Getting Started

1. Install dependencies: `npm install`
2. Build the project: `npm run build`
3. Run the data processing: `npm start`

## License

[Your license information here] 