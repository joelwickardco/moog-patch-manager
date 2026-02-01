# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A cross-platform desktop application for managing Moog Muse synthesizer patch libraries. Built with Tauri 2.0 (Rust backend) + Svelte 5 (frontend) + SQLite.

## Development Commands

### Setup
```bash
npm install                 # Install frontend dependencies
```

### Development
```bash
npm run tauri:dev          # Run in development mode (hot reload enabled)
npm run dev                # Frontend dev server only (port 5173)
npm run lint               # Run ESLint on JavaScript/Svelte files
```

### Building
```bash
npm run tauri:build        # Build production app for current platform
npm run build              # Build frontend assets to dist/
```

Built applications will be in `src-tauri/target/release/bundle/` (platform-specific formats: `.dmg`, `.deb`, `.msi`, `.exe`).

### Testing & Validation
There are no automated tests in this project. Manual testing workflows:
1. Import a library: Use `docs/muse_library_spec.md` for valid library structure
2. Verify database: Check `~/.local/share/muse-patch-manager/patches.db` (Linux), `~/Library/Application Support/com.moog-muse-manager.app/patches.db` (macOS), or AppData on Windows
3. Export and re-import to validate round-trip integrity

## Architecture

### Frontend (Svelte 5)
- **Location:** `src/`
- **State Management:** Svelte 5 runes (`$state`, `$derived`, `$effect`)
- **Key Files:**
  - `src/App.svelte` - Root component with tab switching (`library` vs `banks` views)
  - `src/lib/components/patches/PatchList.svelte` - Main patch grid/list view
  - `src/lib/components/banks/BanksView.svelte` - Bank management with drag-drop
  - `src/lib/utils/api.js` - Wrapper for all Tauri command invocations

**Component Communication:** Props with `$bindable()` for two-way binding + callback functions for events.

### Backend (Rust/Tauri)
- **Location:** `src-tauri/src/`
- **Structure:**
  ```
  src-tauri/src/
  ├── lib.rs                    # Tauri builder setup, AppState initialization
  ├── db/                       # SQLite database layer
  │   ├── connection.rs         # Database wrapper + schema initialization
  │   └── schema.rs             # SQL DDL (15 tables)
  ├── models/                   # DTOs for API responses
  ├── commands/                 # Tauri #[tauri::command] handlers (31+ commands)
  │   ├── libraries.rs
  │   ├── patches.rs
  │   ├── sequences.rs
  │   ├── banks.rs
  │   ├── import.rs
  │   └── export.rs
  ├── moog/                     # Moog library format handling
  │   ├── parser.rs             # Parse filesystem library structure
  │   ├── validator.rs          # Validate library format compliance
  │   └── exporter.rs           # Generate proper Moog directory layout
  └── utils/                    # SHA-256 hashing + ZIP operations
  ```

**AppState Pattern:**
```rust
pub struct AppState {
    pub db: Mutex<db::Database>,
}
```
All commands receive `State<'_, AppState>` and acquire DB lock via `state.db.lock()`.

### Database (SQLite)

**Content-Addressable Storage Pattern:**
- Patches and sequences are **global, reusable assets** stored by SHA-256 hash
- Libraries contain 16 banks, each bank has 16 slots for patches/sequences
- Deduplication: `UNIQUE` constraint on `patches.file_hash` prevents duplicate storage
- Referential integrity: Foreign keys with `ON DELETE CASCADE`

**Critical Tables:**
- `libraries` - Top-level collections
- `banks` - 1-16 per library (FK: library_id)
- `patches` - Content-addressed global store (dedup via file_hash)
- `sequences` - Step sequencer patterns (same pattern as patches)
- `bank_patch_slots` - Assignment of patches to bank positions (1-16 each)
- `bank_sequence_slots` - Assignment of sequences to bank positions

**Non-obvious:** Patches can exist without being assigned to any bank; patches can be reused across multiple banks/libraries.

### Moog Library Format

**Directory Structure:**
```
library/
├── bank01-bank16/              # 16 banks (zero-padded)
│   ├── <name>.bank             # Bank metadata (filename = display name)
│   └── patch01-patch16/        # 16 patches per bank
│       └── <name>.mmp          # Patch file (optional, empty = default patch)
└── sequences/
    └── bank01-bank16/
        └── seq01-seq16/
            └── <name>.mmseq    # Sequence file (optional)
```

**Key Rules:**
1. Folder names are lowercase and zero-padded (`bank01`, `patch05`)
2. `.bank` files are empty; filename (without extension) becomes bank display name
3. `.mmp` and `.mmseq` files are opaque binary blobs (no parsing, stored as-is)
4. Empty patch/sequence folders are valid (loads default/init preset on Moog)

**ZIP Handling:** `moog/parser.rs::find_library_dir()` handles both direct (`library/`) and nested (`SomeFolder/library/`) structures in ZIP files.

See `docs/muse_library_spec.md` for complete Moog integration specification.

### Data Flow (Frontend ↔ Backend)

**Pattern:** Tauri invoke() system for async request-response

1. **Frontend calls** (via `api.js`):
   ```javascript
   import { invoke } from '@tauri-apps/api/core';
   export async function getAllLibraries() {
     return invoke("get_all_libraries");
   }
   ```

2. **Backend handles** (in `commands/`):
   ```rust
   #[tauri::command]
   pub async fn get_all_libraries(state: State<'_, AppState>)
     -> Result<Vec<LibraryDto>, String> {
     // Implementation
   }
   ```

3. **Error Handling:** All commands return `Result<T, String>` - errors serialized as strings to frontend

4. **Registration:** All commands registered in `lib.rs::invoke_handler()`

## Common Development Tasks

### Adding a New Tauri Command

1. Define command handler in appropriate `src-tauri/src/commands/*.rs` file
2. Add function signature with `#[tauri::command]` attribute
3. Register in `src-tauri/src/lib.rs::invoke_handler()`
4. Add wrapper function to `src/lib/utils/api.js`
5. Call from Svelte components

### Modifying Database Schema

1. Update SQL in `src-tauri/src/db/schema.rs`
2. Schema auto-applies on app startup (uses `IF NOT EXISTS` checks)
3. For breaking changes, consider migration strategy (currently none implemented)
4. Update related DTOs in `src-tauri/src/models/`

### Adding New File Type Support

1. Add parser logic in `src-tauri/src/moog/parser.rs`
2. Add export logic in `src-tauri/src/moog/exporter.rs`
3. Update validation in `src-tauri/src/moog/validator.rs`
4. Add database table/fields if needed
5. Update `docs/muse_library_spec.md`

### Handling Moog Library Format Changes

1. Reference `docs/muse_library_spec.md` for current format specification
2. Update parser/exporter in `src-tauri/src/moog/`
3. Validate with real Moog Muse hardware (USB connection required)
4. Test import → export → re-import round-trip integrity

## Critical Patterns

### Auto-tagging on Import
```rust
const PREDEFINED_TAGS: &[&str] = &["Pad", "Lead", "Brass", ...];
```
During import, patch/bank names are scanned for these keywords and matching tags are auto-applied.

### SHA-256 Deduplication
Every patch/sequence is hashed on import. If hash exists in database, the existing record is reused instead of creating a duplicate.

### Empty Slots Handling
- Empty patch folders → `NULL` in `bank_patch_slots.patch_id` → Moog loads default/init patch
- Empty sequence folders → `NULL` in `bank_sequence_slots.sequence_id` → No sequence in slot

### Mutex-based Concurrency
Single `Mutex<Database>` ensures serialized DB access. Acceptable for this use case (patch operations are inherently sequential).

### DTO Layer Separation
- Database models (internal, used in queries)
- DTOs (returned from commands: `PatchDto`, `LibraryDto`)
- Create/Update objects (sent from frontend: `LibraryCreate`, `PatchCreate`)

This allows DB schema changes without breaking the API contract.

## Documentation Requirements

Per `.clinerules`, when making changes update:
- `README.md` - For user-facing changes (features, installation, workflows)
- `docs/muse_library_spec.md` - For Moog format or architecture changes
- This file (`CLAUDE.md`) - For development workflow changes

## Platform-Specific Notes

### macOS
- Unsigned builds require quarantine flag removal: `xattr -cr app.dmg`
- Users must right-click → Open for first launch

### Linux
- Requires webkit2gtk-4.1 support
- Install via `.deb` package: `sudo dpkg -i *.deb`

### Windows
- Uses NSIS installer (`.exe`) or MSI
- SmartScreen warning expected (unsigned)

## Dependencies

**Frontend:**
- Svelte 5 (reactive UI framework)
- TailwindCSS (styling)
- Vite (build tool)
- `@tauri-apps/api` (Tauri JS bindings)

**Backend:**
- Tauri 2.0 (native app framework)
- rusqlite (SQLite bindings with bundled SQLite)
- sha2 (SHA-256 hashing)
- zip (ZIP archive handling)
- walkdir (recursive directory traversal)
- serde/serde_json (serialization)

## Build Configuration

- **Release profile:** Optimized for size (`opt-level = "s"`, LTO enabled, stripped)
- **Tauri config:** `src-tauri/tauri.conf.json` defines app metadata, window settings, bundle targets
- **Frontend output:** `dist/` directory (referenced by Tauri)
