"use client";

import { useEffect, useState } from "react";

interface RackUtilization {
  rack_location: string;
  avg_utilization: number;
}

export default function RackUtilizationPage() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<RackUtilization[]>([]);
  const [minUtilization, setMinUtilization] = useState<number>(0);
  const [maxUtilization, setMaxUtilization] = useState<number>(100);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);

        let url = "http://localhost:4000/consumption/getPanelUtilizationByRack";
        const params = new URLSearchParams();

        if (minUtilization > 0) {
          params.append("minUtilization", minUtilization.toString());
        }

        if (maxUtilization < 100) {
          params.append("maxUtilization", maxUtilization.toString());
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
  }, [minUtilization, maxUtilization]);

  const handleFilter = (e: React.FormEvent) => {
    e.preventDefault();
    // The useEffect will handle the data fetching when state changes
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        Loading rack utilization data...
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

  if (!data || data.length === 0) {
    return (
      <div className="p-6">
        <header className="mb-6">
          <h1 className="text-2xl font-bold mb-2">
            Rack Utilization Dashboard
          </h1>
          <p className="text-gray-500">
            Analyzing fiber panel utilization by rack location
          </p>
        </header>

        <div className="bg-white p-4 rounded-lg shadow mb-6">
          <h2 className="text-lg font-semibold mb-3">Filter by Utilization</h2>
          <form
            onSubmit={handleFilter}
            className="grid grid-cols-1 md:grid-cols-2 gap-4"
          >
            <div>
              <label
                htmlFor="minUtilization"
                className="block text-sm font-medium text-gray-700 mb-1"
              >
                Min Utilization (%)
              </label>
              <input
                type="number"
                id="minUtilization"
                min="0"
                max="100"
                className="w-full border border-gray-300 rounded px-3 py-2"
                value={minUtilization}
                onChange={(e) => setMinUtilization(Number(e.target.value))}
              />
            </div>
            <div>
              <label
                htmlFor="maxUtilization"
                className="block text-sm font-medium text-gray-700 mb-1"
              >
                Max Utilization (%)
              </label>
              <input
                type="number"
                id="maxUtilization"
                min="0"
                max="100"
                className="w-full border border-gray-300 rounded px-3 py-2"
                value={maxUtilization}
                onChange={(e) => setMaxUtilization(Number(e.target.value))}
              />
            </div>
            <div className="md:col-span-2">
              <button
                type="submit"
                className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
              >
                Apply Filters
              </button>
            </div>
          </form>
        </div>

        <div className="bg-white p-8 rounded-lg shadow text-center">
          <h2 className="text-xl font-bold mb-4">
            No Rack Utilization Data Available
          </h2>
          <p className="text-gray-500 mb-4">
            No rack utilization data matching your filter criteria. Try
            adjusting the utilization range.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      <header className="mb-6">
        <h1 className="text-2xl font-bold mb-2">Rack Utilization Dashboard</h1>
        <p className="text-gray-500">
          Analyzing fiber panel utilization by rack location
        </p>
      </header>

      <div className="bg-white p-4 rounded-lg shadow mb-6">
        <h2 className="text-lg font-semibold mb-3">Filter by Utilization</h2>
        <form
          onSubmit={handleFilter}
          className="grid grid-cols-1 md:grid-cols-2 gap-4"
        >
          <div>
            <label
              htmlFor="minUtilization"
              className="block text-sm font-medium text-gray-700 mb-1"
            >
              Min Utilization (%)
            </label>
            <input
              type="number"
              id="minUtilization"
              min="0"
              max="100"
              className="w-full border border-gray-300 rounded px-3 py-2"
              value={minUtilization}
              onChange={(e) => setMinUtilization(Number(e.target.value))}
            />
          </div>
          <div>
            <label
              htmlFor="maxUtilization"
              className="block text-sm font-medium text-gray-700 mb-1"
            >
              Max Utilization (%)
            </label>
            <input
              type="number"
              id="maxUtilization"
              min="0"
              max="100"
              className="w-full border border-gray-300 rounded px-3 py-2"
              value={maxUtilization}
              onChange={(e) => setMaxUtilization(Number(e.target.value))}
            />
          </div>
          <div className="md:col-span-2">
            <button
              type="submit"
              className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
            >
              Apply Filters
            </button>
          </div>
        </form>
      </div>

      <div className="grid grid-cols-1 gap-6">
        <section className="bg-white p-4 rounded-lg shadow">
          <h2 className="text-xl font-bold mb-4">Rack Utilization Overview</h2>
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="bg-gray-100">
                  <th className="px-4 py-2 text-left">Rack Location</th>
                  <th className="px-4 py-2 text-right">Average Utilization</th>
                  <th className="px-4 py-2">Utilization Bar</th>
                </tr>
              </thead>
              <tbody>
                {data.map((rack, index) => (
                  <tr
                    key={index}
                    className={index % 2 === 0 ? "bg-gray-50" : ""}
                  >
                    <td className="px-4 py-2 font-medium">
                      {rack.rack_location}
                    </td>
                    <td className="px-4 py-2 text-right">
                      {rack.avg_utilization.toFixed(2)}%
                    </td>
                    <td className="px-4 py-2">
                      <div className="w-full bg-gray-200 rounded-full h-2.5">
                        <div
                          className={`h-2.5 rounded-full ${getUtilizationColorClass(rack.avg_utilization)}`}
                          style={{ width: `${rack.avg_utilization}%` }}
                        ></div>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="bg-white p-4 rounded-lg shadow">
          <h2 className="text-xl font-bold mb-4">Utilization Distribution</h2>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mt-4">
            <UtilizationCard
              title="Low Utilization"
              range="0-25%"
              count={data.filter((r) => r.avg_utilization < 25).length}
              total={data.length}
              colorClass="bg-green-500"
            />
            <UtilizationCard
              title="Medium Utilization"
              range="25-50%"
              count={
                data.filter(
                  (r) => r.avg_utilization >= 25 && r.avg_utilization < 50,
                ).length
              }
              total={data.length}
              colorClass="bg-blue-500"
            />
            <UtilizationCard
              title="High Utilization"
              range="50-75%"
              count={
                data.filter(
                  (r) => r.avg_utilization >= 50 && r.avg_utilization < 75,
                ).length
              }
              total={data.length}
              colorClass="bg-yellow-500"
            />
            <UtilizationCard
              title="Critical Utilization"
              range="75-100%"
              count={data.filter((r) => r.avg_utilization >= 75).length}
              total={data.length}
              colorClass="bg-red-500"
            />
          </div>
        </section>
      </div>
    </div>
  );
}

function getUtilizationColorClass(utilization: number): string {
  if (utilization < 25) return "bg-green-500";
  if (utilization < 50) return "bg-blue-500";
  if (utilization < 75) return "bg-yellow-500";
  return "bg-red-500";
}

interface UtilizationCardProps {
  title: string;
  range: string;
  count: number;
  total: number;
  colorClass: string;
}

function UtilizationCard({
  title,
  range,
  count,
  total,
  colorClass,
}: UtilizationCardProps) {
  const percentage = total > 0 ? Math.round((count / total) * 100) : 0;

  return (
    <div className="bg-gray-50 p-4 rounded-lg border">
      <h3 className="text-lg font-semibold">{title}</h3>
      <p className="text-gray-500 text-sm">Range: {range}</p>
      <div className="mt-2">
        <div className="flex justify-between text-sm mb-1">
          <span>{count} racks</span>
          <span>{percentage}%</span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-2.5">
          <div
            className={`h-2.5 rounded-full ${colorClass}`}
            style={{ width: `${percentage}%` }}
          ></div>
        </div>
      </div>
    </div>
  );
}
