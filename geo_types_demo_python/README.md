# Geo Types Demo - Python

This demo application showcases the new geo types support in Moose for Python projects.

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
```python
class LocationData(BaseModel):
    id: Key[str]
    name: str
    location: Point      # Exact coordinates
    area: Polygon        # Coverage area
    route: LineString    # Path information
    created_at: datetime
```

### TrackingEvent
Real-time tracking with geo data:
```python
class TrackingEvent(BaseModel):
    id: Key[str]
    user_id: str
    current_position: Point    # Current location
    path_traveled: LineString  # Movement path
    timestamp: datetime
```

### ServiceArea
Service coverage with complex geo shapes:
```python
class ServiceArea(BaseModel):
    id: Key[str]
    name: str
    coverage: MultiPolygon     # Service areas
    boundaries: Ring           # Simple boundaries
    routes: MultiLineString    # Multiple routes
```

## Usage

1. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

2. Start the development server:
   ```bash
   moose dev
   ```

3. Send geo data to your endpoints:
   ```python
   # Example: Sending location data
   location_data = {
       "id": "loc_001",
       "name": "Central Park",
       "location": "POINT(-73.968285 40.785091)",
       "area": "POLYGON((-73.9857 40.7484, -73.9857 40.8007, -73.9479 40.8007, -73.9479 40.7484, -73.9857 40.7484))",
       "route": "LINESTRING(-73.968285 40.785091, -73.967285 40.786091, -73.966285 40.787091)",
       "created_at": "2024-01-01T00:00:00Z"
   }
   ```

## Streaming Functions

The demo includes streaming functions that convert common geo data formats:

### geo_processing_function
Converts lat/lon pairs and coordinate arrays to proper WKT format:
- `lat/lon` → `POINT(lon lat)`
- `path` array → `LINESTRING(...)`  
- `polygon_coords` → `POLYGON(...)`

### track_vehicles
Processes vehicle tracking data with route and service area geo types.

## Benefits

- **Type Safety**: Full Python type hints for geo types
- **ClickHouse Integration**: Native geo type support in ClickHouse
- **Performance**: Spatial indexing and geo functions
- **Standards Compliance**: WKT (Well-Known Text) format support

## Real-World Use Cases

- **Fleet Management**: Track vehicle locations and routes
- **Delivery Services**: Define delivery zones and optimize routes
- **Location Analytics**: Analyze spatial patterns and user behavior
- **Geofencing**: Create location-based triggers and alerts
- **Urban Planning**: Analyze city infrastructure and traffic patterns

## Example Data Ingestion

```python
import requests
import json

# Send tracking event
tracking_data = {
    "id": "track_001",
    "user_id": "user_123",
    "current_position": "POINT(-122.4194 37.7749)",
    "path_traveled": "LINESTRING(-122.4194 37.7749, -122.4094 37.7849, -122.3994 37.7949)",
    "timestamp": "2024-01-01T12:00:00Z"
}

response = requests.post(
    "http://localhost:4000/ingest/TrackingEvent",
    json=tracking_data
)
```