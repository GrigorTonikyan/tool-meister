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
    console.log('Loading application state from:', this.stateFilePath);
    try {
      const file = Bun.file(this.stateFilePath);
      if (await file.exists()) {
        const content = await file.text();
        this.state = JSON.parse(content);
        console.log(`Loaded ${Object.keys(this.state.images).length} images from state.`);
      } else {
        console.log('State file does not exist, starting with empty state.');
      }
    } catch (error) {
      console.error('Failed to load state:', error);
    }
  }

  async save() {
    console.log('Saving application state to:', this.stateFilePath);
    try {
      const content = JSON.stringify(this.state, null, 2);
      await Bun.write(this.stateFilePath, content);
      console.log('State saved successfully.');
    } catch (error) {
      console.error('Failed to save state:', error);
    }
  }

  addImage(image: ImageState) {
    this.state.images[image.nasaId] = image;
  }

  getImage(nasaId: string): ImageState | undefined {
    return this.state.images[nasaId];
  }

  getAllImages(): ImageState[] {
    return Object.values(this.state.images);
  }

  updateStatus(nasaId: string, status: 'pending' | 'approved' | 'rejected') {
    if (this.state.images[nasaId]) {
      this.state.images[nasaId].status = status;
    }
  }
}
