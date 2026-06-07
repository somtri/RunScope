import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Activity,
  AlignCenter,
  Gauge,
  Radio,
  Sparkles,
  Thermometer,
  Waves,
} from "lucide-react";
import { api } from "../api/client";
import AlertLog from "../components/AlertLog";
import ProcessVisualization from "../components/ProcessVisualization";
import RunControls from "../components/RunControls";
import StatusCard from "../components/StatusCard";
import TelemetryChart from "../components/TelemetryChart";
import { useTelemetrySocket } from "../hooks/useTelemetrySocket";
import type {
  Alert,
  Metrics,
  Recipe,
  RunState,
  ServerMessage,
  Telemetry,
} from "../types";

interface DashboardProps {
  onOpenHistory: () => void;
}

export default function Dashboard({ onOpenHistory }: DashboardProps) {
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [selectedRecipe, setSelectedRecipe] = useState("");
  const [telemetry, setTelemetry] = useState<Telemetry[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [runState, setRunState] = useState<RunState | null>(null);
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const handleSocketMessage = useCallback((message: ServerMessage) => {
    if (message.type === "telemetry") {
      setTelemetry((current) => [...current, message.data].slice(-180));
    } else if (message.type === "alert") {
      setAlerts((current) =>
        current.some((alert) => alert.id === message.data.id)
          ? current
          : [...current, message.data],
      );
    } else {
      setRunState(message.data);
      if (message.data.status !== "running") {
        void api.getMetrics().then(setMetrics).catch(() => undefined);
      }
    }
  }, []);

  const connection = useTelemetrySocket(handleSocketMessage);
  const latest = telemetry.at(-1) ?? null;
  const visualTelemetry =
    latest && runState && runState.status !== "running"
      ? { ...latest, status: runState.status, stage: runState.stage }
      : latest;

  useEffect(() => {
    Promise.all([api.getRecipes(), api.getMetrics()])
      .then(([availableRecipes, currentMetrics]) => {
        setRecipes(availableRecipes);
        setSelectedRecipe(availableRecipes[0]?.id ?? "");
        setMetrics(currentMetrics);
      })
      .catch((reason: Error) => setError(reason.message));
  }, []);

  const activeRecipe = useMemo(
    () => recipes.find((recipe) => recipe.id === selectedRecipe),
    [recipes, selectedRecipe],
  );

  const handleStart = async () => {
    setBusy(true);
    setError("");
    setTelemetry([]);
    setAlerts([]);
    try {
      const response = await api.startRun(selectedRecipe);
      setRunState({
        run_id: response.run_id,
        status: "running",
        stage: activeRecipe?.stages[0]?.name ?? "Starting",
        stage_progress: 0,
        overall_progress: 0,
      });
      setMetrics((current) =>
        current ? { ...current, active_run: true } : current,
      );
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Unable to start run");
    } finally {
      setBusy(false);
    }
  };

  const handleAbort = async () => {
    if (!runState) return;
    setBusy(true);
    setError("");
    try {
      await api.abortRun(runState.run_id);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Unable to abort run");
    } finally {
      setBusy(false);
    }
  };

  const qualityTone =
    (latest?.quality_score ?? 100) < 82
      ? "critical"
      : (latest?.quality_score ?? 100) < 92
        ? "warning"
        : "good";

  return (
    <div className="page dashboard-page">
      <header className="page-header">
        <div>
          <span className="eyebrow">Control room / LPBF layer monitoring</span>
          <h1>Live layer monitor</h1>
          <p>
            Recoater, atmosphere, scan, and spatter signals in one view.
          </p>
        </div>
        <div className="header-status">
          <span className={`connection connection-${connection}`}>
            <i />
            Backend {connection}
          </span>
          <button className="text-button" onClick={onOpenHistory}>
            {metrics?.total_runs ?? 0} saved runs
          </button>
        </div>
      </header>

      {error && (
        <div className="error-banner" role="alert">
          <Radio size={16} />
          {error}
          <button onClick={() => setError("")}>Dismiss</button>
        </div>
      )}

      <div className="dashboard-lead">
        <RunControls
          recipes={recipes}
          selectedRecipe={selectedRecipe}
          onRecipeChange={setSelectedRecipe}
          runState={runState}
          busy={busy}
          onStart={handleStart}
          onAbort={handleAbort}
        />
        <ProcessVisualization telemetry={visualTelemetry} />
      </div>

      <section className="status-grid" aria-label="Live process readings">
        <StatusCard
          label="Build plate"
          value={(latest?.build_plate_temperature_c ?? 25).toFixed(1)}
          unit="C"
          target={`Target ${(latest?.target_build_plate_temperature_c ?? 25).toFixed(1)} C`}
          icon={Thermometer}
          spark={((latest?.build_plate_temperature_c ?? 25) / 215) * 100}
        />
        <StatusCard
          label="Chamber oxygen"
          value={(latest?.chamber_oxygen_ppm ?? 1200).toFixed(0)}
          unit="ppm"
          target={`Target ${(latest?.target_chamber_oxygen_ppm ?? 80).toFixed(0)} ppm`}
          icon={Gauge}
          spark={100 - ((latest?.chamber_oxygen_ppm ?? 1200) / 1500) * 100}
        />
        <StatusCard
          label="Spatter rate"
          value={(latest?.spatter_rate_per_s ?? 0).toFixed(1)}
          unit="/s"
          target="Burst threshold 45"
          icon={Waves}
          tone={(latest?.spatter_rate_per_s ?? 0) > 45 ? "warning" : "normal"}
          spark={((latest?.spatter_rate_per_s ?? 0) / 90) * 100}
        />
        <StatusCard
          label="Mean velocity"
          value={(latest?.mean_spatter_velocity_m_s ?? 0).toFixed(2)}
          unit="m/s"
          target={`Mean diameter ${(latest?.mean_spatter_diameter_um ?? 0).toFixed(0)} um`}
          icon={Activity}
          spark={((latest?.mean_spatter_velocity_m_s ?? 0) / 7.5) * 100}
        />
        <StatusCard
          label="Scan-track error"
          value={(latest?.scan_track_error_um ?? 1.0).toFixed(2)}
          unit="um"
          target="Tolerance 4.50"
          icon={AlignCenter}
          spark={((latest?.scan_track_error_um ?? 1.0) / 8) * 100}
        />
        <StatusCard
          label="Layer quality"
          value={(latest?.quality_score ?? 99.8).toFixed(1)}
          unit="%"
          target={`${alerts.length} detected anomalies`}
          icon={Sparkles}
          tone={qualityTone}
          spark={latest?.quality_score ?? 99.8}
        />
      </section>

      <section className="chart-grid">
        <TelemetryChart
          title="Build-plate temperature"
          subtitle="Measured thermal response against recipe target"
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
        <TelemetryChart
          title="Chamber atmosphere"
          subtitle="Oxygen concentration during inerting and scanning"
          data={telemetry}
          unit="ppm"
          series={[
            {
              key: "chamber_oxygen_ppm",
              label: "Measured",
              color: "#2acbb0",
            },
            {
              key: "target_chamber_oxygen_ppm",
              label: "Target",
              color: "#778894",
              dashed: true,
            },
          ]}
          domain={[0, "auto"]}
        />
        <TelemetryChart
          title="Spatter activity"
          subtitle="Detected ejection events during the laser scan"
          data={telemetry}
          unit="/s"
          series={[
            {
              key: "spatter_rate_per_s",
              label: "Ejection rate",
              color: "#7ba7ff",
            },
          ]}
          domain={[0, 70]}
        />
        <TelemetryChart
          title="Layer quality"
          subtitle="Composite atmosphere, motion, scan, and spatter score"
          data={telemetry}
          unit="%"
          series={[
            { key: "quality_score", label: "Quality", color: "#b6db72" },
          ]}
          domain={[70, 100]}
        />
      </section>

      <div className="dashboard-footer-grid">
        <AlertLog alerts={alerts} />
        <section className="panel stage-sequence">
          <div className="panel-heading compact">
            <div>
              <h3>Layer stages</h3>
              <span>{activeRecipe?.name ?? "Loading recipe"}</span>
            </div>
            <span className="sequence-total">
              {activeRecipe
                ? `${Math.round(
                    activeRecipe.stages.reduce(
                      (total, stage) => total + stage.duration_seconds,
                      0,
                    ),
                  )} sec`
                : "--"}
            </span>
          </div>
          <ol>
            {activeRecipe?.stages.map((stage) => {
              const currentIndex = activeRecipe.stages.findIndex(
                (item) => item.name === runState?.stage,
              );
              const index = activeRecipe.stages.indexOf(stage);
              const terminal = ["complete", "faulted", "aborted"].includes(
                runState?.status ?? "",
              );
              const state =
                runState?.status === "complete" || (terminal && index < currentIndex)
                  ? "complete"
                  : index === currentIndex
                    ? "active"
                    : "pending";
              return (
                <li key={stage.name} className={state}>
                  <span>{String(index + 1).padStart(2, "0")}</span>
                  <div>
                    <strong>{stage.name}</strong>
                    <small>{stage.description}</small>
                  </div>
                  <time>{stage.duration_seconds}s</time>
                </li>
              );
            })}
          </ol>
        </section>
      </div>
    </div>
  );
}
