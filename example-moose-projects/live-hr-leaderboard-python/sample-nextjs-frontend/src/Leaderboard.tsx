import mockUsers from "../../mock-user-db.json";

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
  // Create userLookup from mock data
  // Realistically, this would be fetched from a transactional database
  const userLookup: { [key: string]: string } = Object.values(mockUsers).reduce(
    (acc: { [key: string]: string }, user: any) => {
      acc[user.user_name] = user.profile_image;
      return acc;
    },
    {},
  );

  console.log(userLookup);

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
