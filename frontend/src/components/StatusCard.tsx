import type { LucideIcon } from "lucide-react";

interface StatusCardProps {
  label: string;
  value: string;
  unit?: string;
  target?: string;
  icon: LucideIcon;
  tone?: "normal" | "good" | "warning" | "critical";
  spark?: number;
}

export default function StatusCard({
  label,
  value,
  unit,
  target,
  icon: Icon,
  tone = "normal",
  spark = 50,
}: StatusCardProps) {
  return (
    <article className={`status-card tone-${tone}`}>
      <div className="status-card-top">
        <span>{label}</span>
        <Icon size={16} />
      </div>
      <div className="status-value">
        <strong>{value}</strong>
        {unit && <span>{unit}</span>}
      </div>
      <div className="status-card-bottom">
        <span>{target ?? "Live sensor"}</span>
        <span className="micro-bar" aria-hidden="true">
          <i style={{ width: `${Math.min(100, Math.max(4, spark))}%` }} />
        </span>
      </div>
    </article>
  );
}

