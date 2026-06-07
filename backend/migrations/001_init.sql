PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    recipe_name TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    status TEXT NOT NULL,
    duration_seconds REAL,
    max_temperature_c REAL,
    min_pressure_m_torr REAL,
    max_vibration_mm_s REAL,
    alert_count INTEGER DEFAULT 0,
    final_quality_score REAL
);

CREATE TABLE IF NOT EXISTS telemetry_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    stage TEXT NOT NULL,
    elapsed_seconds REAL NOT NULL,
    temperature_c REAL NOT NULL,
    target_temperature_c REAL NOT NULL,
    pressure_m_torr REAL NOT NULL,
    target_pressure_m_torr REAL NOT NULL,
    vibration_mm_s REAL NOT NULL,
    position_mm REAL NOT NULL,
    target_position_mm REAL NOT NULL,
    sample_alignment_error_um REAL NOT NULL,
    controller_output REAL NOT NULL,
    quality_score REAL NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id)
);

CREATE INDEX IF NOT EXISTS idx_telemetry_run_timestamp
    ON telemetry_samples(run_id, timestamp);

CREATE TABLE IF NOT EXISTS alerts (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    severity TEXT NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    stage TEXT NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id)
);

CREATE INDEX IF NOT EXISTS idx_alerts_run_timestamp
    ON alerts(run_id, timestamp);

