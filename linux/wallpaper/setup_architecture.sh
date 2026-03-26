#!/bin/bash

# ==============================================================================
# Wallpaper-Meister Architecture Setup Script
# ==============================================================================
# This script creates the necessary directory structure and placeholder files
# for the application's core logic. It is safe to run multiple times, as it
# will not overwrite any existing files or directories.
# ==============================================================================

echo "🚀 Starting Wallpaper-Meister project setup..."
echo ""

# --- Create Core Directories ---
echo "➡ Ensuring core directory structure exists..."
mkdir -p src/components
mkdir -p src/core/platform
echo "✅ Directories are ready."
echo ""

# --- Create Platform Abstraction Files ---

# types.ts
FILE_PATH="src/core/platform/types.ts"
if [ ! -f "$FILE_PATH" ]; then
    echo "➡ Creating file: $FILE_PATH"
    cat << 'EOF' > "$FILE_PATH"
// Defines the "contract" that any platform-specific service must follow.
// This allows our core logic to be completely decoupled from the OS.

export interface IPlatformService {
  /**
   * Sets the desktop wallpaper.
   * @param imagePath The absolute path to the image file.
   */
  setWallpaper(imagePath: string): Promise<void>;

  /**
   * Returns the name of the current operating system.
   */
  getAppName(): 'linux' | 'windows' | 'darwin';
}
EOF
else
    echo "➡ Skipping file (already exists): $FILE_PATH"
fi

# linux.ts
FILE_PATH="src/core/platform/linux.ts"
if [ ! -f "$FILE_PATH" ]; then
    echo "➡ Creating file: $FILE_PATH"
    cat << 'EOF' > "$FILE_PATH"
// Hyprland/Linux implementation of the IPlatformService.
// It uses Bun's native, high-performance 'spawn' API to call command-line tools.

import type { IPlatformService } from './types';

export const LinuxWallpaperService: IPlatformService = {
  getAppName: () => 'linux',

  async setWallpaper(imagePath: string): Promise<void> {
    console.log(`Setting wallpaper for Hyprland: ${imagePath}`);

    // Unload all existing wallpapers to prevent memory leaks in hyprpaper
    const unloadProc = Bun.spawn(['hyprctl', 'hyprpaper', 'unload', 'all']);
    await unloadProc.exited;

    // Preload the new wallpaper for faster switching
    const preloadProc = Bun.spawn(['hyprctl', 'hyprpaper', 'preload', imagePath]);
    if ((await preloadProc.exited) !== 0) {
      const errText = await new Response(preloadProc.stderr).text();
      throw new Error(`Failed to preload wallpaper: ${errText}`);
    }

    // Set the wallpaper on all monitors (the initial comma is important for hyprpaper)
    const wallpaperProc = Bun.spawn(['hyprctl', 'hyprpaper', 'wallpaper', `,${imagePath}`]);
    if ((await wallpaperProc.exited) !== 0) {
        const errText = await new Response(wallpaperProc.stderr).text();
        throw new Error(`Failed to set wallpaper: ${errText}`);
    }
  },
};
EOF
else
    echo "➡ Skipping file (already exists): $FILE_PATH"
fi

# windows.ts
FILE_PATH="src/core/platform/windows.ts"
if [ ! -f "$FILE_PATH" ]; then
    echo "➡ Creating file: $FILE_PATH"
    cat << 'EOF' > "$FILE_PATH"
// Windows 11 implementation of the IPlatformService.
// This serves as a placeholder for future development.

import type { IPlatformService } from './types';

export const WindowsWallpaperService: IPlatformService = {
  getAppName: () => 'windows',

  async setWallpaper(imagePath: string): Promise<void> {
    console.log(`Setting wallpaper for Windows: ${imagePath}`);
    // TODO: Implement the wallpaper setting logic for Windows.
    // This could be done by spawning a PowerShell script:
    // Bun.spawn(['powershell', '-File', './set-wallpaper.ps1', imagePath]);
    // Or by using a native Bun/Node.js addon that calls the Win32 API directly
    // for better performance and reliability.
    console.warn('Windows wallpaper setting is not yet implemented.');
    return Promise.resolve();
  },
};
EOF
else
    echo "➡ Skipping file (already exists): $FILE_PATH"
fi

# index.ts (The Platform Selector)
FILE_PATH="src/core/platform/index.ts"
if [ ! -f "$FILE_PATH" ]; then
    echo "➡ Creating file: $FILE_PATH"
    cat << 'EOF' > "$FILE_PATH"
// This module dynamically detects the operating system and exports the
// appropriate platform service. This is the only place in our app's core
// that has OS-specific conditional logic.

import { LinuxWallpaperService } from './linux';
import { WindowsWallpaperService } from './windows';
import type { IPlatformService } from './types';

let platformService: IPlatformService;

// Bun.env.OS provides the OS name directly from the Bun runtime.
switch (Bun.env.OS) {
  case 'linux':
    platformService = LinuxWallpaperService;
    break;
  case 'win32': // Bun uses 'win32' for Windows
    platformService = WindowsWallpaperService;
    break;
  // case 'darwin': // For macOS support in the future
  //   platformService = MacWallpaperService;
  //   break;
  default:
    throw new Error(`Unsupported platform: ${Bun.env.OS}`);
}

export default platformService;
EOF
else
    echo "➡ Skipping file (already exists): $FILE_PATH"
fi

# --- Create Core Logic Files (Placeholders) ---

# api.ts
FILE_PATH="src/core/api.ts"
if [ ! -f "$FILE_PATH" ]; then
    echo "➡ Creating file: $FILE_PATH"
    cat << 'EOF' > "$FILE_PATH"
// This module is responsible for all external API communications,
// specifically fetching image data from the NASA API.

export class NasaApi {
  private readonly apiKey: string;
  private readonly baseUrl = 'https://images-api.nasa.gov';

  constructor(apiKey: string) {
    if (!apiKey || apiKey === 'YOUR_API_KEY_HERE') {
      throw new Error('NASA API key is required.');
    }
    this.apiKey = apiKey;
  }

  async searchImages(query: string, limit: number) {
    // TODO: Implement the search logic, pagination, and error handling.
    console.log(`Searching for ${limit} images with query: ${query}`);
    return []; // Return a structured array of image data.
  }
}
EOF
else
    echo "➡ Skipping file (already exists): $FILE_PATH"
fi

# state.ts
FILE_PATH="src/core/state.ts"
if [ ! -f "$FILE_PATH" ]; then
    echo "➡ Creating file: $FILE_PATH"
    cat << 'EOF' > "$FILE_PATH"
// This module manages the application's state, including the list of
// all known images and their curation status (pending, approved, rejected).
// It will be responsible for loading and saving state to a JSON file.

import { join } from 'node:path';
// We would get the app data dir from Tauri's path API.

interface ImageState {
  nasaId: string;
  title: string;
  status: 'pending' | 'approved' | 'rejected';
  localPath: string;
  downloadedAt: string;
}

export class AppState {
  private state: {
    images: Record<string, ImageState>;
  };
  private stateFilePath: string;

  constructor(appDataDir: string) {
    this.stateFilePath = join(appDataDir, 'state.json');
    this.state = { images: {} };
  }

  async load() {
    // TODO: Implement loading state from this.stateFilePath using Bun.file API.
    console.log('Loading application state...');
  }

  async save() {
    // TODO: Implement saving state to this.stateFilePath using Bun.write.
    console.log('Saving application state...');
  }
}
EOF
else
    echo "➡ Skipping file (already exists): $FILE_PATH"
fi

echo ""
echo "✅ All files and folders have been created successfully!"
echo ""
echo "Next steps:"
echo "1. If you haven't already, run 'bun install' to get dependencies."
echo "2. Run 'bun tauri dev' to start the development server."
