// This module detects the operating system and exports the
// appropriate platform service. This is the only place in our app's core
// that has OS-specific conditional logic.
import { LinuxWallpaperService } from "./linux";
import { WindowsWallpaperService } from "./windows";

import type { IPlatformService } from "./types";

let platformService: IPlatformService;

// Bun.env.OS gives us the OS name ('linux', 'windows', 'darwin')
// Bun.env.OS provides the OS name directly from the Bun runtime.
switch (Bun.env.OS) {
  case "linux":
    platformService = LinuxWallpaperService;
    break;
  case "win32": // Bun uses 'win32' for Windows
    platformService = WindowsWallpaperService;
    break;
  // case 'darwin': // For macOS support in the future
  //   platformService = MacWallpaperService;
  //   break;
  default:
    throw new Error(`Unsupported platform: ${Bun.env.OS}`);
}

export default platformService;
