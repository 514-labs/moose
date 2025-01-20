import http from "node:http";

/**
 * Logs a message to the console server with timestamp
 * @param message The message to log
 */
const logToConsole = async (message: string) => {
  const data = JSON.stringify({
    timestamp: new Date().toISOString(),
    message: message,
  });

  const options = {
    hostname: "localhost",
    port: 8081,
    path: "/",
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Content-Length": Buffer.byteLength(data),
    },
  };

  const req = http.request(options, (res) => {
    if (res.statusCode !== 200) {
      console.error(`Failed to send log: ${res.statusCode}`);
    }
  });

  req.on("error", (error) => {
    console.error("Error sending log:", error);
  });

  req.write(data);
  req.end();
};

export { logToConsole };
