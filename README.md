# Moog Muse Patch Manager

A desktop application for managing, organizing, and transferring sound patches for the Moog Muse synthesizer.

## Overview

The Moog Muse Patch Manager is a cross-platform desktop application that allows you to manage your Moog Muse patch library on your computer. Import patches from your synthesizer, organize them into banks, and export them back to your device with ease.

## Features

- **Import Patch Libraries**: Load patches from ZIP files or directly from your Moog Muse device's filesystem
- **Bank Management**: Organize patches into 16 banks with 16 patches each (256 total patches)
- **Sequence Management**: Handle step-sequencer patterns alongside your patches
- **Duplicate Detection**: Automatically detect and skip duplicate patches using SHA-256 hashing
- **Browse & Search**: View and search through your patch collection
- **Export Libraries**: Generate properly formatted ZIP files ready to transfer to your Moog Muse
- **Database Storage**: All patches stored locally in a SQLite database

## Workflows

### Importing Patches from Moog Muse

1. Connect your Moog Muse to your computer via USB (it will appear as a USB storage device)
2. In the app, select "Import Library"
3. Navigate to the `library/` directory on your Moog Muse
4. The app will import all patches and sequences, automatically detecting duplicates
5. Your patches are now stored in the local database

### Organizing Your Patch Library

1. Browse your imported patches organized by bank and patch number
2. View patch details including name, bank assignment, and position
3. Search and filter patches by name or category

### Exporting Patches to Moog Muse

1. Select "Export Library" from the app
2. Choose a destination for the generated ZIP file
3. The app creates a properly formatted `library.zip` file
4. Connect your Moog Muse via USB
5. Copy the library structure to your Moog Muse (may need to unzip first)
6. Safely eject the device
7. Power cycle your Moog Muse to load the new patches

## Installation

### macOS

1. Download the latest `.dmg` file from the releases page
2. Open the `.dmg` and drag the app to your Applications folder
3. **Important**: This application is unsigned and will require security approval:
   - When you first try to open the app, macOS will block it
   - Go to **System Preferences → Privacy & Security**
   - Scroll down to find "Moog Muse Manager was blocked from opening"
   - Click **"Open Anyway"**
   - Confirm by clicking **"Open"** in the dialog
4. The app will now launch normally

**Note**: You only need to approve the app once. macOS will remember your choice for future launches.

### Building from Source

#### Prerequisites

- Node.js 18 or higher
- Rust 1.70 or higher
- macOS 12.0 or higher (for macOS builds)

#### Steps

```bash
# Clone the repository
git clone <repository-url>
cd muse-patch-manager

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Technical Details

### Architecture

- **Frontend**: Svelte 5 with TailwindCSS for a modern, responsive UI
- **Backend**: Tauri (Rust) for native performance and file system access
- **Database**: SQLite for local patch storage with SHA-256 hash-based deduplication
- **File Handling**: Native Rust libraries for ZIP extraction and file operations

### Patch Library Format

The Moog Muse uses a specific directory structure:

```
library/
  bank01-bank16/          # 16 banks
    <name>.bank           # Bank metadata file
    patch01-patch16/      # 16 patches per bank
      <name>.mmp          # Patch file (optional)
  sequences/
    bank01-bank16/        # Sequence banks
      seq01-seq16/        # 16 sequences per bank
        <name>.mmseq      # Sequence file (optional)
```

The app handles all the complexity of this structure automatically.

### Supported File Types

- `.mmp` - Moog Muse Patch files
- `.mmseq` - Moog Muse Sequence files
- `.bank` - Bank metadata files
- `.zip` - Compressed library archives

## System Requirements

- macOS 12.0 or higher
- 100 MB free disk space
- USB port for connecting Moog Muse

## Troubleshooting

### App won't open on macOS

If the app is blocked by macOS security:
1. Go to **System Preferences → Privacy & Security**
2. Find the blocked app notification
3. Click **"Open Anyway"**
4. Confirm in the dialog

### Patches not appearing on Moog Muse

- Ensure you copied the entire `library/` directory structure
- Power cycle your Moog Muse after copying files
- Verify the directory structure matches the expected format

### Import fails with validation errors

- Check that your library has a `library/` root directory
- Verify bank folders are named `bank01` through `bank16`
- Ensure each bank has a `.bank` file

## Development

### Project Structure

```
muse-patch-manager/
├── src/                  # Svelte frontend
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── commands/    # Tauri command handlers
│   │   ├── db/         # Database operations
│   │   ├── models/     # Data models
│   │   └── moog/       # Moog-specific logic
├── docs/               # Documentation
└── dist/              # Built frontend assets
```

### Available Scripts

- `npm run dev` - Start Vite development server
- `npm run build` - Build frontend for production
- `npm run tauri:dev` - Run Tauri in development mode
- `npm run tauri:build` - Build Tauri app for production

## License

[Add your license here]

## Acknowledgments

Built for the Moog Muse synthesizer community. Moog and Muse are trademarks of Moog Music Inc.
