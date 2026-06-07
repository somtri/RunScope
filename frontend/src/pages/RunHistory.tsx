import { useEffect, useState } from "react";
import { ArrowLeft, Database, RefreshCw, TriangleAlert } from "lucide-react";
import { api } from "../api/client";
import AlertLog from "../components/AlertLog";
import RunHistoryTable from "../components/RunHistoryTable";
import TelemetryChart from "../components/TelemetryChart";
import type {
  RunDetail,
  RunSummary,
  StoredTelemetry,
} from "../types";

interface RunHistoryProps {
  onBack: () => void;
}

export default function RunHistory({ onBack }: RunHistoryProps) {
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [selected, setSelected] = useState<RunDetail | null>(null);
  const [telemetry, setTelemetry] = useState<StoredTelemetry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const loadRuns = async () => {
    setLoading(true);
    setError("");
    try {
      const history = await api.getRuns();
      setRuns(history);
      if (history[0]) {
        await selectRun(history[0]);
      } else {
        setSelected(null);
        setTelemetry([]);
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Unable to load history");
    } finally {
      setLoading(false);
    }
  };

  const selectRun = async (run: RunSummary) => {
    setError("");
    try {
      const [detail, samples] = await Promise.all([
        api.getRun(run.id),
        api.getTelemetry(run.id),
      ]);
      setSelected(detail);
      setTelemetry(samples);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Unable to load run");
    }
  };

  useEffect(() => {
    void loadRuns();
  }, []);

  return (
    <div className="page history-page">
      <header className="page-header">
        <div>
          <button className="back-button" onClick={onBack}>
            <ArrowLeft size={15} />
            Live monitor
          </button>
          <span className="eyebrow">Persistent experiment records</span>
          <h1>Run history</h1>
          <p>Review layer outcomes, detected anomalies, and stored telemetry.</p>
        </div>
        <button className="button button-secondary" onClick={loadRuns} disabled={loading}>
          <RefreshCw className={loading ? "spin" : ""} size={16} />
          Refresh
        </button>
      </header>

      {error && (
        <div className="error-banner">
          <TriangleAlert size={16} />
          {error}
        </div>
      )}

      <section className="panel history-overview">
        <div className="panel-heading">
          <div>
            <span className="eyebrow">SQLite run registry</span>
            <h2>Experiment records</h2>
          </div>
          <span className="history-count">
            <Database size={15} />
            {runs.length} records
          </span>
        </div>
        <RunHistoryTable
          runs={runs}
          selectedId={selected?.run.id}
          onSelect={selectRun}
        />
      </section>

      {selected && (
        <section className="history-detail">
          <div className="detail-heading">
            <div>
              <span className="eyebrow">Selected run</span>
              <h2>{selected.run.recipe_name}</h2>
              <code>{selected.run.id}</code>
            </div>
            <span className={`status-pill status-${selected.run.status}`}>
              <i />
              {selected.run.status}
            </span>
          </div>

          <div className="history-stat-grid">
            <div>
              <span>Duration</span>
              <strong>{selected.run.duration_seconds?.toFixed(1) ?? "--"} s</strong>
            </div>
            <div>
              <span>Peak plate temp.</span>
              <strong>{selected.run.max_build_plate_temperature_c?.toFixed(1) ?? "--"} C</strong>
            </div>
            <div>
              <span>Minimum oxygen</span>
              <strong>{selected.run.min_chamber_oxygen_ppm?.toFixed(0) ?? "--"} ppm</strong>
            </div>
            <div>
              <span>Peak spatter rate</span>
              <strong>{selected.run.max_spatter_rate_per_s?.toFixed(1) ?? "--"} /s</strong>
            </div>
            <div>
              <span>Final quality</span>
              <strong>{selected.run.final_quality_score?.toFixed(1) ?? "--"}%</strong>
            </div>
          </div>

          <div className="history-detail-grid">
            <TelemetryChart
              title="Stored thermal response"
              subtitle={`${telemetry.length} persisted samples`}
              data={telemetry}
              unit="C"
              series={[
                {
                  key: "build_plate_temperature_c",
                  label: "Measured",
                  color: "#f3a04c",
                },
                {
                  key: "target_build_plate_temperature_c",
                  label: "Target",
                  color: "#778894",
                  dashed: true,
                },
              ]}
            />
            <AlertLog alerts={selected.alerts} />
          </div>
        </section>
      )}
    </div>
  );
}
