import { spawn } from "bun";
import type { IPlatformService } from "./types";

export const LinuxWallpaperService: IPlatformService = {
  getAppName: () => "linux",

  async setWallpaper(imagePath: string): Promise<void> {
    console.log(`Setting wallpaper for Hyprland: ${imagePath}`);

    // Using bun spawn for native, fast process execution
    const unloadProc = spawn(["hyprctl", "hyprpaper", "unload", "all"]);
    await unloadProc.exited;

    const preloadProc = spawn(["hyprctl", "hyprpaper", "preload", imagePath]);
    if ((await preloadProc.exited) !== 0) {
      throw new Error(`Failed to preload wallpaper: ${imagePath}`);
    }

    const wallpaperProc = spawn([
      "hyprctl",
      "hyprpaper",
      "wallpaper",
      `,${imagePath}`,
    ]);
    if ((await wallpaperProc.exited) !== 0) {
      throw new Error(`Failed to set wallpaper: ${imagePath}`);
    }
  },
};
