/* Built with Webpack 5 - depfused test lab */
import axios from 'axios';
import { init as initApiClient } from '@xq9zk7823/api-client';

const client = initApiClient({ baseUrl: 'https://api.example.com' });

const axiosInstance = axios.create({
  baseURL: 'https://api.example.com',
  timeout: 5000,
  headers: {
    'Content-Type': 'application/json',
    'X-Client-Ready': String(client.ready),
  },
});

export const apiService = {
  async fetchData(endpoint) {
    try {
      const response = await axiosInstance.get(endpoint);
      return response.data;
    } catch (error) {
      console.error('API Error:', error.message);
      return { error: error.message };
    }
  },

  async postData(endpoint, data) {
    try {
      const response = await axiosInstance.post(endpoint, data);
      return response.data;
    } catch (error) {
      console.error('API Error:', error.message);
      return { error: error.message };
    }
  },
};
