// This module is responsible for all external API communications,
// specifically fetching image data from the NASA API.

export class NasaApi {
  private readonly baseUrl = 'https://images-api.nasa.gov';

  constructor() {
    // NASA Images API doesn't actually require an API key for search, but
    // some other NASA APIs do (like APOD). We'll keep it simple for now.
  }

  async searchImages(query: string, limit: number) {
    const url = new URL(`${this.baseUrl}/search`);
    url.searchParams.set('q', query);
    url.searchParams.set('media_type', 'image');

    try {
      const response = await fetch(url.toString());
      if (!response.ok) {
        throw new Error(`NASA API error: ${response.statusText}`);
      }

      const data = await response.json();
      const items = data.collection.items || [];

      // Map to a more friendly structure and respect the limit
      return items.slice(0, limit).map((item: any) => {
        const itemData = item.data[0];
        const links = item.links[0];
        
        return {
          nasaId: itemData.nasa_id,
          title: itemData.title,
          description: itemData.description,
          imageUrl: links.href,
          dateCreated: itemData.date_created,
        };
      });
    } catch (error) {
      console.error('Error fetching from NASA API:', error);
      throw error;
    }
  }
}
