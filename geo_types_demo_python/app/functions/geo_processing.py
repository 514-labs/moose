from moose_lib import StreamingFunction
from typing import Dict, Any, List
from datetime import datetime
import json

def geo_processing_function(source):
    """
    Process incoming geo data and convert to proper WKT format
    """
    def process_event(event: Dict[str, Any]) -> Dict[str, Any]:
        # Convert lat/lon pairs to Point WKT format
        if 'lat' in event and 'lon' in event:
            event['location'] = f"POINT({event['lon']} {event['lat']})"
        
        # Convert path arrays to LineString WKT format
        if 'path' in event and isinstance(event['path'], list):
            coords = []
            for point in event['path']:
                if isinstance(point, dict) and 'lat' in point and 'lon' in point:
                    coords.append(f"{point['lon']} {point['lat']}")
            if coords:
                event['route'] = f"LINESTRING({','.join(coords)})"
        
        # Convert polygon coordinates to Polygon WKT format
        if 'polygon_coords' in event:
            coords = event['polygon_coords']
            if coords and len(coords) > 0:
                # Ensure polygon is closed (first and last points are the same)
                if coords[0] != coords[-1]:
                    coords.append(coords[0])
                coord_str = ','.join([f"{coord[0]} {coord[1]}" for coord in coords])
                event['area'] = f"POLYGON(({coord_str}))"
        
        # Add timestamp if not present
        if 'timestamp' not in event:
            event['timestamp'] = datetime.now().isoformat()
        
        return event
    
    return source.map(process_event)

# Example usage in a streaming function
@StreamingFunction
def track_vehicles(source):
    """
    Process vehicle tracking data with geo types
    """
    def transform_vehicle_data(event):
        return {
            'id': f"vehicle_{event['vehicle_id']}_{int(datetime.now().timestamp())}",
            'vehicle_id': event['vehicle_id'],
            'route': f"LINESTRING({','.join([f'{p[0]} {p[1]}' for p in event['route_points']])})",
            'service_area': f"POLYGON(({','.join([f'{p[0]} {p[1]}' for p in event['service_boundary']])}))",
            'timestamp': datetime.now()
        }
    
    return source.map(transform_vehicle_data)