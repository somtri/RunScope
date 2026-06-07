import { ChevronRight } from "lucide-react";
import type { RunSummary } from "../types";

interface RunHistoryTableProps {
  runs: RunSummary[];
  selectedId?: string;
  onSelect?: (run: RunSummary) => void;
}

const formatDuration = (seconds: number | null) => {
  if (seconds === null) return "--";
  const minutes = Math.floor(seconds / 60);
  const remaining = Math.round(seconds % 60);
  return `${minutes}:${remaining.toString().padStart(2, "0")}`;
};

export default function RunHistoryTable({
  runs,
  selectedId,
  onSelect,
}: RunHistoryTableProps) {
  return (
    <div className="history-table-wrap">
      <table className="history-table">
        <thead>
          <tr>
            <th>Run</th>
            <th>Status</th>
            <th>Duration</th>
            <th>Peak plate</th>
            <th>Min O2</th>
            <th>Peak spatter</th>
            <th>Alerts</th>
            <th>Quality</th>
            {onSelect && <th aria-label="Open run" />}
          </tr>
        </thead>
        <tbody>
          {runs.map((run, index) => (
            <tr
              key={run.id}
              className={selectedId === run.id ? "selected" : ""}
              onClick={() => onSelect?.(run)}
            >
              <td>
                <strong>RUN-{String(runs.length - index).padStart(3, "0")}</strong>
                <span>
                  {new Date(run.started_at).toLocaleDateString(undefined, {
                    month: "short",
                    day: "numeric",
                  })}{" "}
                  {new Date(run.started_at).toLocaleTimeString([], {
                    hour: "2-digit",
                    minute: "2-digit",
                  })}
                </span>
              </td>
              <td>
                <span className={`status-pill status-${run.status}`}>
                  <i />
                  {run.status}
                </span>
              </td>
              <td>{formatDuration(run.duration_seconds)}</td>
              <td>
                {run.max_build_plate_temperature_c?.toFixed(1) ?? "--"}
                <small> C</small>
              </td>
              <td>
                {run.min_chamber_oxygen_ppm?.toFixed(0) ?? "--"}
                <small> ppm</small>
              </td>
              <td>
                {run.max_spatter_rate_per_s?.toFixed(1) ?? "--"}
                <small> /s</small>
              </td>
              <td>{run.alert_count}</td>
              <td>
                <strong>{run.final_quality_score?.toFixed(1) ?? "--"}</strong>
                <small> %</small>
              </td>
              {onSelect && (
                <td>
                  <ChevronRight size={16} />
                </td>
              )}
            </tr>
          ))}
        </tbody>
      </table>
      {runs.length === 0 && (
        <div className="table-empty">No saved runs yet. Start an experiment to create one.</div>
      )}
    </div>
  );
}
