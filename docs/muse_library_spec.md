# Moog Muse Integration Specification

**Version:** 1.0  
**Last Updated:** January 2026  
**Purpose:** Complete specification for Moog Muse patch library format and integration

---

## 1. Overview

The Moog Muse synthesizer uses a filesystem-based organization for patches and sequences. This specification defines:
- Directory structure for complete libraries
- File formats and naming conventions
- Bank organization (16 banks × 16 patches/sequences)
- Import/export workflows
- Validation rules

---

## 2. Library Directory Structure

### 2.1 Complete Library Layout

```
<library-root>/
└── library/
    ├── bank01/
    │   ├── <bankname>.bank          # Bank metadata file
    │   ├── patch01/
    │   │   └── <patchname>.mmp      # Patch file (optional, can be empty for default)
    │   ├── patch02/
    │   │   └── <patchname>.mmp
    │   ├── ... (patches 03-15)
    │   └── patch16/
    │       └── <patchname>.mmp
    ├── bank02/
    │   ├── <bankname>.bank
    │   └── ... (16 patch folders)
    ├── ... (banks 03-15)
    ├── bank16/
    │   ├── <bankname>.bank
    │   └── ... (16 patch folders)
    └── sequences/
        ├── bank01/
        │   ├── seq01/
        │   │   └── <sequencename>.mmseq    # Sequence file (optional)
        │   ├── seq02/
        │   │   └── <sequencename>.mmseq
        │   ├── ... (sequences 03-15)
        │   └── seq16/
        │       └── <sequencename>.mmseq
        ├── bank02/
        │   └── ... (16 sequence folders)
        ├── ... (banks 03-15)
        └── bank16/
            └── ... (16 sequence folders)
```

### 2.2 Structure Rules

1. **Root must contain `library/` directory**
2. **16 banks numbered `bank01` through `bank16`** (zero-padded)
3. **Each bank contains 16 patch folders** (`patch01` through `patch16`)
4. **Sequences subdirectory mirrors bank structure** with `seq01` through `seq16`
5. **All folder names use lowercase**
6. **Numbers are zero-padded to 2 digits**

---

## 3. File Types and Formats

### 3.1 Bank Metadata Files (`.bank`)

**Format:** Simple metadata file  
**Naming:** `<bank-display-name>.bank`  
**Location:** Inside each `bankXX/` folder  
**Required:** Yes

**Purpose:** Defines the display name for the bank

**Example:**
```
library/bank03/pads.bank
```
Result: Bank 03 displays as "pads" in the Moog Muse interface

**Rules:**
- Filename (without `.bank` extension) becomes bank display name
- No content inside file (empty or ignored)
- One `.bank` file per bank folder
- If missing, bank uses default name "Bank XX"

**Character Restrictions:**
- Allowed: alphanumeric, spaces, hyphens, underscores
- Avoid: special characters that might cause filesystem issues
- Recommended max length: 32 characters

---

### 3.2 Patch Files (`.mmp`)

**Format:** Proprietary text-based format (JSON-esque but not strict JSON)  
**Naming:** `<patch-display-name>.mmp`  
**Location:** Inside each `patchXX/` folder  
**Required:** No (folder can be empty for default patch)

**Purpose:** Contains all parameters for a synthesizer sound preset

**Example:**
```
library/bank03/patch05/vox humana.mmp
```
Result: Patch displays as "vox humana" in slot 5 of bank 3

**Rules:**
- Patch name comes from filename only (not stored inside file)
- One `.mmp` file per patch folder (multiple files = undefined behavior)
- File contents are opaque binary/text blob (application treats as blob)
- Empty folder = default/init patch (synthesizer loads init preset at runtime)

**File Characteristics:**
- **Size:** Typically 2-10 KB
- **Encoding:** Text-based but not strict JSON
- **Structure:** Key-value pairs with nested structures
- **No parsing required:** Application stores as-is

**Default Patch Behavior:**
When `patchXX/` folder exists but contains no `.mmp` file:
- Moog Muse loads its internal "init patch" (blank starting preset)
- Application should create empty folder during export
- Database can store `NULL` for `patch_id` in `bank_patches` table

---

### 3.3 Sequence Files (`.mmseq`)

**Format:** Proprietary text-based format (similar to `.mmp`)  
**Naming:** `<sequence-display-name>.mmseq`  
**Location:** Inside each `seqXX/` folder under `sequences/bankXX/`  
**Required:** No (folder can be empty)

**Purpose:** Contains step-sequencer patterns and automation

**Example:**
```
library/sequences/bank03/seq05/new sequence.mmseq
```
Result: Sequence displays as "new sequence" in sequence slot 5 of bank 3

**Rules:**
- Sequence name comes from filename only
- One `.mmseq` file per sequence folder
- Empty folder = no sequence in that slot
- Sequences are independent of patches (no linkage)

**File Characteristics:**
- **Size:** Typically 1-5 KB
- **Encoding:** Text-based
- **No parsing required:** Store as blob

---

## 4. Naming Conventions

### 4.1 Folder Naming

| Type | Pattern | Example | Notes |
|------|---------|---------|-------|
| Library root | `library/` | `library/` | Hardcoded name |
| Bank folders | `bankXX/` | `bank03/` | XX = 01-16, zero-padded |
| Patch folders | `patchXX/` | `patch05/` | XX = 01-16, zero-padded |
| Sequence folders | `seqXX/` | `seq12/` | XX = 01-16, zero-padded |

**Case Sensitivity:** All lowercase (Moog Muse expects lowercase)

### 4.2 File Naming

| Type | Pattern | Example | Notes |
|------|---------|---------|-------|
| Bank metadata | `<name>.bank` | `pads.bank` | Name becomes bank display name |
| Patch files | `<name>.mmp` | `deep bass.mmp` | Name becomes patch display name |
| Sequence files | `<name>.mmseq` | `arpeggio 1.mmseq` | Name becomes sequence display name |

**Allowed Characters:**
- Alphanumeric: `a-z`, `A-Z`, `0-9`
- Spaces: ` `
- Punctuation: `-`, `_`, `.` (in name, not extension)
- **Avoid:** `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`

**Recommended Practices:**
- Keep names under 32 characters
- Use descriptive names (e.g., "Deep Analog Bass" not "patch1")
- Consistent naming scheme (e.g., "Bass - Sub", "Bass - Reese")

---

## 5. Filesystem to Database Mapping

### 5.1 Import Mapping

| Filesystem Element | Database Table | Field Mapping |
|-------------------|----------------|---------------|
| `library/bankXX/` | `banks` | `bank_number` = XX (1-16) |
| `<name>.bank` | `banks.name` | Filename without extension |
| `library/bankXX/patchYY/` | `bank_patches` | `bank_id`, `patch_number` = YY (1-16) |
| `<name>.mmp` | `patches` | `name` = filename, `file_data` = contents |
| `library/sequences/bankXX/seqYY/` | `bank_sequences` | `bank_id`, `sequence_number` = YY (1-16) |
| `<name>.mmseq` | `sequences` | `name` = filename, `file_data` = contents |

### 5.2 Example Mapping

**Filesystem:**
```
library/
  bank03/
    pads.bank
    patch05/
      vox humana.mmp
```

**Database:**
```sql
-- banks table
INSERT INTO banks (bank_number, name) VALUES (3, 'pads');

-- patches table
INSERT INTO patches (name, file_data, file_hash, file_size)
VALUES ('vox humana', <blob>, '<sha256>', 4096);

-- bank_patches table
INSERT INTO bank_patches (bank_id, patch_number, patch_id)
VALUES (1, 5, 1);
```

---

## 6. Import Algorithm

### 6.1 Import from ZIP File

```
Input: library.zip file path
Output: ImportResult with counts and errors

Algorithm:
1. Extract ZIP to temporary directory
2. Validate structure (see section 6.3)
3. For each bank01-bank16:
   a. Read <name>.bank file → bank name
   b. Create/update bank in database
   c. For each patch01-patch16:
      - If .mmp file exists:
        * Calculate SHA-256 hash
        * Check if hash exists in database
        * If duplicate: skip, increment skipped_count
        * If new: insert patch, store blob
        * Link to bank via bank_patches table
      - If folder empty:
        * Insert NULL patch_id in bank_patches (default patch)
4. For each sequences/bank01-bank16:
   a. For each seq01-seq16:
      - If .mmseq file exists:
        * Calculate SHA-256 hash
        * Check if hash exists
        * If new: insert sequence
        * Link to bank via bank_sequences table
      - If folder empty:
        * Insert NULL sequence_id (empty slot)
5. Return ImportResult
```

### 6.2 Import from Bank Directory

```
Input: Single bankXX/ directory path
Output: ImportResult

Algorithm:
1. Validate directory structure
2. Extract bank number from folder name (e.g., bank03 → 3)
3. Read <name>.bank file → bank name
4. Process patches (same as step 3c above)
5. Look for corresponding sequences/bankXX/ directory
6. Process sequences if found
7. Return ImportResult
```

### 6.3 Validation Rules

**Structure Validation:**
```python
def validate_library_structure(path):
    errors = []
    warnings = []
    
    # Must have library/ root
    if not exists(path + "/library"):
        errors.append("Missing 'library/' root directory")
        return ValidationResult(False, errors, warnings)
    
    # Check for 16 banks
    for i in range(1, 17):
        bank_dir = f"library/bank{i:02d}"
        if not exists(path + "/" + bank_dir):
            warnings.append(f"Missing {bank_dir}")
            continue
        
        # Check for .bank file
        bank_files = find_files(bank_dir, "*.bank")
        if len(bank_files) == 0:
            errors.append(f"{bank_dir}: Missing .bank file")
        elif len(bank_files) > 1:
            warnings.append(f"{bank_dir}: Multiple .bank files, using first")
        
        # Check for 16 patch folders
        for j in range(1, 17):
            patch_dir = f"{bank_dir}/patch{j:02d}"
            if not exists(path + "/" + patch_dir):
                warnings.append(f"Missing {patch_dir}")
            else:
                # Check for .mmp file (optional)
                mmp_files = find_files(patch_dir, "*.mmp")
                if len(mmp_files) > 1:
                    warnings.append(f"{patch_dir}: Multiple .mmp files")
    
    # Check sequences/ structure (similar to above)
    # ...
    
    is_valid = len(errors) == 0
    return ValidationResult(is_valid, errors, warnings)
```

---

## 7. Export Algorithm

### 7.1 Export Full Library

```
Input: Output file path (e.g., /Users/me/Desktop/library.zip)
Output: ExportResult

Algorithm:
1. Create temporary directory structure
2. For bank_number 1 to 16:
   a. Create library/bankXX/ directory
   b. Query database for bank with bank_number = XX
   c. Create <bank.name>.bank file (empty file)
   d. For patch_number 1 to 16:
      - Create library/bankXX/patchYY/ directory
      - Query bank_patches for patch_id at this slot
      - If patch_id is NULL:
        * Leave directory empty (default patch)
      - If patch_id exists:
        * Query patches table for file_data
        * Write patches.name + ".mmp" file with file_data blob
   e. Create library/sequences/bankXX/ directory
   f. For sequence_number 1 to 16:
      - Create library/sequences/bankXX/seqYY/ directory
      - Query bank_sequences for sequence_id
      - If sequence_id is NULL:
        * Leave directory empty
      - If sequence_id exists:
        * Query sequences table for file_data
        * Write sequences.name + ".mmseq" file with file_data blob
3. Create ZIP archive from temporary directory
4. Clean up temporary directory
5. Return ExportResult with path and stats
```

### 7.2 Export Preview

Before export, provide user with preview:
```rust
struct ExportPreview {
    total_banks: i32,              // Always 16
    banks_with_custom_names: i32,  // Banks with user-defined names
    total_patches: i32,            // Non-NULL patch slots
    total_sequences: i32,          // Non-NULL sequence slots
    empty_patch_slots: i32,        // NULL patch_id slots (default patches)
    empty_sequence_slots: i32,     // NULL sequence_id slots
    estimated_size: i64,           // Sum of file_size fields + structure overhead
}
```

---

## 8. Edge Cases and Error Handling

### 8.1 Import Edge Cases

| Scenario | Behavior |
|----------|----------|
| Duplicate patches (same hash) | Skip import, increment `patches_skipped` |
| Multiple .mmp files in patch folder | Use first found, log warning |
| Missing .bank file | Use default name "Bank XX", log warning |
| Invalid bank number (bank00, bank17) | Skip, log error |
| Corrupted .mmp file | Import as-is (blob storage), let Moog validate |
| Non-standard folder names | Skip, log error |
| Empty library.zip | Return empty ImportResult, no errors |
| Partial banks (missing patches) | Import available patches, fill rest with NULL |

### 8.2 Export Edge Cases

| Scenario | Behavior |
|----------|----------|
| All banks empty | Export structure with all empty patch folders |
| Bank with no custom name | Use "Bank XX" as .bank filename |
| Patch name with special chars | Sanitize filename (replace `/\:` with `_`) |
| Database corruption (missing patch blob) | Log error, skip slot |
| Disk space full | Fail gracefully, show error to user |
| Write permission denied | Show error, suggest alternative location |

### 8.3 Validation Warnings vs Errors

**Errors (prevent import):**
- Missing `library/` root
- Invalid directory structure
- Missing .bank files

**Warnings (allow import but notify user):**
- Missing patches (creates empty slots)
- Multiple files in single folder
- Non-standard file sizes

---

## 9. File Transfer Workflow

### 9.1 From Computer to Moog Muse

```
1. User exports library from application
   → Generates library.zip at chosen location

2. User connects Moog Muse to computer via USB
   → Muse appears as USB storage device

3. User copies library.zip to Muse storage
   → May need to unzip (check Moog manual)

4. User safely ejects Muse
   → Disconnect USB

5. Muse reads library structure on next boot
   → Banks appear in Muse interface
```

**Note:** This workflow is external to the application. The application only generates the correctly formatted `.zip` file.

### 9.2 From Moog Muse to Computer

```
1. User connects Moog Muse via USB
   → Muse appears as USB storage

2. User locates library/ directory on Muse
   → Copy entire directory or create .zip

3. User imports into application
   → Use "Import Library" feature

4. Application parses structure and imports
   → Patches stored in database
```

---

## 10. Example Structures

### 10.1 Minimal Valid Library

```
library/
  bank01/
    factory.bank
    patch01/
      init.mmp
    patch02/
    patch03/
    ... (empty patch folders 04-16)
  bank02/
    user.bank
    patch01/
    ... (all empty)
  ... (banks 03-16 with empty patches)
  sequences/
    bank01/
      seq01/
      ... (all empty)
    ... (banks 02-16)
```

### 10.2 Fully Populated Bank

```
library/
  bank03/
    pads.bank
    patch01/
      ambient wash.mmp
    patch02/
      cathedral.mmp
    patch03/
      deep space.mmp
    ... (patches 04-16 all populated)
  sequences/
    bank03/
      seq01/
        ambient arp.mmseq
      seq02/
      ... (mix of populated and empty)
```

### 10.3 Real-World Example

```
library.zip contents:
library/
  bank01/
    factory.bank
    patch01/
      init.mmp
    patch02/
      analog bass.mmp
    patch03-16/
      ... (mix of patches)
  bank02/
    bass.bank
    patch01/
      sub bass.mmp
    patch02/
      reese bass.mmp
    ... (more bass patches)
  bank03/
    leads.bank
    ... (lead patches)
  bank04/
    pads.bank
    ... (pad patches)
  bank05/
    fx.bank
    ... (effect patches)
  bank06-16/
    user.bank
    ... (empty or user patches)
  sequences/
    bank01/
      seq01/
        factory seq 1.mmseq
      seq02-16/
        ... (more sequences)
    bank02-16/
      ... (user sequences)
```

---

## 11. Hash-Based Duplicate Detection

### 11.1 Hashing Strategy

**Algorithm:** SHA-256  
**Input:** Raw file contents of `.mmp` or `.mmseq`  
**Storage:** 64-character hexadecimal string

**Rationale:**
- Cryptographically secure (collision-resistant)
- Fast to compute
- Standard library support in Rust

### 11.2 Duplicate Detection Logic

```rust
fn import_patch(file_data: &[u8], name: &str) -> Result<i64, ImportError> {
    let hash = calculate_sha256(file_data);
    
    // Check if hash exists
    if let Some(existing_id) = find_patch_by_hash(&hash)? {
        return Err(ImportError::Duplicate(existing_id));
    }
    
    // Insert new patch
    let patch_id = insert_patch(name, file_data, &hash)?;
    Ok(patch_id)
}
```

**User Experience:**
- Duplicates are silently skipped during import
- Import summary shows: "Imported 45 patches, skipped 3 duplicates"
- User can optionally view list of skipped patches

---

## 12. Implementation Checklist

### 12.1 Import Implementation

- [ ] ZIP extraction to temporary directory
- [ ] Directory structure validation
- [ ] .bank file parsing (extract filename)
- [ ] .mmp file reading and hashing
- [ ] .mmseq file reading and hashing
- [ ] Duplicate detection by hash
- [ ] Database insertion with transactions
- [ ] Temporary directory cleanup
- [ ] Error logging and user feedback

### 12.2 Export Implementation

- [ ] Query all banks from database
- [ ] Create temporary directory structure
- [ ] Create .bank files with correct names
- [ ] Create patch directories (01-16 per bank)
- [ ] Write .mmp files from database blobs
- [ ] Create sequence directories
- [ ] Write .mmseq files from database blobs
- [ ] Handle empty slots (create empty directories)
- [ ] ZIP creation
- [ ] Move ZIP to user-selected location
- [ ] Cleanup temporary files

### 12.3 Validation Implementation

- [ ] Check for library/ root
- [ ] Validate bank folder names (bank01-bank16)
- [ ] Check for .bank files
- [ ] Validate patch folder names (patch01-patch16)
- [ ] Validate sequence folder names (seq01-seq16)
- [ ] Collect warnings for missing optional files
- [ ] Return structured validation result

---

## 13. Testing Scenarios

### 13.1 Import Tests

**Test 1: Valid complete library**
- Input: library.zip with all 16 banks, all patches populated
- Expected: All 256 patches imported, 0 skipped

**Test 2: Partial library**
- Input: library.zip with bank01-bank04 only, sparse patches
- Expected: Available patches imported, remaining banks empty

**Test 3: Duplicate patches**
- Input: library.zip with same patch in multiple banks
- Expected: Patch imported once, duplicates skipped

**Test 4: Empty patch folders**
- Input: Bank with mix of populated and empty patch folders
- Expected: Populated patches imported, empty slots marked as NULL

**Test 5: Invalid structure**
- Input: ZIP without library/ root
- Expected: Validation error, no import

**Test 6: Missing .bank files**
- Input: Bank directory without .bank file
- Expected: Import succeeds with warning, default bank name used

### 13.2 Export Tests

**Test 1: Full library export**
- Input: Database with all 16 banks populated
- Expected: library.zip with correct structure, all files present

**Test 2: Partial library export**
- Input: Database with only bank01-bank03 populated
- Expected: library.zip with all 16 banks, banks 04-16 have empty patches

**Test 3: Empty slots**
- Input: Bank with some NULL patch_id slots
- Expected: Empty patch directories created in export

**Test 4: Special characters in names**
- Input: Patch named "Bass/Lead : Test"
- Expected: Filename sanitized to "Bass_Lead _ Test.mmp"

**Test 5: Round-trip**
- Input: Import library, modify some patches, export
- Expected: Re-import produces identical database state

---

## 14. Glossary

| Term | Definition |
|------|------------|
| **Library** | Complete collection of all banks, patches, and sequences |
| **Bank** | Container for 16 patches or sequences (16 banks total) |
| **Patch** | Single sound preset (`.mmp` file) |
| **Sequence** | Step-sequencer pattern (`.mmseq` file) |
| **Default Patch** | Empty patch slot that loads Moog's init preset at runtime |
| **Bank Metadata** | `.bank` file containing bank display name |
| **Hash** | SHA-256 checksum of file contents for duplicate detection |
| **Slot** | Position within a bank (1-16 for patches, 1-16 for sequences) |
| **Blob** | Binary Large Object - raw file data stored in database |

---

## 15. Moog Muse Hardware Reference

**Note:** The following information may need verification against official Moog documentation.

**USB Connection:**
- Moog Muse connects via USB-C
- Appears as USB Mass Storage device
- No drivers required (macOS/Linux)

**File System:**
- Read/write access when connected
- Library structure persists on internal storage
- Changes take effect on next power cycle

**Limitations:**
- No known file size limits per patch
- Unknown behavior with >16 banks (test needed)
- Unknown behavior with corrupted patch files

---

## 16. References

- **Moog Muse Manual:** [Check official Moog website]
- **Community Forums:** [Moog forums for user-generated patches]
- **Patch Libraries:** [Third-party patch collections]

---

## 17. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-12 | Initial | Complete Moog Muse integration specification |

---

**End of Moog Muse Integration Specification**