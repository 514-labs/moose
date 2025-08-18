# Geo Types Demo - TypeScript

This demo application showcases the new geo types support in Moose for TypeScript projects.

## Features Demonstrated

- **Point**: Store location coordinates (lat/lon)
- **Polygon**: Define service areas and boundaries
- **LineString**: Track routes and paths
- **MultiPolygon**: Handle complex coverage areas
- **Ring**: Simple boundary definitions
- **MultiLineString**: Multiple route collections

## Data Models

### LocationData
Stores location information with geo types:
- `location: Point` - Exact coordinates
- `area: Polygon` - Coverage area
- `route: LineString` - Path information

### TrackingEvent
Real-time tracking with geo data:
- `current_position: Point` - Current location
- `path_traveled: LineString` - Movement path

### ServiceArea
Service coverage with complex geo shapes:
- `coverage: MultiPolygon` - Service areas
- `boundaries: Ring` - Simple boundaries
- `routes: MultiLineString` - Multiple routes

## Usage

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the development server:
   ```bash
   npm run dev
   ```

3. Send geo data to your endpoints:
   ```javascript
   // Example: Sending location data
   const locationData = {
     id: "loc_001",
     name: "Central Park",
     location: "POINT(-73.968285 40.785091)",
     area: "POLYGON((-73.9857 40.7484, -73.9857 40.8007, -73.9479 40.8007, -73.9479 40.7484, -73.9857 40.7484))",
     route: "LINESTRING(-73.968285 40.785091, -73.967285 40.786091, -73.966285 40.787091)",
     created_at: new Date()
   };
   ```

## Benefits

- **Type Safety**: Full TypeScript support for geo types
- **ClickHouse Integration**: Native geo type support in ClickHouse
- **Performance**: Spatial indexing and geo functions
- **Standards Compliance**: WKT (Well-Known Text) format support

## Real-World Use Cases

- **Location Services**: Store and query location data
- **Delivery Tracking**: Track vehicle routes and delivery areas
- **Geofencing**: Define service boundaries and coverage areas
- **Analytics**: Analyze spatial patterns and movements