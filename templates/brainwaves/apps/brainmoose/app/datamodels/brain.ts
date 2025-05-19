import { Key } from "@514labs/moose-lib";

interface AccelerometerData {
  x: number;
  y: number;
  z: number;
}

interface GyroscopeData {
  x: number;
  y: number;
  z: number;
}

interface PpmData {
  channel1: number;
  channel2: number;
  channel3: number;
}

export interface Brain {
  sessionId: Key<string>;
  timestamp: Date;
  bandOn: boolean;
  acc: AccelerometerData;
  gyro: GyroscopeData;
  alpha: number;
  beta: number;
  delta: number;
  theta: number;
  gamma: number;
  ppm?: PpmData;
}
