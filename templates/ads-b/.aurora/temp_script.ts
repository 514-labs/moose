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
  } catch (error) {
    if (error instanceof Error) {
      // Write error to stdout
      console.log(`Error: ${error.message}`);
    }
  }
}

// Execute the fetch
fetchMilitaryAircraftData();
