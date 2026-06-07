import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { Telemetry } from "../types";

interface Series {
  key: keyof Telemetry;
  label: string;
  color: string;
  dashed?: boolean;
}

interface TelemetryChartProps {
  title: string;
  subtitle: string;
  data: Telemetry[];
  series: Series[];
  unit: string;
  domain?: [number | "auto", number | "auto"];
}

export default function TelemetryChart({
  title,
  subtitle,
  data,
  series,
  unit,
  domain = ["auto", "auto"],
}: TelemetryChartProps) {
  const latest = data.at(-1);

  return (
    <section className="panel chart-panel">
      <div className="panel-heading compact">
        <div>
          <h3>{title}</h3>
          <span>{subtitle}</span>
        </div>
        <div className="chart-current">
          {series.map((item) => (
            <span key={item.key} style={{ "--series": item.color } as React.CSSProperties}>
              <i />
              {latest ? Number(latest[item.key]).toFixed(1) : "--"} {unit}
            </span>
          ))}
        </div>
      </div>
      <div className="chart-wrap">
        {data.length > 1 ? (
          <ResponsiveContainer
            width="100%"
            height="100%"
            minWidth={0}
            initialDimension={{ width: 600, height: 190 }}
          >
            <LineChart data={data}>
              <CartesianGrid stroke="#24313b" strokeDasharray="3 6" vertical={false} />
              <XAxis
                dataKey="elapsed_seconds"
                stroke="#657480"
                tickLine={false}
                axisLine={false}
                tick={{ fontSize: 10 }}
                tickFormatter={(value) => `${Math.round(value)}s`}
              />
              <YAxis
                domain={domain}
                stroke="#657480"
                tickLine={false}
                axisLine={false}
                tick={{ fontSize: 10 }}
                width={38}
              />
              <Tooltip
                contentStyle={{
                  background: "#10181e",
                  border: "1px solid #32414c",
                  borderRadius: 6,
                  fontSize: 12,
                }}
                labelFormatter={(value) => `${Number(value).toFixed(1)} seconds`}
              />
              {series.map((item) => (
                <Line
                  key={item.key}
                  type="monotone"
                  dataKey={item.key}
                  name={item.label}
                  stroke={item.color}
                  strokeWidth={item.dashed ? 1.5 : 2}
                  strokeDasharray={item.dashed ? "5 5" : undefined}
                  dot={false}
                  isAnimationActive={false}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <div className="chart-empty">
            <span className="scan-line" />
            <p>Waiting for live sensor data</p>
          </div>
        )}
      </div>
    </section>
  );
}
