import { useState } from "react";
import {
  Activity,
  Database,
  FlaskConical,
  History,
} from "lucide-react";
import Dashboard from "./pages/Dashboard";
import RunHistory from "./pages/RunHistory";

type Page = "dashboard" | "history";

export default function App() {
  const [page, setPage] = useState<Page>("dashboard");

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <button
          className="brand"
          onClick={() => setPage("dashboard")}
          aria-label="Open dashboard"
        >
          <span className="brand-mark">
            <Activity size={19} />
          </span>
          <span>
            <strong>run_scope</strong>
            <small>LPBF Monitor</small>
          </span>
        </button>

        <nav className="primary-nav" aria-label="Primary navigation">
          <button
            className={page === "dashboard" ? "active" : ""}
            onClick={() => setPage("dashboard")}
          >
            <FlaskConical size={17} />
            Live Run
          </button>
          <button
            className={page === "history" ? "active" : ""}
            onClick={() => setPage("history")}
          >
            <History size={17} />
            Run History
          </button>
        </nav>

        <div className="sidebar-system">
          <span className="eyebrow">System</span>
          <div>
            <Database size={14} />
            SQLite persistence
          </div>
          <div>
            <Activity size={14} />
            300 ms process loop
          </div>
        </div>

        <div className="sidebar-footer">Rust / Axum / React</div>
      </aside>

      <main className="main-content">
        {page === "dashboard" ? (
          <Dashboard onOpenHistory={() => setPage("history")} />
        ) : (
          <RunHistory onBack={() => setPage("dashboard")} />
        )}
      </main>
    </div>
  );
}
