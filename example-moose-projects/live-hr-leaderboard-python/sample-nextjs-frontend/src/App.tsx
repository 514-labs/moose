import HeartRateGraph from "./HeartRateGraph";
import Leaderboard from "./Leaderboard";

import { User, UserPlus, Home, FileText, Activity, Menu } from "lucide-react";
import { useState, useEffect } from "react";

type MetricType = "HEART RATE" | "HEART RATE ZONES" | "POWER" | "CALORIES";

interface GraphDataPayload {
  timestamp: string;
  [username: string]: number | string;
}

interface MooseAPIResponse {
  user_name: string;
  avg_heart_rate: number;
  last_processed_timestamp: string;
}

interface LeaderboardEntry {
  name: string;
  value: number;
  metric: MetricType;
}

interface CoachingResponse {
  coaching_message: string;
  analysis_message: string;
}

export default function App() {
  const [selectedMetric, setSelectedMetric] =
    useState<MetricType>("HEART RATE");
  const metrics: MetricType[] = [
    "HEART RATE",
    "HEART RATE ZONES",
    "POWER",
    "CALORIES",
  ];
  const [graphData, setGraphData] = useState<GraphDataPayload[]>([]);
  const [uniqueUsers, setUniqueUsers] = useState<Set<string>>(new Set());
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [coachingMessage, setCoachingMessage] = useState<string>("");
  const [analysisMessage, setAnalysisMessage] = useState<string>("");
  // const newLeaderboard: LeaderboardEntry[] = [];

  const handleMetricSelect = (metric: MetricType) => {
    setSelectedMetric(metric);
    console.log(`Selected metric: ${metric}`);
  };

  useEffect(() => {
    const fetchData = async () => {
      try {
        const timestamp = new Date().toISOString();
        const response = await fetch(
          `http://localhost:4000/consumption/getHR?timestamp=${timestamp}`,
        );
        const mooseAPIResponse: MooseAPIResponse[] = await response.json();

        if (mooseAPIResponse.length > 0) {
          setGraphData((prevData) => {
            const newDataPoint: GraphDataPayload = { timestamp };

            mooseAPIResponse.forEach((entry: MooseAPIResponse) => {
              // Round the heart rate to the nearest integer
              newDataPoint[entry.user_name] = Math.round(entry.avg_heart_rate);
              setUniqueUsers((prev) => new Set(prev).add(entry.user_name));
            });
            // Send the last 60 data points to the graph
            return [...prevData, newDataPoint].slice(-60);
          });
        } else {
          console.log("No data received from Moose API");
        }
      } catch (error) {
        console.error("Error fetching heart rate data:", error);
      }
    };

    const intervalId = setInterval(fetchData, 1000); // Poll every second

    return () => clearInterval(intervalId);
  }, []);

  useEffect(() => {
    // Update the leaderboard based on the selected metric
    const latestDataPoint: GraphDataPayload = graphData[graphData.length - 1];
    if (latestDataPoint) {
      const newLeaderboard: LeaderboardEntry[] = Object.entries(latestDataPoint)
        .filter(
          ([username, value]) =>
            typeof value === "number" && username !== "timestamp",
        )
        .map(([username, value]) => ({
          name: username,
          value: value as number,
          metric: selectedMetric,
        }))
        .sort((a, b) => b.value - a.value);

      // Update the leaderboard state
      setLeaderboard(newLeaderboard);
    } else {
      console.log("No data to update leaderboard");
    }
  }, [graphData, selectedMetric]);

  useEffect(() => {
    const fetchCoaching = async () => {
      try {
        // Hardcoded analysis window of 5 seconds
        const response = await fetch(
          `http://localhost:4000/consumption/getCoaching`,
        );
        const coachingData: CoachingResponse = await response.json();
        setCoachingMessage(coachingData.coaching_message);
        setAnalysisMessage(coachingData.analysis_message);
      } catch (error) {
        console.error("Error fetching coaching data:", error);
      }
    };

    // Set up interval for polling every 30 seconds
    const intervalId = setInterval(fetchCoaching, 30000);

    // Clean up interval on component unmount
    return () => clearInterval(intervalId);
  }, []);

  console.log(coachingMessage);
  console.log(analysisMessage);
  return (
    <div className="w-screen h-screen flex items-center justify-center">
      <div className="iphone-frame bg-black text-white flex flex-col min-w-[380px] rounded-[40px] shadow-xl mx-auto">
        <div className="app-container p-6 flex-grow flex flex-col my-4">
          <header className="mb-4 flex justify-between items-center">
            <div className="flex justify-between items-center w-full">
              <User className="w-6 h-6" />
              <h1 className="text-xl font-semibold">INDOOR CYCLING</h1>
              <UserPlus className="w-6 h-6" />
            </div>
          </header>
          <HeartRateGraph userData={graphData} uniqueUsers={uniqueUsers} />
          <div className="flex justify-between mt-4 mb-6">
            {metrics.map((metric) => (
              <button
                key={metric}
                className={`px-4 py-2 rounded-full font-semibold ${
                  selectedMetric === metric
                    ? "bg-gray-800 text-white"
                    : "text-gray-400"
                }`}
                onClick={() => handleMetricSelect(metric)}
                disabled={selectedMetric === metric}
              >
                {metric}
              </button>
            ))}
          </div>
          <div className="flex-grow">
            <Leaderboard leaderboard={leaderboard} />
          </div>
        </div>
        {/* {(coachingMessage || analysisMessage) && (
      <div className="px-6 pb-4 max-w-[380px]">
        {coachingMessage && (
          <div className="bg-gray-800 rounded-lg p-3 mb-2 text-center">
            <h3 className="text-sm font-semibold text-gray-400 mb-1">Coaching</h3>
            <p className="text-sm">{coachingMessage}</p>
          </div>
        )}
        {analysisMessage && (
          <div className="bg-gray-800 rounded-lg p-3">
            <h3 className="text-sm font-semibold text-gray-400 mb-1">Analysis</h3>
            <p className="text-sm">{analysisMessage}</p>
          </div>
        )}
      </div>
    )} */}
        <nav className="flex justify-around items-center py-4">
          <Home className="w-6 h-6" />
          <FileText className="w-6 h-6" />
          <Activity className="w-6 h-6" />
          <Menu className="w-6 h-6" />
        </nav>
      </div>
    </div>
  );
}
