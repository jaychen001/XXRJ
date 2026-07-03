CREATE TABLE IF NOT EXISTS pdf_catalog_items (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    chapter TEXT NOT NULL,
    catalog_page TEXT,
    matched_pages TEXT,
    excerpt TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (source_id) REFERENCES knowledge_sources(id) ON DELETE CASCADE
);
