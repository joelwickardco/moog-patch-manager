# Moog Muse Patch Manager

A desktop application for managing, organizing, and transferring sound patches for the Moog Muse synthesizer.

<img width="1569" height="977" alt="Screenshot 2026-01-21 at 9 06 35 PM" src="https://github.com/user-attachments/assets/3cb7a0fe-ac5e-460f-9b9a-41937d7a7ec6" />


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

**Important**: This application is unsigned and requires special steps to install on macOS.

#### Installation Steps:

1. Download the latest `.dmg` file from the [releases page](https://github.com/joelwickardco/moog-patch-manager/releases)

2. **Remove the quarantine flag** (required for unsigned apps):
   - Open Terminal
   - Run this command (replace the path with your actual download location):
   ```bash
   xattr -cr ~/Downloads/Moog\ Muse\ Manager_*.dmg
   ```

3. Open the `.dmg` file and drag the app to your Applications folder

4. **First launch security approval**:
   - Right-click (or Control-click) on the app in Applications
   - Select **"Open"** from the menu
   - Click **"Open"** in the security dialog
   - Alternatively, if you try to open normally and it's blocked:
     - Go to **System Settings → Privacy & Security**
     - Scroll down to find "Moog Muse Manager was blocked"
     - Click **"Open Anyway"**

5. The app will now launch normally

**Note**: You only need to do this once. macOS will remember your choice for future launches.

#### Why These Steps Are Needed

This app is not code-signed or notarized with an Apple Developer certificate (which costs $99/year). macOS blocks unsigned apps downloaded from the internet as a security measure. The steps above safely bypass this restriction for apps you trust.

### Linux

Download the `.deb` package from the [releases page](https://github.com/joelwickardco/moog-patch-manager/releases) and install it:

```bash
sudo dpkg -i moog-muse-manager_*.deb
```

If you encounter dependency errors:
```bash
sudo apt-get install -f
```

**Supported distributions**: Ubuntu 22.04+, Debian 11+, or any distribution with webkit2gtk-4.1 support.

See `INSTALL_LINUX.txt` in the release for detailed installation instructions and troubleshooting.

### Windows

Download either the `.msi` or `.exe` installer from the [releases page](https://github.com/joelwickardco/moog-patch-manager/releases) and run it.

**Important**: Windows will show a "Windows protected your PC" SmartScreen warning because this application is not code-signed. This is expected and safe. Click **"More info"** then **"Run anyway"** to proceed with installation.

**Requirements**: Windows 10 (64-bit) or newer.

See `INSTALL_WINDOWS.txt` in the release for detailed installation instructions and troubleshooting.

### Building from Source

#### Prerequisites

- Node.js 18 or higher
- Rust 1.70 or higher
- Platform-specific dependencies:
  - **macOS**: macOS 12.0 or higher
  - **Linux**: webkit2gtk-4.1, build-essential, and other dev libraries
  - **Windows**: Windows 10 SDK

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

### macOS
- macOS 12.0 or higher
- 100 MB free disk space
- USB port for connecting Moog Muse

### Linux
- Ubuntu 22.04+ / Debian 11+ (or equivalent)
- webkit2gtk-4.1 support
- 100 MB free disk space
- USB port for connecting Moog Muse

### Windows
- Windows 10 (64-bit) or newer
- 100 MB free disk space
- USB port for connecting Moog Muse

## Troubleshooting

### "Damaged" or "Can't be opened" error on macOS

If you see an error saying the DMG or app is "damaged" or "can't be opened":

**Solution**: Remove the quarantine attribute that macOS adds to downloaded files:
```bash
# For the DMG file:
xattr -cr ~/Downloads/Moog\ Muse\ Manager_*.dmg

# Or for the app itself:
xattr -cr /Applications/Moog\ Muse\ Manager.app
```

Then try opening the app again by right-clicking and selecting "Open".

### App blocked by macOS security

If the app is blocked by macOS with an "unidentified developer" warning:
1. Right-click the app and select **"Open"** (this gives you an override option)
2. Or go to **System Settings → Privacy & Security**
3. Find the blocked app notification
4. Click **"Open Anyway"**
5. Confirm in the dialog

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
