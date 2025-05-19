import dgram from "dgram";
import * as oscmin from "osc-min";
import { BrainwaveData } from "./types.js";
import { Logger } from "./logger.js";
import { pcap } from "./utils.js";
import { estimateHeartRateFromPPG } from "./brainwave-analyzer.js";

export class UDPServer {
  private server!: dgram.Socket;
  private msg: BrainwaveData;
  private requestCount = 0;
  private lastRequestLog = Date.now();
  private readonly SAMPLE_LOG_INTERVAL = 5000; // 5 seconds

  // Buffer for PPG data (channel 0)
  private ppgBuffer: { timestamp: number; value: number }[] = [];
  private readonly PPG_BUFFER_SECONDS = 15;

  private lastBPM: number | null = null;

  constructor(
    private onDataUpdate: (data: BrainwaveData, bpm?: number | null) => void,
  ) {
    this.msg = {
      sessionId: "",
      timestamp: new Date(),
      bandOn: false,
      acc: { x: 0, y: 0, z: 0 },
      gyro: { x: 0, y: 0, z: 0 },
      alpha: 0,
      beta: 0,
      delta: 0,
      theta: 0,
      gamma: 0,
    };
  }

  start(port: number): void {
    this.server = dgram.createSocket("udp4");

    this.server.on("message", (buffer, _rinfo) => {
      this.handleMessage(buffer);
    });

    this.server.on("listening", () => {
      const address = this.server.address();
      Logger.info(`server listening ${address.address}:${address.port}`);
    });

    this.server.on("error", (err) => {
      Logger.error(`Server error: ${err.message}`);
    });

    this.server.bind(port);
  }

  private handleMessage(buffer: Buffer): void {
    // Log the raw UDP message as a hex+ASCII dump (16 bytes per line)
    const bytes = Array.from(buffer);
    let hexAsciiDump = "\n";
    for (let i = 0; i < bytes.length; i += 16) {
      const chunk = bytes.slice(i, i + 16);
      const hex = chunk.map((b) => b.toString(16).padStart(2, "0")).join(" ");
      const ascii = chunk
        .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : "."))
        .join("");
      hexAsciiDump += hex.padEnd(16 * 3) + "  " + ascii + "\n";
    }
    // Logger.info('UDP message hex dump:' + hexAsciiDump);
    this.requestCount++;

    const now = Date.now();
    if (now - this.lastRequestLog >= this.SAMPLE_LOG_INTERVAL) {
      const requestsPerSecond =
        this.requestCount / (this.SAMPLE_LOG_INTERVAL / 1000);
      Logger.info(
        `Sample rate: ${requestsPerSecond.toFixed(2)} samples/second`,
      );
      this.requestCount = 0;
      this.lastRequestLog = now;
    }

    const message = oscmin.fromBuffer(buffer);
    if (
      !message.elements ||
      !Array.isArray(message.elements) ||
      !message.elements[0]
    ) {
      Logger.warn("Received malformed or empty OSC message");
      return;
    }
    const type = message.elements[0].address;
    const args = message.elements[0].args;

    const bpm = this.updateMessageData(type, args);
    this.msg.timestamp = new Date();
    this.onDataUpdate(this.msg, bpm);
  }

  private updateMessageData(type: string, args: any[]): number | null {
    if (type.includes("touching_forehead")) {
      const newBandState = args[0].value === 1;
      if (newBandState !== this.msg.bandOn) {
        if (newBandState) {
          Logger.info("Band state changed from OFF to ON");
        } else {
          Logger.warn("Band state changed from ON to OFF");
        }
      }
      this.msg.bandOn = newBandState;
    }
    if (type.includes("gyro")) {
      this.msg.gyro = {
        x: pcap(args[0].value),
        y: pcap(args[1].value),
        z: pcap(args[2].value),
      };
    }
    if (type.includes("acc")) {
      this.msg.acc = {
        x: pcap(args[0].value),
        y: pcap(args[1].value),
        z: pcap(args[2].value),
      };
    }
    if (type.includes("alpha")) this.msg.alpha = pcap(args[0].value);
    if (type.includes("beta")) this.msg.beta = pcap(args[0].value);
    if (type.includes("delta")) this.msg.delta = pcap(args[0].value);
    if (type.includes("theta")) this.msg.theta = pcap(args[0].value);
    if (type.includes("gamma")) this.msg.gamma = pcap(args[0].value);
    if (type.includes("ppg")) {
      // Use channel 0 for heart rate estimation
      const now = Date.now() / 1000; // seconds
      this.ppgBuffer.push({ timestamp: now, value: args[0].value });

      // Remove old samples
      while (
        this.ppgBuffer.length > 0 &&
        now - this.ppgBuffer[0].timestamp > this.PPG_BUFFER_SECONDS
      ) {
        this.ppgBuffer.shift();
      }

      // Estimate heart rate
      const bpm = estimateHeartRateFromPPG(this.ppgBuffer);
      if (bpm) {
        Logger.info(`Estimated Heart Rate: ${bpm.toFixed(1)} BPM`);
        this.lastBPM = bpm;
        return bpm;
      } else {
        Logger.info("No plausible heart rate");
      }
      return this.lastBPM;
    }
    return this.lastBPM;
  }

  close(): void {
    if (this.server) {
      this.server.close();
    }
  }
}
