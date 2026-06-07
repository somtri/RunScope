import { AlertTriangle, CheckCircle2, ShieldAlert } from "lucide-react";
import type { Alert } from "../types";

interface AlertLogProps {
  alerts: Alert[];
}

export default function AlertLog({ alerts }: AlertLogProps) {
  return (
    <section className="panel alert-log">
      <div className="panel-heading compact">
        <div>
          <h3>Fault detection</h3>
          <span>Rule-based anomaly stream</span>
        </div>
        <span className="count-badge">{alerts.length}</span>
      </div>

      <div className="alert-list">
        {alerts.length === 0 ? (
          <div className="alert-empty">
            <CheckCircle2 size={20} />
            <div>
              <strong>No active alerts</strong>
              <span>Process signals are inside expected bounds.</span>
            </div>
          </div>
        ) : (
          alerts
            .slice()
            .reverse()
            .slice(0, 8)
            .map((alert) => (
              <article key={alert.id} className={`alert-row ${alert.severity}`}>
                {alert.severity === "critical" ? (
                  <ShieldAlert size={16} />
                ) : (
                  <AlertTriangle size={16} />
                )}
                <div className="alert-copy">
                  <div>
                    <strong>{alert.code.replaceAll("_", " ")}</strong>
                    <time>
                      {new Date(alert.timestamp).toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                        second: "2-digit",
                      })}
                    </time>
                  </div>
                  <p>{alert.message}</p>
                  <span>{alert.stage}</span>
                </div>
              </article>
            ))
        )}
      </div>
    </section>
  );
}

