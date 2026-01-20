# Moog Muse Patch Manager - Application Specification

**Version:** 1.2
**Last Updated:** January 2026
**Target Platforms:** macOS (primary), Linux (secondary)

---

## 1. Executive Summary

The Moog Muse Patch Manager is a desktop application for organizing and managing sound patches and sequences for the Moog Muse synthesizer. The application provides a local database for patch management with features including favorites and custom bank organization - capabilities not available in the synthesizer's native filesystem-based organization.

### Key Features
- Import patches/sequences from Moog library archives (.zip) or bank directories
- Organize patches with favorites
- Search and filter patch library
- Build custom bank configurations (16 banks × 16 patches)
- Export complete library structure for transfer to synthesizer
- Duplicate detection via file hashing
- Metadata management (notes, tags)

### Design Goals
- Minimal dependencies and small binary size
- Native macOS feel with easy Linux portability
- Fast, local-first operation (no cloud dependencies)
- Simple CRUD interface with path to advanced drag-and-drop UI
- Easy distribution via GitHub releases

---

## 2. System Architecture

### 2.1 Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                  Desktop Application                 │
├─────────────────────────────────────────────────────┤
│  Presentation Layer (Svelte + TailwindCSS)          │
│  ├── Components (PatchList, BankBuilder, etc.)      │
│  ├── Stores (patches, sequences, banks)             │
│  └── Routes (main view, settings)                   │
├─────────────────────────────────────────────────────┤
│  API Layer (Tauri Commands)                         │
│  ├── Patch Management                               │
│  ├── Bank Management                                │
│  ├── Import/Export Operations                       │
│  └── Search/Filter                                  │
├─────────────────────────────────────────────────────┤
│  Business Logic Layer (Rust)                        │
│  ├── Database Operations (rusqlite)                 │
│  ├── File System Operations                         │
│  ├── Hash Calculation (SHA-256)                     │
│  ├── ZIP Operations (zip crate)                     │
│  └── Moog Library Parser                            │
├─────────────────────────────────────────────────────┤
│  Data Layer (SQLite)                                │
│  └── Single-file database with ACID guarantees      │
└─────────────────────────────────────────────────────┘
```

### 2.2 Technology Rationale

| Technology | Justification |
|------------|---------------|
| **Tauri** | Small binaries (3-5MB), native performance, cross-platform, secure |
| **Svelte** | Minimal boilerplate, fast compilation, easy to learn, great for CRUD |
| **Rust** | Safe file operations, excellent SQLite support, zero-cost abstractions |
| **SQLite** | Zero-config, ACID compliance, perfect for desktop apps, single file |
| **TailwindCSS** | Rapid UI development, consistent design system |

---

## 3. Technology Stack

### 3.1 Core Technologies

```toml
# Backend (Rust)
tauri = "2.x"
rusqlite = "0.31"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"  # For file hashing
zip = "0.6"
walkdir = "2.4"
```

```json
// Frontend (JavaScript/Svelte)
{
  "svelte": "^4.0.0",
  "vite": "^5.0.0",
  "@tauri-apps/api": "^2.0.0",
  "tailwindcss": "^3.4.0"
}
```

### 3.2 Development Tools

- **Package Manager:** npm/pnpm
- **Build Tool:** Vite (bundler), Cargo (Rust compiler)
- **Version Control:** Git
- **Deployment:** GitHub Releases with binary artifacts

### 3.3 Target Runtimes

- **macOS:** 12.0+ (Monterey and later)
- **Linux:** Ubuntu 20.04+, Fedora 36+, or equivalent

---

## 4. Data Model

### 4.1 Database Schema

The data model reflects the actual Moog Muse filesystem structure where:
- A **library** is a collection of 16 banks
- Each **bank** contains 16 patch slots and 16 sequence slots
- **Patches and sequences** are global content stores (deduplicated by hash)
- **Bank slots** reference patches/sequences, defining where content appears in a library

```sql
-- Libraries: a named collection of 16 banks (maps to library/ root directory)
CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    source_filename TEXT,              -- Original import filename (informational)
    color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Banks: 16 per library (maps to library/bankXX/ directories)
CREATE TABLE IF NOT EXISTS banks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    bank_number INTEGER NOT NULL CHECK (bank_number BETWEEN 1 AND 16),
    name TEXT NOT NULL,                -- Becomes <name>.bank filename on export
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
    name TEXT NOT NULL,                -- Display name (from original filename)
    file_data BLOB NOT NULL,           -- .mmp file contents
    file_hash TEXT NOT NULL UNIQUE,    -- SHA-256 for deduplication
    file_size INTEGER NOT NULL,
    is_favorite BOOLEAN DEFAULT 0,
    notes TEXT,
    source_library TEXT,               -- Informational: which library it came from
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Sequences: content-addressable store (same pattern as patches)
CREATE TABLE IF NOT EXISTS sequences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    file_data BLOB NOT NULL,           -- .mmseq file contents
    file_hash TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    notes TEXT,
    source_library TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Bank patch slots: maps to library/bankXX/patchYY/ directories
-- Each bank has exactly 16 slots (1-16)
CREATE TABLE IF NOT EXISTS bank_patch_slots (
    bank_id INTEGER NOT NULL,
    slot_number INTEGER NOT NULL CHECK (slot_number BETWEEN 1 AND 16),
    patch_id INTEGER,                  -- NULL = empty/default patch (empty directory)
    PRIMARY KEY (bank_id, slot_number),
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (patch_id) REFERENCES patches(id) ON DELETE SET NULL
);

-- Bank sequence slots: maps to library/sequences/bankXX/seqYY/ directories
CREATE TABLE IF NOT EXISTS bank_sequence_slots (
    bank_id INTEGER NOT NULL,
    slot_number INTEGER NOT NULL CHECK (slot_number BETWEEN 1 AND 16),
    sequence_id INTEGER,               -- NULL = empty slot
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
```

### 4.2 Data Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                     LIBRARY STRUCTURE                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  libraries (1) ──────────────── (16) banks                      │
│                                       │                         │
│                          ┌────────────┴────────────┐            │
│                          │                         │            │
│                   (16) bank_patch_slots    (16) bank_sequence_slots
│                          │                         │            │
│                          ▼                         ▼            │
│                     (0..1) patches           (0..1) sequences   │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                     CONTENT STORE (Global)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  patches                                                        │
│                                                                 │
│  sequences                                                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Filesystem Mapping:**
```
library/bank03/pads.bank           → banks(library_id=X, bank_number=3, name="pads")
library/bank03/patch05/bass.mmp    → 1. patches(name="bass", file_data=..., file_hash=...)
                                     2. bank_patch_slots(bank_id=Y, slot_number=5, patch_id=Z)
library/sequences/bank03/seq01/... → similar pattern for sequences
```

### 4.3 Key Constraints

- Library names must be unique
- Patches identified by SHA-256 hash (no duplicates by content globally)
- Sequences identified by SHA-256 hash (no duplicates by content globally)
- Each library has exactly 16 banks (bank_number 1-16 per library)
- Each bank has exactly 16 patch slots and 16 sequence slots
- Slot numbers are 1-16 per bank (enforced by CHECK constraint)
- Bank names are user-defined, exported as `<name>.bank` files
- **Patches/sequences are global**: not owned by libraries, can be referenced by multiple bank slots
- **Deleting a library**: cascades to banks and slot assignments; patches/sequences remain in global store
- **Deleting a patch**: sets referencing slot's patch_id to NULL (empty slot)

---

## 5. API Specification (Tauri Commands)

All commands are async and return `Result<T, String>` where `String` is the error message.

### 5.1 Patch Management

```rust
#[tauri::command]
async fn get_all_patches(
    filter: Option<PatchFilter>
) -> Result<Vec<PatchDto>, String>

#[tauri::command]
async fn get_patch_by_id(id: i64) -> Result<PatchDto, String>

#[tauri::command]
async fn toggle_favorite(patch_id: i64) -> Result<(), String>

#[tauri::command]
async fn update_patch_notes(
    patch_id: i64, 
    notes: String
) -> Result<(), String>

#[tauri::command]
async fn delete_patch(patch_id: i64) -> Result<(), String>

#[tauri::command]
async fn search_patches(query: String) -> Result<Vec<PatchDto>, String>
```

**PatchFilter Structure:**
```rust
struct PatchFilter {
    is_favorite: Option<bool>,
    name_contains: Option<String>,
    source_library: Option<String>,    // Filter by original import source
}
```

**PatchDto Structure:**
```rust
struct PatchDto {
    id: i64,
    name: String,
    file_hash: String,
    file_size: i64,
    is_favorite: bool,
    notes: Option<String>,
    source_library: Option<String>,    // Informational: where it was imported from
    usage_count: i64,                  // Number of bank slots referencing this patch
    created_at: String,
    updated_at: String,
}
```

### 5.2 Sequence Management

```rust
#[tauri::command]
async fn get_all_sequences(
    filter: Option<SequenceFilter>
) -> Result<Vec<SequenceDto>, String>

#[tauri::command]
async fn get_sequence_by_id(id: i64) -> Result<SequenceDto, String>

#[tauri::command]
async fn update_sequence_notes(
    sequence_id: i64,
    notes: String
) -> Result<(), String>

#[tauri::command]
async fn delete_sequence(sequence_id: i64) -> Result<(), String>

#[tauri::command]
async fn search_sequences(query: String) -> Result<Vec<SequenceDto>, String>
```

**SequenceFilter Structure:**
```rust
struct SequenceFilter {
    name_contains: Option<String>,
    source_library: Option<String>,    // Filter by original import source
}
```

**SequenceDto Structure:**
```rust
struct SequenceDto {
    id: i64,
    name: String,
    file_hash: String,
    file_size: i64,
    notes: Option<String>,
    source_library: Option<String>,    // Informational: where it was imported from
    usage_count: i64,                  // Number of bank slots referencing this sequence
    created_at: String,
    updated_at: String,
}
```

### 5.3 Library Management

```rust
#[tauri::command]
async fn get_all_libraries() -> Result<Vec<LibraryDto>, String>

#[tauri::command]
async fn get_library_by_id(id: i64) -> Result<LibraryDto, String>

#[tauri::command]
async fn create_library(name: String) -> Result<LibraryDto, String>

#[tauri::command]
async fn update_library(
    id: i64,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>
) -> Result<LibraryDto, String>

#[tauri::command]
async fn delete_library(id: i64) -> Result<(), String>
```

**LibraryDto Structure:**
```rust
struct LibraryDto {
    id: i64,
    name: String,
    description: Option<String>,
    source_filename: Option<String>,
    color: Option<String>,
    patch_count: i64,
    sequence_count: i64,
    created_at: String,
    updated_at: String,
}
```

### 5.5 Bank Management

```rust
#[tauri::command]
async fn get_banks_for_library(library_id: i64) -> Result<Vec<BankDto>, String>

#[tauri::command]
async fn get_bank(
    library_id: i64,
    bank_number: i32
) -> Result<BankDto, String>

#[tauri::command]
async fn update_bank_name(
    library_id: i64,
    bank_number: i32,
    name: String
) -> Result<(), String>

#[tauri::command]
async fn assign_patch_to_slot(
    library_id: i64,
    bank_number: i32,
    slot_number: i32,          // 1-16
    patch_id: Option<i64>      // None = clear slot (empty/default patch)
) -> Result<(), String>

#[tauri::command]
async fn assign_sequence_to_slot(
    library_id: i64,
    bank_number: i32,
    slot_number: i32,          // 1-16
    sequence_id: Option<i64>   // None = clear slot
) -> Result<(), String>

#[tauri::command]
async fn clear_patch_slot(
    library_id: i64,
    bank_number: i32,
    slot_number: i32
) -> Result<(), String>

#[tauri::command]
async fn clear_sequence_slot(
    library_id: i64,
    bank_number: i32,
    slot_number: i32
) -> Result<(), String>
```

**BankDto Structure:**
```rust
struct BankDto {
    id: i64,
    library_id: i64,
    bank_number: i32,
    name: String,
    description: Option<String>,
    patch_slots: Vec<BankSlotDto<PatchDto>>,     // 16 slots
    sequence_slots: Vec<BankSlotDto<SequenceDto>>, // 16 slots
    created_at: String,
    updated_at: String,
}

struct BankSlotDto<T> {
    slot_number: i32,          // 1-16
    content: Option<T>,        // Some(patch/sequence) or None for empty slot
}
```

### 5.6 Import Operations

```rust
#[tauri::command]
async fn import_library_zip(
    file_path: String
) -> Result<ImportResult, String>

#[tauri::command]
async fn import_bank_directory(
    library_id: i64,
    directory_path: String
) -> Result<ImportResult, String>

#[tauri::command]
async fn validate_library_structure(
    path: String
) -> Result<ValidationResult, String>
```

**ImportResult Structure:**
```rust
struct ImportResult {
    library_id: i64,
    library_name: String,
    patches_imported: i32,         // New patches added to global store
    patches_reused: i32,           // Existing patches (by hash) linked to slots
    sequences_imported: i32,       // New sequences added to global store
    sequences_reused: i32,         // Existing sequences linked to slots
    banks_created: i32,            // Always 16 for a full library import
    slots_populated: i32,          // Non-empty patch + sequence slots
    errors: Vec<String>,
    warnings: Vec<String>,
}
```

**Import Behavior:**
- Patches/sequences are added to the global content store (deduplicated by hash)
- If a patch already exists (same hash), the existing patch is linked to the slot
- `source_library` field on patches/sequences records where content was first imported from
- Bank slots are created/updated to reference the appropriate patches/sequences

**ValidationResult Structure:**
```rust
struct ValidationResult {
    is_valid: bool,
    structure_errors: Vec<String>,
    missing_files: Vec<String>,
    invalid_files: Vec<String>,
}
```

### 5.7 Export Operations

```rust
#[tauri::command]
async fn export_library(
    library_id: i64,
    output_path: String
) -> Result<ExportResult, String>

#[tauri::command]
async fn preview_export() -> Result<ExportPreview, String>
```

**ExportResult Structure:**
```rust
struct ExportResult {
    output_path: String,
    file_size: i64,
    banks_exported: i32,
    patches_exported: i32,
    sequences_exported: i32,
    empty_slots: i32,
}
```

**ExportPreview Structure:**
```rust
struct ExportPreview {
    total_banks: i32,
    total_patches: i32,
    total_sequences: i32,
    empty_patch_slots: i32,
    empty_sequence_slots: i32,
    estimated_size: i64,
}
```

---

## 6. UI/UX Specification

### 6.1 Application Layout

```
┌─────────────────────────────────────────────────────┐
│  Moog Muse Patch Manager            [? Help] [⚙️]   │
├─────────────────────────────────────────────────────┤
│  📁 Library  |  🏦 Banks                            │
├──────────┬──────────────────────────────────────────┤
│          │  🔍 Search: [__________]  ⭐ ❤️ 🎨       │
│ Sidebar  │                                          │
│          │  ┌──────────────────────────────────┐   │
│ ☆ Fav    │  │  Patch Card                      │   │
│ 📂 All   │  │  Name: Deep Bass                 │   │
│          │  │  [⭐] [❤️] [✏️] [🗑️]             │   │
│          │  └──────────────────────────────────┘   │
│          │                                          │
│          │  [Similar cards in grid layout...]      │
│          │                                          │
│          │                                          │
│ [Import] │                                          │
│ [Export] │                                          │
└──────────┴──────────────────────────────────────────┘
```

### 6.2 Core Components

#### 6.2.1 PatchList Component
**Purpose:** Display all patches with filtering and search

**Props:**
- `filter: PatchFilter`
- `searchQuery: string`

**Features:**
- Grid or list view toggle
- Sort by: name, date, favorites
- Quick actions: favorite, delete, edit

**State:**
```javascript
let patches = [];
let selectedPatches = [];
let viewMode = 'grid'; // or 'list'
```

#### 6.2.2 PatchCard Component
**Purpose:** Display individual patch with metadata

**Props:**
- `patch: PatchDto`

**Features:**
- Patch name (editable on click)
- Favorite star (toggle)
- Quick actions menu
- Notes display (expandable)

#### 6.2.3 Banks UI Components
**Purpose:** Visual interface for managing library banks and slots

**Component Hierarchy:**
```
BanksView (Container)
├── BankList (Left panel - bank selection)
└── BankDetail (Right panel - 16 patch + 16 sequence slots)
```

**BanksView Component**
- Props: `selectedLibraryId`, `libraries`
- Manages: bank data loading, slot assignment handlers
- Layout: Two-column layout with status messages

**BankList Component**
- Props: `banks`, `selectedBankNumber` (bindable), `loading`
- Features: List of 16 banks per library with slot counts
- Visual: Scrollable list with active bank highlighting

**BankDetail Component**
- Props: `bank`, `libraryId`, event handlers for slot clicks/drops, `onBankNameUpdate`
- Features:
  - **Inline bank name editing**: Click bank name to edit, save with Enter/blur, cancel with Escape
  - 4x4 grid of 16 patch slots
  - 4x4 grid of 16 sequence slots
  - Drag-and-drop support for patches and sequences
  - Visual feedback for drag-over state
  - Empty slot indicators
  - Hover hint showing "(click to edit)" on bank name
- Layout:
  ```
  Bank 03: "Bass Patches" ← (click to edit)

  Patches (8/16)
  ┌─────┬─────┬─────┬─────┐
  │ #01 │ #02 │ #03 │ #04 │
  │ Pch │ Pch │ --- │ Pch │
  ├─────┼─────┼─────┼─────┤
  │ #05 │ #06 │ #07 │ #08 │
  │ --- │ Pch │ Pch │ Pch │
  └─────┴─────┴─────┴─────┘
  ... (continues to 16)

  Sequences (2/16)
  [Similar 4x4 grid for sequences]
  ```

#### 6.2.4 ImportDialog Component
**Purpose:** Guide user through import process

**Steps:**
1. Select file (.zip) or directory
2. Validate structure
3. Preview import (show duplicates)
4. Confirm and execute
5. Show results

#### 6.2.5 ExportDialog Component
**Purpose:** Configure and execute export

**Steps:**
1. Preview export (show bank structure)
2. Select output location
3. Confirm
4. Show progress
5. Show results with file location

### 6.3 User Workflows

#### Workflow 1: Import Library from ZIP
```
User Action → System Response
──────────────────────────────────────────────
1. Click "Import" button
   → Open file/folder picker dialog

2. Select library.zip file
   → Extract library name from ZIP filename
   → Validate structure in background
   → Show validation results

3. Review preview (duplicates highlighted)
   → Display ImportPreview component
   → Check for name conflicts

4. Confirm import
   → Show progress indicator
   → Create new library entry
   → Import files, calculate hashes
   → Store patches/sequences in database
   → Create 16 banks for the library

5. View results
   → Show ImportResult summary with library name
   → Auto-select newly imported library
   → Navigate to imported library's patches
```

#### Workflow 2: Copy Patches to Build Custom Library
```
User Action → System Response
──────────────────────────────────────────────
1. Create new empty library
   → Click "+" button in sidebar
   → Modal: "Create New Library"
   → Enter name (e.g., "My Live Set")
   → Library created with 16 empty banks

2. Browse existing patches
   → Navigate to Libraries tab
   → Search/filter patches from any library
   → "All Libraries" or specific library filter

3. Copy patch to new library
   → Click copy icon (📋) on patch card (left of favorite star)
   → CopyPatchModal opens:
     - Library selector: "My Live Set" (dropdown)
     - Bank selector: "Bank 03: Bass" (dropdown)
     - Slot selector: "Next available" (default) or specific slot
   → Click "Copy Patch"
   → Patch data duplicated to destination bank slot
   → Success notification shown

4. Continue copying patches
   → Modal appears for each copy operation
   → User selects destination library/bank/slot each time
   → Builds custom library incrementally

5. Repeat for all desired patches
   → Build complete custom library
   → Banks filled with curated patches

6. Export library
   → Navigate to Banks tab
   → Select "My Live Set" library
   → Click "Export Library"
   → Save .zip file
   → Transfer to synthesizer
```

**Key Benefits of This Workflow:**
- ✅ Never leave Libraries tab (stay in browsing mode)
- ✅ Explicit destination selection (no guessing)
- ✅ Can copy from any library to any other library
- ✅ Non-destructive (original patches unchanged)
- ✅ Works well with search/filter to find patches
- ✅ Copy creates new patch entry (duplicate data, not reference)

#### Workflow 3: Organize Patches
```
User Action → System Response
──────────────────────────────────────────────
1. Browse patch list
   → Display all patches with search/filter
   
2. Star favorite patches
   → Toggle favorite flag in DB
   → Update UI immediately

3. Add notes to patch
   → Click edit icon
   → Open notes editor
   → Save to DB
```

#### Workflow 4: Organize Banks and Export Library
```
User Action → System Response
──────────────────────────────────────────────
1. Navigate to "Banks" tab
   → Select a library from sidebar
   → BanksView shows 16 banks for selected library

2. Select bank to edit (e.g., Bank 03)
   → BankList highlights selected bank
   → BankDetail shows 16 patch slots + 16 sequence slots

3. Edit bank name (optional)
   → Click on bank name in BankDetail header
   → Name becomes editable text input
   → Type new name (e.g., "Live Set Bass")
   → Press Enter or click away to save
   → Press Escape to cancel
   → Backend updates banks table via update_bank_name
   → Success message shown: "Bank name updated"
   → Both BankList and BankDetail reflect new name
   → New name will be used for .bank filename on export

4. Drag patches into slots
   → Drag from library or from other bank slots
   → Drop into target slot
   → Update bank_patch_slots table
   → Visual feedback during drag/drop

5. Drag sequences into sequence slots
   → Same drag-and-drop workflow for sequences
   → Update bank_sequence_slots table

6. Repeat for other banks
   → Select different bank from BankList
   → Continue organizing patches/sequences

7. Click "Export Library"
   → Show ExportPreview
   → Select output location
   → Generate ZIP with library structure (16 banks)
   → Each bank exports with <bankname>.bank file
   → Show success with file path
```

### 6.4 Design System

#### Colors (TailwindCSS)
```javascript
// Primary
primary: '#FF6B35',      // Moog orange
secondary: '#004E89',    // Dark blue
accent: '#F7B801',       // Yellow

// Semantic
favorite: '#FFD700',     // Gold star

// UI
background: '#1A1A1A',   // Dark mode primary
surface: '#2D2D2D',      // Cards/panels
text: '#FFFFFF',
textSecondary: '#B0B0B0',
border: '#404040',
```

#### Typography
```css
/* Headings */
h1: text-3xl font-bold
h2: text-2xl font-semibold
h3: text-xl font-medium

/* Body */
body: text-base
small: text-sm
```

#### Spacing
- Use Tailwind's default spacing scale (4px increments)
- Component padding: p-4 (16px)
- Card gaps: gap-4
- Section margins: mb-6

---

## 7. Project Structure

```
moog-muse-manager/
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── main.rs                 # App entry point
│   │   ├── commands/               # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── libraries.rs
│   │   │   ├── patches.rs
│   │   │   ├── sequences.rs
│   │   │   ├── banks.rs
│   │   │   ├── import.rs
│   │   │   └── export.rs
│   │   ├── db/                     # Database operations
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs           # SQL schema
│   │   │   ├── connection.rs       # Connection pool
│   │   │   └── migrations.rs
│   │   ├── models/                 # Data structures
│   │   │   ├── mod.rs
│   │   │   ├── library.rs
│   │   │   ├── patch.rs
│   │   │   ├── sequence.rs
│   │   │   └── bank.rs
│   │   ├── moog/                   # Moog-specific logic
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs           # Parse directory structure
│   │   │   ├── exporter.rs         # Generate library structure
│   │   │   └── validator.rs        # Validate library structure
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── hash.rs             # SHA-256 hashing
│   │       └── zip.rs              # ZIP operations
│   ├── Cargo.toml
│   ├── tauri.conf.json             # Tauri configuration
│   └── build.rs
│
├── src/                            # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── patches/
│   │   │   │   ├── PatchList.svelte
│   │   │   │   ├── PatchCard.svelte
│   │   │   │   ├── PatchDetail.svelte
│   │   │   │   └── PatchSearch.svelte
│   │   │   ├── sequences/
│   │   │   │   ├── SequenceList.svelte
│   │   │   │   └── SequenceCard.svelte
│   │   │   ├── banks/
│   │   │   │   ├── BanksView.svelte    # Container component
│   │   │   │   ├── BankList.svelte     # Left panel - bank selection
│   │   │   │   └── BankDetail.svelte   # Right panel - slot grid
│   │   │   ├── import/
│   │   │   │   ├── ImportDialog.svelte
│   │   │   │   └── ImportPreview.svelte
│   │   │   ├── export/
│   │   │   │   ├── ExportDialog.svelte
│   │   │   │   └── ExportPreview.svelte
│   │   │   └── common/
│   │   │       ├── Button.svelte
│   │   │       ├── Modal.svelte
│   │   │       ├── SearchBar.svelte
│   │   │       ├── Sidebar.svelte
│   │   │       └── NewLibraryModal.svelte
│   │   ├── stores/
│   │   │   ├── patches.js          # Patch state management
│   │   │   ├── sequences.js
│   │   │   ├── banks.js
│   │   │   └── ui.js               # UI state (modals, etc.)
│   │   └── utils/
│   │       ├── api.js              # Tauri command wrappers
│   │       └── formatters.js       # Date, size formatting
│   ├── routes/
│   │   ├── +page.svelte            # Main view (patch library)
│   │   ├── +layout.svelte          # App shell
│   │   └── banks/
│   │       └── +page.svelte        # Bank management view
│   ├── app.html
│   └── app.css
│
├── public/                         # Static assets
│   ├── icons/
│   └── fonts/
│
├── package.json
├── package-lock.json
├── vite.config.js
├── svelte.config.js
├── tailwind.config.js
├── tsconfig.json                   # Optional TypeScript
├── .gitignore
├── README.md
├── LICENSE
└── docs/
    ├── application_spec.md         # This document
    └── moog_muse_integration_spec.md
```

---

## 8. Development Phases

### Phase 1: Core Functionality (Week 1-2)
**Goal:** Usable application for personal patch management

**Features:**
- ✅ Database setup with schema
- ✅ Import library from .zip
- ✅ Display patches in list/grid view
- ✅ Search and filter patches
- ✅ Mark favorites
- ✅ Export full library structure
- ✅ Duplicate detection by hash

**UI:**
- Simple list/grid of patches
- File picker dialogs
- Progress indicators

**Deliverable:** macOS .dmg installer

---

### Phase 2: Enhanced Organization (Week 3-4)
**Goal:** Polished daily-use application

**Features:**
- ✅ Bank builder UI (assign patches to banks)
- ✅ Edit patch/sequence metadata (notes)
- ✅ Batch import from bank directories
- ✅ Better search (by favorites, name)
- ✅ Sort options (name, date, favorites)
- ✅ Import validation with preview
- ✅ Export preview before generation

**UI:**
- Visual bank builder (16-slot grid)
- Improved patch cards
- Filter sidebar
- Better modals and dialogs

**Deliverable:** macOS + Linux builds on GitHub

---

### Phase 3: Advanced Features (Month 2+)
**Goal:** Community-ready, polished application

**Features:**
- ✅ Drag-and-drop patch organization
- ✅ Batch operations (delete, move)
- ✅ Import/export user preferences
- ✅ Keyboard shortcuts
- ✅ Dark/light theme toggle
- ✅ Auto-save and backup
- ✅ Statistics (patch count, storage used)

**UI:**
- Drag-and-drop throughout
- Animations and transitions
- Refined visual design
- Tooltips and help text

**Deliverable:** v1.0 release

---

## 9. Setup & Build Instructions

### 9.1 Prerequisites

```bash
# macOS
brew install nodejs rust

# Linux (Ubuntu/Debian)
sudo apt update
sudo apt install nodejs npm curl build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli
```

### 9.2 Project Initialization

```bash
# Create Tauri + Svelte app
npm create tauri-app@latest

# Select options:
# - Framework: Svelte
# - TypeScript: No (optional, can add later)
# - Package manager: npm

# Navigate to project
cd moog-muse-manager

# Install dependencies
npm install
cd src-tauri && cargo build && cd ..

# Install additional dependencies
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

### 9.3 Database Initialization

```rust
// src-tauri/src/db/schema.rs
pub fn initialize_database(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    
    // Execute schema from section 4.1
    conn.execute_batch(include_str!("schema.sql"))?;
    
    Ok(conn)
}
```

### 9.4 Development Commands

```bash
# Run development server (hot reload)
npm run tauri dev

# Build for production
npm run tauri build

# Run tests
cargo test --manifest-path=src-tauri/Cargo.toml
npm run test

# Lint
cargo clippy --manifest-path=src-tauri/Cargo.toml
npm run lint
```

### 9.5 Build Configuration

```json
// tauri.conf.json (key sections)
{
  "package": {
    "productName": "Moog Muse Manager",
    "version": "0.1.0"
  },
  "build": {
    "distDir": "../dist",
    "devPath": "http://localhost:5173"
  },
  "tauri": {
    "bundle": {
      "active": true,
      "targets": ["dmg", "appimage", "deb"],
      "identifier": "com.moogmuse.manager",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/icon.icns"
      ],
      "macOS": {
        "minimumSystemVersion": "12.0"
      }
    },
    "allowlist": {
      "fs": {
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "createDir": true,
        "scope": ["$RESOURCE/*", "$APPDATA/*"]
      },
      "dialog": {
        "open": true,
        "save": true
      }
    }
  }
}
```

---

## 10. Testing Strategy

### 10.1 Unit Tests

**Rust Backend:**
```rust
// src-tauri/src/utils/hash.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let data = b"test patch data";
        let hash = calculate_sha256(data);
        assert_eq!(hash.len(), 64); // SHA-256 is 64 hex chars
    }
    
    #[test]
    fn test_duplicate_detection() {
        // Test that identical files produce same hash
    }
}
```

**Frontend:**
```javascript
// src/lib/utils/api.test.js
import { describe, it, expect } from 'vitest';
import { formatFileSize, formatDate } from './formatters';

describe('formatFileSize', () => {
  it('formats bytes correctly', () => {
    expect(formatFileSize(1024)).toBe('1.00 KB');
    expect(formatFileSize(1048576)).toBe('1.00 MB');
  });
});
```

### 10.2 Integration Tests

```rust
// Test full import workflow
#[tokio::test]
async fn test_import_library_zip() {
    let temp_dir = create_temp_library();
    let result = import_library_zip(temp_dir.path()).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().patches_imported, 16);
}
```

### 10.3 Manual Testing Checklist

**Import Flow:**
- [ ] Import valid library.zip
- [ ] Import bank directory
- [ ] Import with duplicates (should skip)
- [ ] Import invalid structure (should show errors)
- [ ] Import empty library

**Patch Management:**
- [ ] View all patches
- [ ] Search patches by name
- [ ] Filter by favorites
- [ ] Toggle favorite status
- [ ] Add/edit notes
- [ ] Delete patch

**Bank Management:**
- [ ] View all banks
- [ ] Edit bank name
- [ ] Assign patches to slots
- [ ] Clear slots
- [ ] Export library

**Export Flow:**
- [ ] Export full library
- [ ] Verify ZIP structure
- [ ] Verify bank names
- [ ] Verify empty slots created
- [ ] Import exported library (round-trip test)

---

## 11. Deployment & Distribution

### 11.1 GitHub Releases

```bash
# Tag release
git tag -a v0.1.0 -m "Initial release"
git push origin v0.1.0

# GitHub Actions will build for all platforms
# Artifacts: 
#   - moog-muse-manager_0.1.0_amd64.dmg (macOS)
#   - moog-muse-manager_0.1.0_amd64.AppImage (Linux)
#   - moog-muse-manager_0.1.0_amd64.deb (Linux)
```

### 11.2 GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-20.04]
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - run: npm install
      - run: npm run tauri build
      
      - uses: softprops/action-gh-release@v1
        with:
          files: src-tauri/target/release/bundle/**/*
```

### 11.3 Code Signing (Future)

**macOS:**
- Requires Apple Developer account ($99/year)
- Sign with Developer ID certificate
- Notarize with Apple

**Linux:**
- No signing required for initial release
- Consider AppImage signing for v1.0

---

## 12. Non-Functional Requirements

### 12.1 Performance

| Metric | Target | Notes |
|--------|--------|-------|
| App launch time | < 2 seconds | Cold start on modern hardware |
| Import 256 patches | < 5 seconds | From .zip file |
| Search response | < 100ms | For library of 1000+ patches |
| Export library | < 3 seconds | Full 16 banks |
| Database size | ~10MB per 1000 patches | Includes blob storage |
| Memory usage | < 100MB idle | Tauri is lightweight |

### 12.2 Reliability

- Database operations use transactions (ACID)
- Import validates structure before committing
- Export verifies structure before writing
- Auto-save bank changes
- Graceful error handling with user feedback

### 12.3 Usability

- Keyboard shortcuts for common actions
- Undo/redo for destructive operations (Phase 3)
- Progress indicators for long operations
- Clear error messages with actionable suggestions
- Help tooltips for complex features

### 12.4 Security

- No network requests (fully offline)
- Database stored in user's application data directory
- No telemetry or analytics
- Patch files stored as blobs (no external file references)

### 12.5 Compatibility

**macOS:**
- Tested on: Monterey (12.x), Ventura (13.x), Sonoma (14.x)
- Target: macOS 12.0+
- Architecture: Apple Silicon (M1/M2/M3) and Intel

**Linux:**
- Tested on: Ubuntu 22.04, Fedora 38
- Target: Any distro with GTK 3.24+
- Architecture: x86_64

---

## 13. Future Enhancements (Post v1.0)

### Phase 4: Community Features
- Cloud backup (optional)
- Share patches with other users
- Import patches from URL
- Community patch ratings
- Automatic updates

### Phase 5: Advanced Audio
- Audio preview (if Moog provides API)
- Waveform visualization
- Patch analysis/tagging (ML-based)

### Phase 6: Multi-Synth Support
- Support other Moog synthesizers
- Generic patch manager framework
- Plugin architecture for synth adapters

---

## 14. Success Metrics

**Phase 1 Success:**
- [ ] Can import/export full library without errors
- [ ] Can mark favorites
- [ ] macOS build installs and runs

**Phase 2 Success:**
- [ ] Can build custom banks visually
- [ ] Linux build available
- [ ] 10+ users from community
- [ ] No critical bugs reported

**Phase 3 Success:**
- [ ] 100+ GitHub stars
- [ ] Featured on Moog community forums
- [ ] Active user feedback and feature requests
- [ ] v1.0 release with stable API

---

## 15. References

- **Tauri Documentation:** https://tauri.app/
- **Svelte Documentation:** https://svelte.dev/
- **SQLite Documentation:** https://sqlite.org/docs.html
- **Moog Muse Integration:** See `moog_muse_integration_spec.md`
- **GitHub Repository:** [To be created]

---

## 16. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-12 | Initial | Complete application specification |
| 1.1 | 2026-01-18 | Update | Updated schema to reflect multi-library implementation with library-scoped banks, added Library Management API (section 5.4), updated BanksView/BankList/BankDetail components, added Copy Patch workflow |
| 1.2 | 2026-01-18 | Update | **Major data model revision**: Patches/sequences are now global content stores (not library-owned). Removed `library_id` from patches/sequences tables. Added `source_library` informational field. Renamed `bank_patches`/`bank_sequences` to `bank_patch_slots`/`bank_sequence_slots` with composite primary keys. Updated API DTOs to reflect new model. This aligns the data model with actual Moog filesystem structure per moog_spec.md section 2.1. |
| 1.3 | 2026-01-18 | Update | **Added inline bank name editing**: BankDetail component now supports clicking bank name to edit inline. Added keyboard shortcuts (Enter to save, Escape to cancel). Updated BankDetail props to include `libraryId` and `onBankNameUpdate` callback. Updated Workflow 4 to document bank name editing UX. Bank names are persisted via existing `update_bank_name` API command. |

---

## Appendix A: Example API Calls

### Import Library
```javascript
// Frontend: Import button click handler
import { invoke } from '@tauri-apps/api/tauri';

async function handleImport() {
  const result = await invoke('import_library_zip', {
    filePath: '/Users/me/Downloads/library.zip'
  });
  
  console.log(`Imported ${result.patches_imported} patches`);
  console.log(`Skipped ${result.patches_skipped} duplicates`);
}
```

### Search Patches
```javascript
async function searchPatches(query) {
  const patches = await invoke('search_patches', {
    query: query
  });
  
  return patches;
}
```

---

**End of Application Specification**