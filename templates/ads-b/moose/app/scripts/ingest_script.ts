import * as http from "http";

async function mapToAircraftTrackingData(aircraft: any): Promise<any> {
  return {
    hex: aircraft.hex,
    type: aircraft.type || "",
    flight: aircraft.flight || "",
    r: aircraft.r || "",
    t: aircraft.t || "",
    dbFlags: aircraft.dbFlags || 0,
    lat: aircraft.lat || 0,
    lon: aircraft.lon || 0,
    alt_baro: aircraft.alt_baro || 0,
    alt_geom: aircraft.alt_geom || 0,
    gs: aircraft.gs || 0,
    track: aircraft.track || 0,
    baro_rate: aircraft.baro_rate || 0,
    geom_rate: aircraft.geom_rate,
    squawk: aircraft.squawk || "",
    emergency: aircraft.emergency || "",
    category: aircraft.category || "",
    nav_qnh: aircraft.nav_qnh,
    nav_altitude_mcp: aircraft.nav_altitude_mcp,
    nav_heading: aircraft.nav_heading,
    nav_modes: aircraft.nav_modes,
    nic: aircraft.nic || 0,
    rc: aircraft.rc || 0,
    seen_pos: aircraft.seen_pos || 0,
    version: aircraft.version || 0,
    nic_baro: aircraft.nic_baro || 0,
    nac_p: aircraft.nac_p || 0,
    nac_v: aircraft.nac_v || 0,
    sil: aircraft.sil || 0,
    sil_type: aircraft.sil_type || "",
    gva: aircraft.gva || 0,
    sda: aircraft.sda || 0,
    alert: aircraft.alert || 0,
    spi: aircraft.spi || 0,
    mlat: aircraft.mlat || [],
    tisb: aircraft.tisb || [],
    messages: aircraft.messages || 0,
    seen: aircraft.seen || 0,
    rssi: aircraft.rssi || 0,
  };
}

async function mapToAircraftTrackingDataAltBaroInt(
  aircraft: any,
): Promise<any> {
  return {
    hex: aircraft.hex,
    type: aircraft.type || "",
    flight: aircraft.flight || "",
    r: aircraft.r || "",
    t: aircraft.t || "",
    dbFlags: aircraft.dbFlags || 0,
    lat: aircraft.lat || 0,
    lon: aircraft.lon || 0,
    alt_baro: parseInt(aircraft.alt_baro) || 0, // Convert to integer
    alt_geom: aircraft.alt_geom || 0,
    gs: aircraft.gs || 0,
    track: aircraft.track || 0,
    baro_rate: aircraft.baro_rate || 0,
    geom_rate: aircraft.geom_rate,
    squawk: aircraft.squawk || "",
    emergency: aircraft.emergency || "",
    category: aircraft.category || "",
    nav_qnh: aircraft.nav_qnh,
    nav_altitude_mcp: aircraft.nav_altitude_mcp,
    nav_heading: aircraft.nav_heading,
    nav_modes: aircraft.nav_modes,
    nic: aircraft.nic || 0,
    rc: aircraft.rc || 0,
    seen_pos: aircraft.seen_pos || 0,
    version: aircraft.version || 0,
    nic_baro: aircraft.nic_baro || 0,
    nac_p: aircraft.nac_p || 0,
    nac_v: aircraft.nac_v || 0,
    sil: aircraft.sil || 0,
    sil_type: aircraft.sil_type || "",
    gva: aircraft.gva || 0,
    sda: aircraft.sda || 0,
    alert: aircraft.alert || 0,
    spi: aircraft.spi || 0,
    mlat: aircraft.mlat || [],
    tisb: aircraft.tisb || [],
    messages: aircraft.messages || 0,
    seen: aircraft.seen || 0,
    rssi: aircraft.rssi || 0,
  };
}

async function mapToAircraftTrackingDataAltBaroString(
  aircraft: any,
): Promise<any> {
  return {
    hex: aircraft.hex,
    type: aircraft.type || "",
    flight: aircraft.flight || "",
    r: aircraft.r || "",
    t: aircraft.t || "",
    dbFlags: aircraft.dbFlags || 0,
    lat: aircraft.lat || 0,
    lon: aircraft.lon || 0,
    alt_baro: String(aircraft.alt_baro || "0"), // Convert to string
    alt_geom: aircraft.alt_geom || 0,
    gs: aircraft.gs || 0,
    track: aircraft.track || 0,
    baro_rate: aircraft.baro_rate || 0,
    geom_rate: aircraft.geom_rate,
    squawk: aircraft.squawk || "",
    emergency: aircraft.emergency || "",
    category: aircraft.category || "",
    nav_qnh: aircraft.nav_qnh,
    nav_altitude_mcp: aircraft.nav_altitude_mcp,
    nav_heading: aircraft.nav_heading,
    nav_modes: aircraft.nav_modes,
    nic: aircraft.nic || 0,
    rc: aircraft.rc || 0,
    seen_pos: aircraft.seen_pos || 0,
    version: aircraft.version || 0,
    nic_baro: aircraft.nic_baro || 0,
    nac_p: aircraft.nac_p || 0,
    nac_v: aircraft.nac_v || 0,
    sil: aircraft.sil || 0,
    sil_type: aircraft.sil_type || "",
    gva: aircraft.gva || 0,
    sda: aircraft.sda || 0,
    alert: aircraft.alert || 0,
    spi: aircraft.spi || 0,
    mlat: aircraft.mlat || [],
    tisb: aircraft.tisb || [],
    messages: aircraft.messages || 0,
    seen: aircraft.seen || 0,
    rssi: aircraft.rssi || 0,
  };
}

async function sendToMoose(data: any, endpoint: string): Promise<void> {
  const options = {
    hostname: "localhost",
    port: 4000,
    path: `/ingest/${endpoint}/0.0`,
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
  };

  return new Promise((resolve, reject) => {
    const req = http.request(options, (res) => {
      let responseData = "";
      res.on("data", (chunk) => {
        responseData += chunk;
      });
      res.on("end", () => {
        if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
          console.log(`Successfully sent to Moose: ${responseData}`);
          resolve();
        } else {
          reject(new Error(`HTTP Error: ${res.statusCode} - ${responseData}`));
        }
      });
    });

    req.on("error", (error) => {
      reject(error);
    });

    req.write(JSON.stringify(data));
    req.end();
  });
}

async function fetchMilitaryAircraftData(): Promise<void> {
  try {
    const url = "https://api.adsb.lol/v2/mil";
    const controller = new AbortController();

    // Set 30 second timeout
    const timeoutId = setTimeout(() => controller.abort(), 30000);

    const response = await fetch(url, {
      method: "GET",
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();

    // Add collection timestamp if not present
    const timestamp = new Date().toISOString();
    const enrichedData = {
      ...data,
      collectionTimestamp: timestamp,
    };

    // Write response to stdout
    console.log(JSON.stringify(enrichedData));

    // Process and send each aircraft to Moose
    if (enrichedData.ac && Array.isArray(enrichedData.ac)) {
      for (const aircraft of enrichedData.ac) {
        try {
          // Send to alt_baro as integer endpoint
          const mappedDataAltBaroInt =
            await mapToAircraftTrackingDataAltBaroInt(aircraft);
          await sendToMoose(
            mappedDataAltBaroInt,
            "AircraftTrackingData_altBaroInt",
          );

          // Send to alt_baro as string endpoint
          const mappedDataAltBaroString =
            await mapToAircraftTrackingDataAltBaroString(aircraft);
          await sendToMoose(
            mappedDataAltBaroString,
            "AircraftTrackingData_altBaroString",
          );
        } catch (error) {
          console.log(`Error processing aircraft ${aircraft.hex}: ${error}`);
        }
      }
    }
  } catch (error) {
    if (error instanceof Error) {
      console.log(`Error: ${error.message}`);
    }
  }
}

// Execute the fetch
fetchMilitaryAircraftData();
