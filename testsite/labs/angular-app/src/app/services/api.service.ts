import { Injectable } from '@angular/core';
import { init as apiInit, VERSION as API_VERSION } from '@xq9zk7823/api-client';

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  private client: any;

  constructor() {
    this.client = apiInit({
      baseUrl: 'https://api.acmecorp.internal',
      timeout: 5000
    });
    console.log('API Client v' + API_VERSION + ' initialized');
  }

  async getStatus(): Promise<string> {
    return this.client.ready ? 'connected' : 'disconnected';
  }

  async fetchData(endpoint: string): Promise<any> {
    console.log('Fetching from:', endpoint);
    return { data: 'mock response from @xq9zk7823/api-client' };
  }
}
