import { Key } from "@514labs/moose-lib";

export interface Brain {
  sessionId: Key<string>;
  timestamp: Date;
  bandOn: boolean;
  acc: {
    x: number;
    y: number;
    z: number;
  };
  gyro: {
    x: number;
    y: number;
    z: number;
  };
  alpha: number;
  beta: number;
  delta: number;
  theta: number;
  gamma: number;
}
