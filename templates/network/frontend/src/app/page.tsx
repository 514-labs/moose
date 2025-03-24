"use client";

import { useEffect, useState } from "react";
import Image from "next/image";

interface PanelSummary {
  panelCount: number;
  totalPorts: number;
  connectedPorts: number;
  utilizationRate: number;
}

interface PanelDetail {
  rackLocation: string;
  panelName: string;
  connectedPorts: number;
  totalPorts: number;
  utilizationRate: number;
}

interface ConnectionType {
  connectionType: string;
  connectionCount: number;
  percentage: number;
}

interface TopPanel {
  rackLocation: string;
  panelName: string;
  connectionCount: number;
}

interface PanelSummaryResponse {
  summary: PanelSummary;
  panels: PanelDetail[];
  connectionTypes: ConnectionType[];
  topPanels: TopPanel[];
}

export default function Home() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<PanelSummaryResponse | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        const response = await fetch(
          "http://localhost:4000/consumption/getFiberPanelSummary",
        );
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
  }, []);

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        Loading network data...
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

  if (!data) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        No data available
      </div>
    );
  }

  return (
    <div className="min-h-screen p-6">
      <header className="mb-6">
        <h1 className="text-2xl font-bold mb-2">
          Network Fiber Panel Dashboard
        </h1>
        <p className="text-gray-500">
          Displaying data from the fiber panel inventory system
        </p>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard
          title="Total Panels"
          value={data.summary.panelCount}
          description="Fiber panels in the network"
          icon="📊"
        />
        <StatCard
          title="Total Ports"
          value={data.summary.totalPorts}
          description="Available fiber ports"
          icon="🔌"
        />
        <StatCard
          title="Connected Ports"
          value={data.summary.connectedPorts}
          description="Ports with active connections"
          icon="🔗"
        />
        <StatCard
          title="Utilization Rate"
          value={`${data.summary.utilizationRate}%`}
          description="Overall port utilization"
          icon="📈"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <section className="bg-white p-4 rounded-lg shadow">
          <h2 className="text-xl font-bold mb-4">Connection Types</h2>
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
                {data.connectionTypes.map((type, index) => (
                  <tr
                    key={index}
                    className={index % 2 === 0 ? "bg-gray-50" : ""}
                  >
                    <td className="px-4 py-2">{type.connectionType}</td>
                    <td className="px-4 py-2 text-right">
                      {type.connectionCount}
                    </td>
                    <td className="px-4 py-2 text-right">{type.percentage}%</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="bg-white p-4 rounded-lg shadow">
          <h2 className="text-xl font-bold mb-4">Top Panels by Connections</h2>
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr className="bg-gray-100">
                  <th className="px-4 py-2 text-left">Rack Location</th>
                  <th className="px-4 py-2 text-left">Panel Name</th>
                  <th className="px-4 py-2 text-right">Connections</th>
                </tr>
              </thead>
              <tbody>
                {data.topPanels.map((panel, index) => (
                  <tr
                    key={index}
                    className={index % 2 === 0 ? "bg-gray-50" : ""}
                  >
                    <td className="px-4 py-2">{panel.rackLocation}</td>
                    <td className="px-4 py-2">{panel.panelName}</td>
                    <td className="px-4 py-2 text-right">
                      {panel.connectionCount}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      </div>

      <section className="mt-6 bg-white p-4 rounded-lg shadow">
        <h2 className="text-xl font-bold mb-4">Panel Utilization</h2>
        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="bg-gray-100">
                <th className="px-4 py-2 text-left">Rack Location</th>
                <th className="px-4 py-2 text-left">Panel Name</th>
                <th className="px-4 py-2 text-right">Connected</th>
                <th className="px-4 py-2 text-right">Total</th>
                <th className="px-4 py-2 text-right">Utilization</th>
                <th className="px-4 py-2">Utilization Bar</th>
              </tr>
            </thead>
            <tbody>
              {data.panels.map((panel, index) => (
                <tr key={index} className={index % 2 === 0 ? "bg-gray-50" : ""}>
                  <td className="px-4 py-2">{panel.rackLocation}</td>
                  <td className="px-4 py-2">{panel.panelName}</td>
                  <td className="px-4 py-2 text-right">
                    {panel.connectedPorts}
                  </td>
                  <td className="px-4 py-2 text-right">{panel.totalPorts}</td>
                  <td className="px-4 py-2 text-right">
                    {panel.utilizationRate}%
                  </td>
                  <td className="px-4 py-2">
                    <div className="w-full bg-gray-200 rounded-full h-2.5">
                      <div
                        className="bg-blue-600 h-2.5 rounded-full"
                        style={{ width: `${panel.utilizationRate}%` }}
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
