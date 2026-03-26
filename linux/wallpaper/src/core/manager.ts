import { NasaApi } from './api';
import { AppState } from './state';
import platformService from './platform';

export class WallpaperManager {
  private api: NasaApi;
  private state: AppState;

  constructor(appDataDir: string) {
    this.api = new NasaApi();
    this.state = new AppState(appDataDir);
  }

  async initialize() {
    await this.state.load();
  }

  async fetchAndSetRandomWallpaper(query: string = 'nebula') {
    try {
      console.log(`Fetching new wallpaper with query: ${query}`);
      const images = await this.api.searchImages(query, 10);
      
      if (images.length === 0) {
        throw new Error('No images found for the query.');
      }

      // Pick a random image from the results
      const randomImage = images[Math.floor(Math.random() * images.length)];
      
      // For now, we'll set it directly (even though we might want to download it first)
      // In a real app, we'd probably download to a local path and then set it.
      // But the setWallpaper API expects a path.
      
      // Assume download logic is here...
      const localPath = randomImage.imageUrl; // Placeholder for actual download

      await platformService.setWallpaper(localPath);
      
      // Update state
      this.state.addImage({
        nasaId: randomImage.nasaId,
        title: randomImage.title,
        status: 'pending',
        localPath: localPath,
        downloadedAt: new Date().toISOString(),
      });
      
      await this.state.save();
      return randomImage.title;
    } catch (error) {
      console.error('Failed to set random wallpaper:', error);
      throw error;
    }
  }

  async approveCurrent(nasaId: string) {
    this.state.updateStatus(nasaId, 'approved');
    await this.state.save();
  }

  async rejectCurrent(nasaId: string) {
    this.state.updateStatus(nasaId, 'rejected');
    await this.state.save();
  }
}
