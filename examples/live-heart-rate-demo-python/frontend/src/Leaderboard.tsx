type MetricType = "HEART RATE" | "HEART RATE ZONES" | "POWER" | "CALORIES";

interface LeaderboardEntry {
  name: string;
  value: number;
  metric: MetricType;
}

export default function Leaderboard({
  leaderboard,
}: {
  leaderboard: LeaderboardEntry[];
}) {
  // TODO: Get this from SUPABASE
  const userLookup = {
    Joj: "https://ca.slack-edge.com/T062WLJGPMW-U071QSWGQAK-dc420b7ca59c-512",
    Olivia:
      "https://ca.slack-edge.com/T062WLJGPMW-U06ESHX61R7-466642c6a419-512",
    Alex: "https://ca.slack-edge.com/T062WLJGPMW-U0624P1RC0M-a52dbc57c8a8-512",
    Samia: "https://ca.slack-edge.com/T062WLJGPMW-U06USNZMB45-6fcb4c2dfd19-512",
    Tim: "https://ca.slack-edge.com/T062WLJGPMW-U062A4HPG4C-f7dfbbfa5065-512",
    Arman: "https://ca.slack-edge.com/T062WLJGPMW-U079W7WJ8MV-31c7dff99db9-512",
    Chris: "https://ca.slack-edge.com/T062WLJGPMW-U06DCH974P9-6f641aeed2f5-512",
  };
  return (
    <div className="leaderboard h-full bg-black rounded-lg shadow-md p-6">
      <ul className="space-y-4">
        {leaderboard.map((entry, index) => (
          <li key={index} className="flex items-center">
            <img
              src={
                userLookup[entry.name as keyof typeof userLookup] ||
                "https://via.placeholder.com/40"
              }
              alt={entry.name}
              className="w-10 h-10 rounded-full mr-4"
            />
            <div className="flex-grow">
              <span className="font-semibold text-lg text-white">
                {entry.name}
              </span>
              <span className="block text-sm text-gray-400">
                {entry.value} {entry.metric == "HEART RATE" ? "BPM" : ""}
              </span>
            </div>
            <span
              className={`w-3 h-3 rounded-full ${index === 0 ? "bg-green-500" : "bg-gray-500"}`}
            ></span>
          </li>
        ))}
      </ul>
    </div>
  );
}
