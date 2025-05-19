export interface BrainwaveData {
  sessionId: string;
  timestamp: Date;
  bandOn: boolean;
  acc: AccelerometerData;
  gyro: GyroscopeData;
  alpha: number;
  beta: number;
  delta: number;
  theta: number;
  gamma: number;
  ppmchannel1?: number;
  ppmchannel2?: number;
  ppmchannel3?: number;
}

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
