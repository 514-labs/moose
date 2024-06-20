export interface KPI {
  timestamp: string;
  visits: number;
  pageviews: number;
  bounce_rate: number;
  avg_session_sec: number;
}

export function getKPI() {
  const test = fetch("http://localhost:4000/consumption/kpi_timeseries")
    .then((response) => response.json())
    .then((data: KPI[]) => data)
    .catch((error) => console.error("Error:", error));

  return test;
}
