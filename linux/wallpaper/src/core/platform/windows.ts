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
