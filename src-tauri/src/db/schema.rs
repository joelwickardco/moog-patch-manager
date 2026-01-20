pub const SCHEMA: &str = r#"
-- Libraries: a named collection of 16 banks (maps to library/ root directory)
CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    source_filename TEXT,
    color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Banks: 16 per library (maps to library/bankXX/ directories)
CREATE TABLE IF NOT EXISTS banks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    bank_number INTEGER NOT NULL CHECK (bank_number BETWEEN 1 AND 16),
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
    UNIQUE (library_id, bank_number)
);

-- Patches: content-addressable store of unique patch data
-- NOT tied to a specific library - patches are global, reusable assets
CREATE TABLE IF NOT EXISTS patches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    file_data BLOB NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    is_favorite BOOLEAN DEFAULT 0,
    notes TEXT,
    source_library TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Sequences: content-addressable store (same pattern as patches)
CREATE TABLE IF NOT EXISTS sequences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    file_data BLOB NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    notes TEXT,
    source_library TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Tags: free-form tags for organizing patches
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Patch tags: many-to-many relationship between patches and tags
CREATE TABLE IF NOT EXISTS patch_tags (
    patch_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (patch_id, tag_id),
    FOREIGN KEY (patch_id) REFERENCES patches(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Bank patch slots: maps to library/bankXX/patchYY/ directories
-- Each bank has exactly 16 slots (1-16)
CREATE TABLE IF NOT EXISTS bank_patch_slots (
    bank_id INTEGER NOT NULL,
    slot_number INTEGER NOT NULL CHECK (slot_number BETWEEN 1 AND 16),
    patch_id INTEGER,
    PRIMARY KEY (bank_id, slot_number),
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (patch_id) REFERENCES patches(id) ON DELETE SET NULL
);

-- Bank sequence slots: maps to library/sequences/bankXX/seqYY/ directories
CREATE TABLE IF NOT EXISTS bank_sequence_slots (
    bank_id INTEGER NOT NULL,
    slot_number INTEGER NOT NULL CHECK (slot_number BETWEEN 1 AND 16),
    sequence_id INTEGER,
    PRIMARY KEY (bank_id, slot_number),
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (sequence_id) REFERENCES sequences(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_banks_library ON banks(library_id);
CREATE INDEX IF NOT EXISTS idx_patches_hash ON patches(file_hash);
CREATE INDEX IF NOT EXISTS idx_patches_name ON patches(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_patches_favorite ON patches(is_favorite);
CREATE INDEX IF NOT EXISTS idx_sequences_hash ON sequences(file_hash);
CREATE INDEX IF NOT EXISTS idx_sequences_name ON sequences(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_bank_patch_slots_patch ON bank_patch_slots(patch_id);
CREATE INDEX IF NOT EXISTS idx_bank_sequence_slots_sequence ON bank_sequence_slots(sequence_id);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_patch_tags_tag ON patch_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_patch_tags_patch ON patch_tags(patch_id);
"#;
