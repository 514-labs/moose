import { Readable } from "moose-lib";
import { LocationData, TrackingEvent } from "../datamodels/models";

interface InputEvent {
    user_id: string;
    lat: number;
    lon: number;
    path: Array<{lat: number, lon: number}>;
}

export default {
    run: (source: Readable<InputEvent>) => {
        return source
            .map((event) => ({
                id: `${event.user_id}_${Date.now()}`,
                user_id: event.user_id,
                current_position: `POINT(${event.lon} ${event.lat})` as Point,
                path_traveled: `LINESTRING(${event.path.map(p => `${p.lon} ${p.lat}`).join(',')})` as LineString,
                timestamp: new Date()
            }))
    }
};