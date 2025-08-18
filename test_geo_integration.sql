-- Test script to verify geo types work end-to-end
-- This would be run against a ClickHouse instance with the geo types enabled

-- Create test table with geo types
CREATE TABLE geo_test_table (
    id String,
    name String,
    location Point,
    area Polygon,
    route LineString,
    boundaries Ring,
    multi_areas MultiPolygon,
    multi_routes MultiLineString,
    created_at DateTime
) ENGINE = MergeTree()
ORDER BY id;

-- Insert test data with geo types
INSERT INTO geo_test_table VALUES 
(
    'test_001',
    'Test Location',
    (0, 0),  -- Point
    [[(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]],  -- Polygon
    [(0, 0), (1, 1), (2, 2)],  -- LineString
    [(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)],  -- Ring
    [[[(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]]],  -- MultiPolygon
    [[(0, 0), (1, 1)], [(2, 2), (3, 3)]],  -- MultiLineString
    now()
);

-- Query the data to verify it works
SELECT 
    id,
    name,
    location,
    area,
    route,
    boundaries,
    multi_areas,
    multi_routes,
    created_at
FROM geo_test_table;

-- Test geo functions
SELECT 
    id,
    name,
    geoDistance(tupleElement(location, 1), tupleElement(location, 2), 1.0, 1.0) as distance_to_point
FROM geo_test_table;

-- Clean up
DROP TABLE geo_test_table;