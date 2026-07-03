CREATE TABLE IF NOT EXISTS app_meta (
    id TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS calculation_modules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL DEFAULT '0.1.0',
    fields_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS module_items (
    id TEXT PRIMARY KEY,
    module_id TEXT NOT NULL,
    name TEXT NOT NULL,
    item_type TEXT NOT NULL,
    source_chapter TEXT NOT NULL,
    source_page TEXT,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (module_id) REFERENCES calculation_modules(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS knowledge_sources (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    title TEXT NOT NULL,
    imported_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS knowledge_entries (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    page TEXT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    tags_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (source_id) REFERENCES knowledge_sources(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS pdf_coverage_items (
    id TEXT PRIMARY KEY,
    chapter TEXT NOT NULL,
    implementation_shape TEXT NOT NULL,
    status TEXT NOT NULL,
    source_page_range TEXT,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS internal_parameter_candidates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    unit TEXT,
    scenario TEXT NOT NULL DEFAULT '',
    source_page TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS internal_parameters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    value TEXT NOT NULL,
    unit TEXT,
    scenario TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS calculation_cases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    module_id TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    current_run_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (module_id) REFERENCES calculation_modules(id)
);

CREATE TABLE IF NOT EXISTS calculation_runs (
    id TEXT PRIMARY KEY,
    case_id TEXT,
    module_id TEXT NOT NULL,
    input_snapshot_json TEXT NOT NULL,
    defaults_snapshot_json TEXT NOT NULL DEFAULT '{}',
    formula_version TEXT NOT NULL,
    result_snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (case_id) REFERENCES calculation_cases(id) ON DELETE SET NULL,
    FOREIGN KEY (module_id) REFERENCES calculation_modules(id)
);

CREATE TABLE IF NOT EXISTS module_fixtures (
    id TEXT PRIMARY KEY,
    module_id TEXT NOT NULL,
    name TEXT NOT NULL,
    input_json TEXT NOT NULL,
    expected_summary_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (module_id) REFERENCES calculation_modules(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vendor_libraries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    component_type TEXT NOT NULL,
    source_file TEXT NOT NULL,
    source_format TEXT NOT NULL,
    version_name TEXT NOT NULL DEFAULT '',
    imported_at TEXT NOT NULL DEFAULT (datetime('now')),
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS vendor_import_jobs (
    id TEXT PRIMARY KEY,
    library_id TEXT,
    source_file TEXT NOT NULL,
    source_format TEXT NOT NULL,
    status TEXT NOT NULL,
    extracted_preview_json TEXT NOT NULL DEFAULT '{}',
    confidence REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (library_id) REFERENCES vendor_libraries(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vendor_models (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    brand TEXT NOT NULL DEFAULT '',
    series TEXT NOT NULL DEFAULT '',
    parameters_json TEXT NOT NULL DEFAULT '{}',
    normalized_parameters_json TEXT NOT NULL DEFAULT '{}',
    source_page TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (library_id) REFERENCES vendor_libraries(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS recommendation_results (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    matched_rules_json TEXT NOT NULL DEFAULT '[]',
    failed_rules_json TEXT NOT NULL DEFAULT '[]',
    score REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (run_id) REFERENCES calculation_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (model_id) REFERENCES vendor_models(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS report_exports (
    id TEXT PRIMARY KEY,
    case_id TEXT,
    run_id TEXT NOT NULL,
    format TEXT NOT NULL,
    path TEXT NOT NULL,
    exported_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (case_id) REFERENCES calculation_cases(id) ON DELETE SET NULL,
    FOREIGN KEY (run_id) REFERENCES calculation_runs(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO app_meta (id, value) VALUES ('schema_version', '0001_init');
