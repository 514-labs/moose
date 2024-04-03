import { PlotOptions, barY, ruleY } from "@observablehq/plot";
import PlotComponent from "./ui/plot-react";

interface Props {
  data: object[];
  toolbar: React.ReactElement;
  breakdown: string[];
}

function createDimensions(breakdown: string[]) {
  return {
    ...(breakdown?.[0] && { fx: breakdown[0] }),
    ...(breakdown?.[1] && { fy: breakdown[1] }),
  };
}
export default function FunnelChart({ data, toolbar, breakdown }: Props) {
  const options: PlotOptions = {
    y: {
      grid: true,
    },
    marks: [
      barY(data, {
        y: "count",
        x: "level",
        fill: breakdown?.[0] || "white",
        tip: { fill: "black" },
        ...createDimensions(breakdown),
      }),
      ruleY([0]),
    ],
  };

  return (
    <div className="w-full h-full flex flex-col">
      <div className="h-12 justify-end flex w-full p-2">{toolbar}</div>
      <div className="flex-1 overflow-hidden m-4">
        {data?.[0] ? (
          <PlotComponent options={options} />
        ) : (
          <div className="flex items-center justify-center h-full text-center">
            No Data in Time Range for selected events
          </div>
        )}
      </div>
    </div>
  );
}
