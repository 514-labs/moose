import { IngestPipeline, Key, ClickHouseInt, ClickHouseDecimal, ClickHousePrecision, ClickHouseByteSize, ClickHouseNamedTuple } from "@514labs/moose-lib";
import typia from "typia";

// Example interface showing geo type usage
export interface LocationData {
    id: Key<string>;
    name: string;
    location: Point;  // Point geo type for coordinates
    area: Polygon;    // Polygon geo type for coverage area
    route: LineString;  // LineString geo type for paths
    created_at: Date;
}

export interface TrackingEvent {
    id: Key<string>;
    user_id: string;
    current_position: Point;
    path_traveled: LineString;
    timestamp: Date;
}

export interface ServiceArea {
    id: Key<string>;
    name: string;
    coverage: MultiPolygon;  // MultiPolygon for complex service areas
    boundaries: Ring;        // Ring for simple boundaries
    routes: MultiLineString; // MultiLineString for multiple routes
}

export const LocationDataPipeline = new IngestPipeline<LocationData>("LocationData", {
    table: true,
    stream: true,
    ingest: true,
});

export const TrackingEventPipeline = new IngestPipeline<TrackingEvent>("TrackingEvent", {
    table: true,
    stream: true,
    ingest: true,
});

export const ServiceAreaPipeline = new IngestPipeline<ServiceArea>("ServiceArea", {
    table: true,
    stream: true,
    ingest: true,
});