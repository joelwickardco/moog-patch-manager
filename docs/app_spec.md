# Moog Muse Patch Manager - Application Specification

**Version:** 1.3
**Last Updated:** January 17, 2026  
**Target Platforms:** macOS (primary), Linux (secondary)

---

## 1. Executive Summary

The Moog Muse Patch Manager is a desktop application for organizing, categorizing, and managing sound patches and sequences for the Moog Muse synthesizer. The application provides a local database for patch management with features including favorites, user-defined categories, and custom bank organization - capabilities not available in the synthesizer's native filesystem-based organization.

### Key Features
- **Multi-library management:** Import multiple patch libraries from different sources (Moog factory, third-party sound designers, user-created)
- **Automatic library naming:** ZIP filename becomes the library name on import (e.g., `Moog Factory Sounds v2.zip` → "Moog Factory Sounds v2")
- Organize patches with user-defined categories and favorites
- Search and filter across all libraries or within specific libraries
- Each library contains 16 banks × 16 patch slots × 16 sequence slots
- Export complete library structure for transfer to synthesizer
- Duplicate detection via file hashing (across all libraries)
- Metadata management (notes, tags, categories)

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
│  ├── Stores (patches, categories, banks)            │
│  └── Routes (main view, settings)                   │
├─────────────────────────────────────────────────────┤
│  API Layer (Tauri Commands)                         │
│  ├── Patch Management                               │
│  ├── Category Management                            │
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

```sql
-- Source libraries (imported ZIP archives)
-- Each ZIP file import creates a new library entry
CREATE TABLE libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,            -- Derived from ZIP filename (without .zip extension)
    description TEXT,                     -- Optional user description
    source_filename TEXT,                 -- Original ZIP filename for reference
    color TEXT,                           -- Hex color for UI identification (#FF5733)
    patch_count INTEGER DEFAULT 0,        -- Cached count for performance
    sequence_count INTEGER DEFAULT 0,     -- Cached count for performance
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Core patch storage
CREATE TABLE patches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,          -- Which library this patch came from
    name TEXT NOT NULL,
    file_data BLOB NOT NULL,              -- .mmp file contents
    file_hash TEXT NOT NULL UNIQUE,       -- SHA-256 for duplicate detection
    file_size INTEGER NOT NULL,           -- Size in bytes
    is_favorite BOOLEAN DEFAULT 0,
    notes TEXT,                           -- User notes
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
);

-- Sequence storage (independent of patches)
CREATE TABLE sequences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,          -- Which library this sequence came from
    name TEXT NOT NULL,
    file_data BLOB NOT NULL,              -- .mmseq file contents
    file_hash TEXT NOT NULL UNIQUE,       -- SHA-256 for duplicate detection
    file_size INTEGER NOT NULL,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
);

-- User-defined categories (app-only, not in Moog)
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,                           -- Hex color for UI (#FF5733)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Many-to-many: patches can have multiple categories
CREATE TABLE patch_categories (
    patch_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (patch_id, category_id),
    FOREIGN KEY (patch_id) REFERENCES patches(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- Many-to-many: sequences can have multiple categories
CREATE TABLE sequence_categories (
    sequence_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (sequence_id, category_id),
    FOREIGN KEY (sequence_id) REFERENCES sequences(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- Banks within a library (each library has exactly 16 banks)
CREATE TABLE banks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,          -- Banks belong to a library
    bank_number INTEGER NOT NULL,         -- 1-16 within the library
    name TEXT NOT NULL,                   -- Bank name (becomes filename.bank)
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
    UNIQUE (library_id, bank_number)      -- Bank numbers unique within a library
);

-- Bank slots: which patches go in which positions
CREATE TABLE bank_patches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bank_id INTEGER NOT NULL,
    patch_number INTEGER NOT NULL,        -- 1-16 (slot within bank)
    patch_id INTEGER,                     -- NULL = default/empty patch
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (patch_id) REFERENCES patches(id) ON DELETE SET NULL,
    UNIQUE (bank_id, patch_number)
);

-- Bank sequence slots
CREATE TABLE bank_sequences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bank_id INTEGER NOT NULL,
    sequence_number INTEGER NOT NULL,     -- 1-16 (slot within bank)
    sequence_id INTEGER,                  -- NULL = empty sequence slot
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (sequence_id) REFERENCES sequences(id) ON DELETE SET NULL,
    UNIQUE (bank_id, sequence_number)
);

-- Indexes for performance
CREATE INDEX idx_libraries_name ON libraries(name COLLATE NOCASE);
CREATE INDEX idx_patches_library ON patches(library_id);
CREATE INDEX idx_patches_favorite ON patches(is_favorite);
CREATE INDEX idx_patches_name ON patches(name COLLATE NOCASE);
CREATE INDEX idx_patches_hash ON patches(file_hash);
CREATE INDEX idx_sequences_library ON sequences(library_id);
CREATE INDEX idx_sequences_name ON sequences(name COLLATE NOCASE);
CREATE INDEX idx_sequences_hash ON sequences(file_hash);
CREATE INDEX idx_patch_categories_patch ON patch_categories(patch_id);
CREATE INDEX idx_patch_categories_category ON patch_categories(category_id);
CREATE INDEX idx_sequence_categories_sequence ON sequence_categories(sequence_id);
CREATE INDEX idx_banks_library ON banks(library_id);
CREATE INDEX idx_banks_number ON banks(library_id, bank_number);
```

### 4.2 Data Relationships

```
libraries (1) ←→ (16) banks        -- Each library always has exactly 16 banks
libraries (1) ←→ (N) patches
libraries (1) ←→ (N) sequences

banks (1) ←→ (16) bank_patches (1) ←→ (0..1) patches    -- 16 patch slots per bank
banks (1) ←→ (16) bank_sequences (1) ←→ (0..1) sequences -- 16 sequence slots per bank

patches (1) ←→ (N) patch_categories (N) ←→ (1) categories
sequences (1) ←→ (N) sequence_categories (N) ←→ (1) categories
```

**Hierarchy:**
```
Libraries (each containing 16 banks)
├── "Moog Factory Sounds v2"
│   ├── Bank 01: "Bass"
│   │   ├── Patch slots 1-16 (some filled, some empty)
│   │   └── Sequence slots 1-16
│   ├── Bank 02: "Leads"
│   │   └── ...
│   └── ... (Banks 03-16)
├── "Sound Designer Pack - Bass"
│   ├── Bank 01-16 (may have sparse content)
│   └── ...
└── "My Custom Patches"
    └── Bank 01-16
```

### 4.3 Key Constraints

- **Library names must be unique** (derived from ZIP filename, user can rename)
- **Each library has exactly 16 banks** (created on library import)
- **Banks belong to a library** and do not exist outside that context
- Bank numbers 1-16 within each library (unique per library, not globally)
- Patch slots 1-16 per bank
- Sequence slots 1-16 per bank
- Patches identified by SHA-256 hash (no duplicates by content across ALL libraries)
- Sequences identified by SHA-256 hash (no duplicates by content)
- Each patch/sequence belongs to exactly one source library
- Category names must be unique (categories are global, not library-scoped)
- Bank names are user-defined and exported as `<name>.bank` files

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
    library_id: Option<i64>,              // Filter by source library
    is_favorite: Option<bool>,
    category_ids: Option<Vec<i64>>,
    name_contains: Option<String>,
}
```

**PatchDto Structure:**
```rust
struct PatchDto {
    id: i64,
    library_id: i64,
    library_name: String,                 // Denormalized for display
    name: String,
    file_hash: String,
    file_size: i64,
    is_favorite: bool,
    notes: Option<String>,
    categories: Vec<CategoryDto>,
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
    library_id: Option<i64>,              // Filter by source library
    category_ids: Option<Vec<i64>>,
    name_contains: Option<String>,
}
```

**SequenceDto Structure:**
```rust
struct SequenceDto {
    id: i64,
    library_id: i64,
    library_name: String,                 // Denormalized for display
    name: String,
    file_hash: String,
    file_size: i64,
    notes: Option<String>,
    categories: Vec<CategoryDto>,
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

### 5.4 Category Management

```rust
#[tauri::command]
async fn get_all_categories() -> Result<Vec<CategoryDto>, String>

#[tauri::command]
async fn create_category(
    name: String,
    description: Option<String>,
    color: Option<String>
) -> Result<CategoryDto, String>

#[tauri::command]
async fn update_category(
    id: i64,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>
) -> Result<CategoryDto, String>

#[tauri::command]
async fn delete_category(id: i64) -> Result<(), String>

#[tauri::command]
async fn assign_patch_to_category(
    patch_id: i64,
    category_id: i64
) -> Result<(), String>

#[tauri::command]
async fn remove_patch_from_category(
    patch_id: i64,
    category_id: i64
) -> Result<(), String>

#[tauri::command]
async fn assign_sequence_to_category(
    sequence_id: i64,
    category_id: i64
) -> Result<(), String>

#[tauri::command]
async fn remove_sequence_from_category(
    sequence_id: i64,
    category_id: i64
) -> Result<(), String>
```

**CategoryDto Structure:**
```rust
struct CategoryDto {
    id: i64,
    name: String,
    description: Option<String>,
    color: Option<String>,
    patch_count: i64,
    sequence_count: i64,
    created_at: String,
}
```

### 5.4 Bank Management

```rust
#[tauri::command]
async fn get_banks_for_library(
    library_id: i64
) -> Result<Vec<BankDto>, String>  // Returns all 16 banks for a library

#[tauri::command]
async fn get_bank_by_number(
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
async fn assign_patch_to_bank(
    library_id: i64,
    bank_number: i32,
    patch_number: i32,
    patch_id: Option<i64>  // None = default/empty patch
) -> Result<(), String>

#[tauri::command]
async fn assign_sequence_to_bank(
    library_id: i64,
    bank_number: i32,
    sequence_number: i32,
    sequence_id: Option<i64>  // None = empty sequence
) -> Result<(), String>

#[tauri::command]
async fn clear_bank_slot(
    library_id: i64,
    bank_number: i32,
    patch_number: i32
) -> Result<(), String>
```

**BankDto Structure:**
```rust
struct BankDto {
    id: i64,
    library_id: i64,
    bank_number: i32,                        // 1-16 within the library
    name: String,
    description: Option<String>,
    patches: Vec<Option<PatchDto>>,          // 16 slots, Some(patch) or None
    sequences: Vec<Option<SequenceDto>>,     // 16 slots
    created_at: String,
    updated_at: String,
}
```

### 5.5 Import Operations

```rust
#[tauri::command]
async fn import_library_zip(
    file_path: String
) -> Result<ImportResult, String>

#[tauri::command]
async fn import_bank_directory(
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
    library_id: i64,                  // ID of created library
    library_name: String,             // Name derived from ZIP filename
    patches_imported: i32,
    patches_skipped: i32,             // Duplicates by hash (across all libraries)
    sequences_imported: i32,
    sequences_skipped: i32,
    banks_imported: i32,
    errors: Vec<String>,
    warnings: Vec<String>,
}
```

**ValidationResult Structure:**
```rust
struct ValidationResult {
    is_valid: bool,
    structure_errors: Vec<String>,
    missing_files: Vec<String>,
    invalid_files: Vec<String>,
}
```

### 5.6 Export Operations

```rust
#[tauri::command]
async fn export_library(
    library_id: i64,
    output_path: String
) -> Result<ExportResult, String>

#[tauri::command]
async fn preview_export(
    library_id: i64
) -> Result<ExportPreview, String>
```

**ExportResult Structure:**
```rust
struct ExportResult {
    library_id: i64,
    library_name: String,
    output_path: String,
    file_size: i64,
    banks_exported: i32,              // Always 16
    patches_exported: i32,
    sequences_exported: i32,
    empty_slots: i32,
}
```

**ExportPreview Structure:**
```rust
struct ExportPreview {
    library_id: i64,
    library_name: String,
    total_banks: i32,                 // Always 16
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
┌──────────────────────────────────────────────────────────────────┐
│  Moog Muse Patch Manager                        [? Help] [⚙️]    │
├──────────────────────────────────────────────────────────────────┤
│  📚 Libraries  |  🏦 Banks  |  🏷️ Categories                     │
├────────────────┬─────────────────────────────────────────────────┤
│                │  🔍 Search: [__________]  ⭐ ❤️ 🎨 [Library: All]│
│  LIBRARIES     │                                                  │
│  ────────────  │  ┌──────────────────────────────────┐           │
│  📂 All        │  │  Patch Card                      │           │
│  ├─ 🔴 Moog    │  │  Name: Deep Bass                 │           │
│  │  Factory    │  │  Library: Moog Factory v2        │           │
│  ├─ 🟢 Bass    │  │  Categories: Bass, Dark          │           │
│  │  Pack       │  │  [⭐] [❤️] [✏️] [🗑️]             │           │
│  └─ 🔵 My      │  └──────────────────────────────────┘           │
│     Patches    │                                                  │
│                │  [Similar cards in grid layout...]              │
│  FILTERS       │                                                  │
│  ────────────  │                                                  │
│  ☆ Favorites   │                                                  │
│                │                                                  │
│  CATEGORIES    │                                                  │
│  ────────────  │                                                  │
│  🏷️ Bass       │                                                  │
│  🏷️ Lead       │                                                  │
│  🏷️ Pad        │                                                  │
│                │                                                  │
│  [+ Import]    │                                                  │
│  [↗ Export]    │                                                  │
└────────────────┴─────────────────────────────────────────────────┘
```

**Sidebar Hierarchy:**
- **Libraries section** shows all imported libraries with color indicators
- Clicking a library filters to show only patches/sequences from that source
- "All" shows patches from all libraries combined
- Each library displays its name (derived from ZIP filename) and colored dot
- Libraries can be renamed by the user after import

### 6.2 Core Components

#### 6.2.1 LibrarySidebar Component
**Purpose:** Display all imported libraries for filtering

**Props:**
- `libraries: LibraryDto[]`
- `selectedLibraryId: number | null`

**Features:**
- List all imported libraries with color indicators
- "All Libraries" option to show all patches
- Click to filter patches by library
- Library patch/sequence count display
- Context menu: Rename, Change Color, Delete

**State:**
```javascript
let libraries = [];
let selectedLibraryId = null; // null = "All"
```

#### 6.2.2 PatchList Component
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

#### 6.2.3 PatchCard Component
**Purpose:** Display individual patch with metadata

**Props:**
- `patch: PatchDto`

**Features:**
- Patch name (editable on click)
- Category badges (colored)
- Favorite star (toggle)
- Quick actions menu
- Notes display (expandable)

#### 6.2.4 BankBuilder Component (Phase 2+)
**Purpose:** Visual 16-slot grid for building banks

**Features:**
- Drag-and-drop patches into slots
- Bank name editor
- Empty slot indicators
- Batch operations

**Layout:**
```
Bank 03: "My Favorites"
┌─────┬─────┬─────┬─────┐
│  1  │  2  │  3  │  4  │
│ Pch │ Pch │ --- │ Pch │
├─────┼─────┼─────┼─────┤
│  5  │  6  │  7  │  8  │
│ --- │ Pch │ Pch │ Pch │
└─────┴─────┴─────┴─────┘
... (continues to 16)
```

#### 6.2.5 CategoryManager Component
**Purpose:** CRUD operations for categories

**Features:**
- Create new category with color picker
- Edit existing categories
- Delete with confirmation
- View patches per category

#### 6.2.6 ImportDialog Component
**Purpose:** Guide user through import process

**Steps:**
1. Select file (.zip) or directory
2. Validate structure
3. Preview import (show duplicates)
4. Confirm and execute
5. Show results

#### 6.2.7 ExportDialog Component
**Purpose:** Configure and execute export

**Steps:**
1. Preview export (show bank structure)
2. Select output location
3. Confirm
4. Show progress
5. Show results with file location

### 6.3 User Workflows

#### Workflow 1: Import Library
```
User Action → System Response
──────────────────────────────────────────────
1. Click "Import" button
   → Open file picker dialog (accepts .zip files)

2. Select library.zip file
   → Extract library name from filename:
     "Moog Factory Sounds v2.zip" → "Moog Factory Sounds v2"
     "Bass_Pack_2024.zip" → "Bass_Pack_2024"
   → Validate internal structure (bankXX/ folders)
   → Show validation results with library name preview

3. Review import preview
   → Display ImportPreview component showing:
     - Proposed library name (editable)
     - Number of patches found
     - Number of sequences found
     - Duplicates highlighted (by hash across ALL libraries)
     - Library color picker (optional)

4. Confirm import
   → Create new library entry in database
   → Show progress indicator
   → Import all patches/sequences with library_id reference
   → Calculate hashes for duplicate detection

5. View results
   → Show ImportResult summary:
     - Library: "Moog Factory Sounds v2" created
     - Patches imported: 128
     - Sequences imported: 64
     - Duplicates skipped: 3
   → Navigate to newly imported library in sidebar
```

**Library Name Rules:**
- File extension (.zip) is stripped
- Name must be unique; duplicates prompt user to rename
- User can edit the name during import preview
- Original filename stored in `source_filename` for reference

#### Workflow 2: Organize Patches
```
User Action → System Response
──────────────────────────────────────────────
1. Browse patch list
   → Display all patches with search/filter
   
2. Star favorite patches
   → Toggle favorite flag in DB
   → Update UI immediately
   
3. Assign to categories
   → Open category picker
   → Multi-select categories
   → Save associations
   
4. Add notes to patch
   → Click edit icon
   → Open notes editor
   → Save to DB
```

#### Workflow 3: Build and Export Banks
```
User Action → System Response
──────────────────────────────────────────────
1. Navigate to "Banks" tab
   → Show all 16 banks with current assignments
   
2. Select bank to edit (e.g., Bank 03)
   → Open BankBuilder with 16 slots
   
3. Name the bank (e.g., "Live Set")
   → Update bank name in DB
   
4. Drag patches into slots 1-16
   → Update bank_patches table
   → Show visual feedback
   
5. Repeat for other banks
   
6. Click "Export Library"
   → Show ExportPreview
   → Select output location
   → Generate ZIP with full structure
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
category: {
  bass: '#8B4513',
  lead: '#FF6347',
  pad: '#9370DB',
  fx: '#20B2AA',
}

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
│   │   │   ├── libraries.rs        # Library CRUD operations
│   │   │   ├── patches.rs
│   │   │   ├── sequences.rs
│   │   │   ├── categories.rs
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
│   │   │   ├── library.rs          # Library model and DTOs
│   │   │   ├── patch.rs
│   │   │   ├── sequence.rs
│   │   │   ├── category.rs
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
│   │   │   ├── libraries/
│   │   │   │   ├── LibrarySidebar.svelte
│   │   │   │   ├── LibraryCard.svelte
│   │   │   │   └── LibraryColorPicker.svelte
│   │   │   ├── patches/
│   │   │   │   ├── PatchList.svelte
│   │   │   │   ├── PatchCard.svelte
│   │   │   │   ├── PatchDetail.svelte
│   │   │   │   └── PatchSearch.svelte
│   │   │   ├── sequences/
│   │   │   │   ├── SequenceList.svelte
│   │   │   │   └── SequenceCard.svelte
│   │   │   ├── banks/
│   │   │   │   ├── BankList.svelte
│   │   │   │   ├── BankBuilder.svelte  # Phase 2
│   │   │   │   └── BankSlot.svelte
│   │   │   ├── categories/
│   │   │   │   ├── CategoryManager.svelte
│   │   │   │   ├── CategoryBadge.svelte
│   │   │   │   └── CategoryPicker.svelte
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
│   │   │       └── Sidebar.svelte
│   │   ├── stores/
│   │   │   ├── libraries.js        # Library state and selection
│   │   │   ├── patches.js          # Patch state management
│   │   │   ├── sequences.js
│   │   │   ├── categories.js
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
- ✅ Basic category management (CRUD)
- ✅ Assign patches to categories
- ✅ Export full library structure
- ✅ Duplicate detection by hash

**UI:**
- Simple list/grid of patches
- Basic forms for categories
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
- ✅ Better search (by category, favorites, name)
- ✅ Sort options (name, date, favorites)
- ✅ Import validation with preview
- ✅ Export preview before generation

**UI:**
- Visual bank builder (16-slot grid)
- Improved patch cards with categories
- Filter sidebar
- Better modals and dialogs

**Deliverable:** macOS + Linux builds on GitHub

---

### Phase 3: Advanced Features (Month 2+)
**Goal:** Community-ready, polished application

**Features:**
- ✅ Drag-and-drop patch organization
- ✅ Batch operations (delete, categorize, move)
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
- [ ] Filter by category
- [ ] Toggle favorite status
- [ ] Add/edit notes
- [ ] Delete patch

**Category Management:**
- [ ] Create category
- [ ] Edit category
- [ ] Delete category with patches
- [ ] Assign patch to multiple categories

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
- Auto-save category and bank changes
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
- [ ] Can organize patches into categories
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
| 1.1 | 2026-01-17 | Update | Added multi-library support: libraries table, library filtering, import workflow with ZIP filename as library name |
| 1.2 | 2026-01-17 | Update | Removed unimplemented get_library_statistics command; fixed LibraryDto.source_filename to Option<String> |
| 1.3 | 2026-01-17 | Update | Banks are now library-scoped: each library has exactly 16 banks, banks table has library_id FK, bank APIs require library_id |

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

### Assign Category
```javascript
async function assignCategory(patchId, categoryId) {
  await invoke('assign_patch_to_category', {
    patchId,
    categoryId
  });
}
```

---

**End of Application Specification**