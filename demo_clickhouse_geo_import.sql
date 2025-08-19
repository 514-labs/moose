-- Demo: ClickHouse table with geo types that can now be imported into Moose
-- Before the geo types implementation, this would fail with "UnsupportedType" errors
-- After the implementation, this should work seamlessly

-- Create a realistic geo-enabled table
CREATE TABLE location_analytics (
    id String,
    user_id String,
    session_id String,
    
    -- Geo types that are now supported
    current_location Point,
    travel_path LineString,
    service_area Polygon,
    coverage_zones MultiPolygon,
    boundary_ring Ring,
    route_collection MultiLineString,
    
    -- Standard types
    timestamp DateTime,
    device_type String,
    accuracy_meters Float32,
    speed_kmh Float32,
    
    -- Nullable geo types
    previous_location Nullable(Point),
    planned_route Nullable(LineString)
) ENGINE = MergeTree()
ORDER BY (timestamp, user_id);

-- Insert sample data
INSERT INTO location_analytics VALUES 
(
    'loc_001',
    'user_123', 
    'session_456',
    
    -- Geo data in ClickHouse format
    (37.7749, -122.4194),  -- Point: San Francisco
    [(37.7749, -122.4194), (37.7849, -122.4094), (37.7949, -122.3994)],  -- LineString
    [[(37.7749, -122.4194), (37.7849, -122.4094), (37.7949, -122.3994), (37.7749, -122.4194)]],  -- Polygon
    [[[(37.7749, -122.4194), (37.7849, -122.4094), (37.7949, -122.3994), (37.7749, -122.4194)]]],  -- MultiPolygon
    [(37.7749, -122.4194), (37.7849, -122.4094), (37.7949, -122.3994), (37.7749, -122.4194)],  -- Ring
    [[(37.7749, -122.4194), (37.7849, -122.4094)], [(37.7949, -122.3994), (37.8049, -122.3894)]],  -- MultiLineString
    
    now(),
    'mobile',
    10.5,
    25.3,
    
    NULL,  -- previous_location
    NULL   -- planned_route
);

-- Test geo functions that now work with imported tables
SELECT 
    id,
    user_id,
    current_location,
    geoDistance(
        tupleElement(current_location, 1), 
        tupleElement(current_location, 2), 
        37.7849, 
        -122.4094
    ) as distance_to_target,
    travel_path,
    service_area
FROM location_analytics
WHERE distance_to_target < 1000;  -- Within 1km

-- Demonstrate that this table can now be imported into Moose
-- Command: moose import --from-clickhouse --table location_analytics
-- Result: ✅ Success (previously would fail with UnsupportedType errors)