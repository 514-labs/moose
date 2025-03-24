"use client";

import { useEffect, useState } from "react";

interface TransportSummary {
  totalSchedules: number;
  appliedFilters: {
    scheduleType: string | null;
    dateRange: {
      from: string | null;
      to: string | null;
    } | null;
  };
}

interface TransitStats {
  avgTransitMinutes: number;
  minTransitMinutes: number;
  maxTransitMinutes: number;
}

interface ScheduleType {
  scheduleType: string;
  scheduleCount: number;
  percentage: number;
}

interface TimeOfDay {
  timeOfDay: string;
  scheduleCount: number;
}

interface Route {
  origin: string;
  destination: string;
  scheduleCount: number;
}

interface TransportResponse {
  summary: TransportSummary;
  transitStats: TransitStats;
  scheduleTypes: ScheduleType[];
  timeOfDay: TimeOfDay[];
  topRoutes: Route[];
}

export default function TransportPage() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<TransportResponse | null>(null);
  const [scheduleType, setScheduleType] = useState<string>("");
  const [dateFrom, setDateFrom] = useState<string>("");
  const [dateTo, setDateTo] = useState<string>("");

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);

        let url = "http://localhost:4001/getTransportScheduleSummary";
        const params = new URLSearchParams();

        if (scheduleType) {
          params.append("scheduleType", scheduleType);
        }

        if (dateFrom) {
          params.append("dateFrom", dateFrom);
        }

        if (dateTo) {
          params.append("dateTo", dateTo);
        }

        const queryString = params.toString();
        if (queryString) {
          url += `?${queryString}`;
        }

        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(
            `Failed to fetch data: ${response.status} ${response.statusText}`,
          );
        }
        const jsonData = await response.json();
        setData(jsonData);
      } catch (err) {
        console.error("Error fetching data:", err);
        setError(err instanceof Error ? err.message : "Unknown error occurred");
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [scheduleType, dateFrom, dateTo]);

  const handleFilter = (e: React.FormEvent) => {
    e.preventDefault();
    // The useEffect will handle the data fetching when state changes
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        Loading transport schedule data...
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center flex-col gap-4">
        <div className="text-red-500 text-xl">Error: {error}</div>
        <div className="text-gray-500">
          Make sure your Moose server is running with{" "}
          <code className="bg-gray-100 px-2 py-1 rounded">moose dev</code>
        </div>
      </div>
    );
  }

  const emptyData = !data || data.summary.totalSchedules === 0;

  return (
    <div className="p-6">
      <header className="mb-6">
        <h1 className="text-2xl font-bold mb-2">
          Transport Schedule Dashboard
        </h1>
        <p className="text-gray-500">Analyzing transport schedule data</p>
      </header>

      <div className="bg-white p-4 rounded-lg shadow mb-6">
        <h2 className="text-lg font-semibold mb-3">Filter Schedules</h2>
        <form
          onSubmit={handleFilter}
          className="grid grid-cols-1 md:grid-cols-3 gap-4"
        >
          <div>
            <label
              htmlFor="scheduleType"
              className="block text-sm font-medium text-gray-700 mb-1"
            >
              Schedule Type
            </label>
            <input
              type="text"
              id="scheduleType"
              className="w-full border border-gray-300 rounded px-3 py-2"
              value={scheduleType}
              onChange={(e) => setScheduleType(e.target.value)}
              placeholder="Enter schedule type"
            />
          </div>
          <div>
            <label
              htmlFor="dateFrom"
              className="block text-sm font-medium text-gray-700 mb-1"
            >
              From Date
            </label>
            <input
              type="date"
              id="dateFrom"
              className="w-full border border-gray-300 rounded px-3 py-2"
              value={dateFrom}
              onChange={(e) => setDateFrom(e.target.value)}
            />
          </div>
          <div>
            <label
              htmlFor="dateTo"
              className="block text-sm font-medium text-gray-700 mb-1"
            >
              To Date
            </label>
            <input
              type="date"
              id="dateTo"
              className="w-full border border-gray-300 rounded px-3 py-2"
              value={dateTo}
              onChange={(e) => setDateTo(e.target.value)}
            />
          </div>
          <div className="md:col-span-3">
            <button
              type="submit"
              className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
            >
              Apply Filters
            </button>
          </div>
        </form>
      </div>

      {emptyData ? (
        <div className="bg-white p-8 rounded-lg shadow text-center">
          <h2 className="text-xl font-bold mb-4">
            No Transport Schedule Data Available
          </h2>
          <p className="text-gray-500 mb-4">
            The transport schedule table appears to be empty. Once data is
            added, you'll be able to see detailed statistics and analysis here.
          </p>
        </div>
      ) : (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard
              title="Total Schedules"
              value={data.summary.totalSchedules}
              description="Transport schedules in the database"
              icon="🚆"
            />
            <StatCard
              title="Avg Transit Time"
              value={`${data.transitStats.avgTransitMinutes} min`}
              description="Average journey duration"
              icon="⏱️"
            />
            <StatCard
              title="Min Transit Time"
              value={`${data.transitStats.minTransitMinutes} min`}
              description="Shortest journey time"
              icon="⏳"
            />
            <StatCard
              title="Max Transit Time"
              value={`${data.transitStats.maxTransitMinutes} min`}
              description="Longest journey time"
              icon="⌛"
            />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
            <section className="bg-white p-4 rounded-lg shadow">
              <h2 className="text-xl font-bold mb-4">Schedule Types</h2>
              <div className="overflow-x-auto">
                <table className="min-w-full">
                  <thead>
                    <tr className="bg-gray-100">
                      <th className="px-4 py-2 text-left">Type</th>
                      <th className="px-4 py-2 text-right">Count</th>
                      <th className="px-4 py-2 text-right">Percentage</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.scheduleTypes.map((type, index) => (
                      <tr
                        key={index}
                        className={index % 2 === 0 ? "bg-gray-50" : ""}
                      >
                        <td className="px-4 py-2">{type.scheduleType}</td>
                        <td className="px-4 py-2 text-right">
                          {type.scheduleCount}
                        </td>
                        <td className="px-4 py-2 text-right">
                          {type.percentage}%
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>

            <section className="bg-white p-4 rounded-lg shadow">
              <h2 className="text-xl font-bold mb-4">
                Time of Day Distribution
              </h2>
              <div className="overflow-x-auto">
                <table className="min-w-full">
                  <thead>
                    <tr className="bg-gray-100">
                      <th className="px-4 py-2 text-left">Time Period</th>
                      <th className="px-4 py-2 text-right">Schedule Count</th>
                      <th className="px-4 py-2">Distribution</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.timeOfDay.map((time, index) => (
                      <tr
                        key={index}
                        className={index % 2 === 0 ? "bg-gray-50" : ""}
                      >
                        <td className="px-4 py-2">{time.timeOfDay}</td>
                        <td className="px-4 py-2 text-right">
                          {time.scheduleCount}
                        </td>
                        <td className="px-4 py-2">
                          <div className="w-full bg-gray-200 rounded-full h-2.5">
                            <div
                              className="bg-blue-600 h-2.5 rounded-full"
                              style={{
                                width: `${(time.scheduleCount / Math.max(...data.timeOfDay.map((t) => t.scheduleCount))) * 100}%`,
                              }}
                            ></div>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          </div>

          <section className="bg-white p-4 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Top Routes</h2>
            <div className="overflow-x-auto">
              <table className="min-w-full">
                <thead>
                  <tr className="bg-gray-100">
                    <th className="px-4 py-2 text-left">Origin</th>
                    <th className="px-4 py-2 text-left">Destination</th>
                    <th className="px-4 py-2 text-right">Schedule Count</th>
                  </tr>
                </thead>
                <tbody>
                  {data.topRoutes.map((route, index) => (
                    <tr
                      key={index}
                      className={index % 2 === 0 ? "bg-gray-50" : ""}
                    >
                      <td className="px-4 py-2">{route.origin}</td>
                      <td className="px-4 py-2">{route.destination}</td>
                      <td className="px-4 py-2 text-right">
                        {route.scheduleCount}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>
        </>
      )}
    </div>
  );
}

function StatCard({
  title,
  value,
  description,
  icon,
}: {
  title: string;
  value: number | string;
  description: string;
  icon: string;
}) {
  return (
    <div className="bg-white p-4 rounded-lg shadow">
      <div className="flex items-start">
        <div className="p-2 bg-blue-100 rounded-lg text-2xl mr-3">{icon}</div>
        <div>
          <h3 className="font-bold text-lg">{title}</h3>
          <p className="text-2xl font-semibold">{value}</p>
          <p className="text-gray-500 text-sm">{description}</p>
        </div>
      </div>
    </div>
  );
}
