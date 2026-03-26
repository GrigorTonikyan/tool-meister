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
