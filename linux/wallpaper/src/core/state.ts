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
