pub const SCHEMA: &str = r#"
-- Source libraries (imported ZIP archives)
CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    source_filename TEXT,
    color TEXT,
    patch_count INTEGER DEFAULT 0,
    sequence_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Core patch storage
CREATE TABLE IF NOT EXISTS patches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    file_data BLOB NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    is_favorite BOOLEAN DEFAULT 0,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
);

-- Sequence storage (independent of patches)
CREATE TABLE IF NOT EXISTS sequences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    file_data BLOB NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
);

-- User-defined categories (app-only, not in Moog)
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Many-to-many: patches can have multiple categories
CREATE TABLE IF NOT EXISTS patch_categories (
    patch_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (patch_id, category_id),
    FOREIGN KEY (patch_id) REFERENCES patches(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- Many-to-many: sequences can have multiple categories
CREATE TABLE IF NOT EXISTS sequence_categories (
    sequence_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (sequence_id, category_id),
    FOREIGN KEY (sequence_id) REFERENCES sequences(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- Bank configurations (logical groupings for export)
CREATE TABLE IF NOT EXISTS banks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bank_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (bank_number)
);

-- Bank slots: which patches go in which positions
CREATE TABLE IF NOT EXISTS bank_patches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bank_id INTEGER NOT NULL,
    patch_number INTEGER NOT NULL,
    patch_id INTEGER,
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (patch_id) REFERENCES patches(id) ON DELETE SET NULL,
    UNIQUE (bank_id, patch_number)
);

-- Bank sequence slots
CREATE TABLE IF NOT EXISTS bank_sequences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bank_id INTEGER NOT NULL,
    sequence_number INTEGER NOT NULL,
    sequence_id INTEGER,
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (sequence_id) REFERENCES sequences(id) ON DELETE SET NULL,
    UNIQUE (bank_id, sequence_number)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_libraries_name ON libraries(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_patches_library ON patches(library_id);
CREATE INDEX IF NOT EXISTS idx_patches_favorite ON patches(is_favorite);
CREATE INDEX IF NOT EXISTS idx_patches_name ON patches(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_patches_hash ON patches(file_hash);
CREATE INDEX IF NOT EXISTS idx_sequences_library ON sequences(library_id);
CREATE INDEX IF NOT EXISTS idx_sequences_name ON sequences(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_sequences_hash ON sequences(file_hash);
CREATE INDEX IF NOT EXISTS idx_patch_categories_patch ON patch_categories(patch_id);
CREATE INDEX IF NOT EXISTS idx_patch_categories_category ON patch_categories(category_id);
CREATE INDEX IF NOT EXISTS idx_sequence_categories_sequence ON sequence_categories(sequence_id);
CREATE INDEX IF NOT EXISTS idx_banks_number ON banks(bank_number);
"#;
