'use client'

import { useMemo } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis, Legend, ResponsiveContainer } from "recharts"

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import {
  ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent
} from "@/components/ui/chart"


interface GraphDataPayload {
  timestamp: string
  [username: string]: number | string
}

interface HeartRateGraphProps {
  userData: GraphDataPayload[]
  uniqueUsers: Set<string>
}


const userColors = [
  "hsl(var(--chart-1))",
  "hsl(var(--chart-2))",
  "hsl(var(--chart-3))",
  "hsl(var(--chart-4))",
  "hsl(var(--chart-5))",
]

export default function HeartRateGraph({ userData , uniqueUsers }: HeartRateGraphProps) {
  // Only trigger when uniqueUsers changes
  // Sensors can be added live 

  const chartConfig = useMemo(() => {
    const config: ChartConfig = {}
    Array.from(uniqueUsers).forEach((user, index) => {
      config[user] = {
        label: user,
        color: userColors[index % userColors.length],
      }
    })
    return config
  }, [uniqueUsers])


  const calculateXDomain = () => {
    if (userData.length === 0) {
      return []
    }
    const firstTimestamp = userData[0].timestamp
    const lastTimestamp = userData[userData.length - 1].timestamp
    return [firstTimestamp.slice(14, 19), lastTimestamp.slice(14, 19)]
  }

  return (
    <Card className="bg-black text-white border-0 h-full">
      <CardContent className="p-0">
        <div className="text-center text-sm text-gray-400 mb-2">
          {userData.length > 0 && (
            <span>
            {userData[userData.length - 1].timestamp.slice(14, 19)}
            </span>
          )}
        </div>
        <ChartContainer config={chartConfig}>
          <ResponsiveContainer width="100%" height={200}>
            <LineChart
              data={userData}
              margin={{
                top: 5,
                right: 10,
                left: 0,
                bottom: 5,
              }}
            >
              <CartesianGrid strokeDasharray="3 3" stroke="rgba(255, 255, 255, 0.1)" />
              <XAxis
                // domain={calculateXDomain()}
                dataKey="timestamp"
                type="category"
                ticks={calculateXDomain()}
                // tick={{ fill: 'rgba(255, 255, 255, 0.7)', fontSize: 12 }}
                axisLine={{ stroke: 'rgba(255, 255, 255, 0.3)' }}
              />
              <YAxis 
                domain={[0, 220]}
                ticks={[0, 50, 100, 150, 200]}
                tick={{ fill: 'rgba(255, 255, 255, 0.7)', fontSize: 12 }}
                tickFormatter={(value) => `${value}`}
                axisLine={{ stroke: 'rgba(255, 255, 255, 0.3)' }}
              />
              {Array.from(uniqueUsers).map((user, index) => (
                <Line
                  key={user}
                  type="monotoneX"
                  dataKey={user}
                  name={user}
                  stroke={userColors[index % userColors.length]}
                  dot={false}
                  isAnimationActive={false}
                  connectNulls={true}
                  strokeWidth={2}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}
