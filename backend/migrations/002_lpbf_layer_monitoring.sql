ALTER TABLE runs ADD COLUMN max_build_plate_temperature_c REAL;
ALTER TABLE runs ADD COLUMN min_chamber_oxygen_ppm REAL;
ALTER TABLE runs ADD COLUMN max_spatter_rate_per_s REAL;

DROP INDEX IF EXISTS idx_telemetry_run_timestamp;
ALTER TABLE telemetry_samples RENAME TO telemetry_samples_legacy;

CREATE TABLE telemetry_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    stage TEXT NOT NULL,
    elapsed_seconds REAL NOT NULL,
    build_plate_temperature_c REAL NOT NULL,
    target_build_plate_temperature_c REAL NOT NULL,
    chamber_oxygen_ppm REAL NOT NULL,
    target_chamber_oxygen_ppm REAL NOT NULL,
    recoater_vibration_mm_s REAL NOT NULL,
    recoater_position_mm REAL NOT NULL,
    target_recoater_position_mm REAL NOT NULL,
    scan_track_error_um REAL NOT NULL,
    thermal_controller_output REAL NOT NULL,
    laser_power_pct REAL NOT NULL,
    spatter_rate_per_s REAL NOT NULL,
    mean_spatter_velocity_m_s REAL NOT NULL,
    mean_spatter_diameter_um REAL NOT NULL,
    spatter_angle_deg REAL NOT NULL,
    quality_score REAL NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id)
);

DROP TABLE telemetry_samples_legacy;

CREATE INDEX idx_telemetry_run_timestamp
    ON telemetry_samples(run_id, timestamp);
