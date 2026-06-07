import { Crosshair, Gauge, Orbit } from "lucide-react";
import type { Telemetry } from "../types";

interface ProcessVisualizationProps {
  telemetry: Telemetry | null;
}

export default function ProcessVisualization({
  telemetry,
}: ProcessVisualizationProps) {
  const status = telemetry?.status ?? "idle";
  const position = telemetry?.recoater_position_mm ?? 0;
  const recoaterX = 58 + position * 2.24;
  const laserActive = telemetry?.stage === "LaserScan";
  const scanDistance = ((telemetry?.elapsed_seconds ?? 0) * 19) % 204;
  const laserX = 68 + scanDistance;
  const laserY =
    70 + (Math.floor(((telemetry?.elapsed_seconds ?? 0) * 19) / 204) % 4) * 27;
  const spatterIntensity = Math.min(
    1,
    (telemetry?.spatter_rate_per_s ?? 0) / 55,
  );

  return (
    <section className={`panel process-visual status-${status}`}>
      <div className="panel-heading compact">
        <div>
          <h3>LPBF layer visualization</h3>
          <span>Build plate / recoater / laser scan</span>
        </div>
        <span className="live-tag">
          <i />
          live model
        </span>
      </div>

      <div className="lpbf-wrap">
        <svg viewBox="0 0 340 230" role="img" aria-label="Simulated LPBF build plate">
          <defs>
            <filter id="softGlow">
              <feGaussianBlur stdDeviation="4" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          <rect x="48" y="37" width="244" height="158" rx="7" className="build-envelope" />
          <rect x="61" y="50" width="218" height="132" rx="3" className="build-plate" />
          {[0, 1, 2, 3, 4].map((row) => (
            <path
              key={row}
              d={`M69 ${66 + row * 25} H271`}
              className="hatch-line"
            />
          ))}
          <path d="M69 57v116M271 57v116" className="plate-boundary" />
          <path
            d={`M${recoaterX} 43 V189`}
            className="recoater-bar"
          />
          <path d="M69 66 H271 M271 91 H69 M69 116 H271 M271 141 H69" className="scan-path" />
          {laserActive && (
            <>
              <circle cx={laserX} cy={laserY} r="8" className="laser-halo" />
              <circle
                cx={laserX}
                cy={laserY}
                r="3.5"
                className="laser-spot"
                filter="url(#softGlow)"
              />
              {[
                [-18, -18],
                [-8, -28],
                [11, -22],
                [19, -12],
                [4, -34],
              ].map(([dx, dy], index) => (
                <g
                  key={index}
                  className="spatter-vector"
                  style={{ opacity: spatterIntensity }}
                >
                  <path d={`M${laserX} ${laserY} l${dx} ${dy}`} />
                  <circle cx={laserX + dx} cy={laserY + dy} r={1.5 + (index % 2)} />
                </g>
              ))}
            </>
          )}
          <text x="24" y="24" className="svg-label">
            BUILD PLATE 01
          </text>
          <text x="256" y="216" className="svg-label">
            {telemetry?.stage?.toUpperCase() ?? "IDLE"}
          </text>
        </svg>
      </div>

      <div className="process-readouts">
        <div>
          <Crosshair size={15} />
          <span>Laser power</span>
          <strong>{(telemetry?.laser_power_pct ?? 0).toFixed(1)}%</strong>
        </div>
        <div>
          <Orbit size={15} />
          <span>Spatter diameter</span>
          <strong>{(telemetry?.mean_spatter_diameter_um ?? 0).toFixed(1)} um</strong>
        </div>
        <div>
          <Gauge size={15} />
          <span>Ejection angle</span>
          <strong>{(telemetry?.spatter_angle_deg ?? 0).toFixed(1)} deg</strong>
        </div>
      </div>
    </section>
  );
}
