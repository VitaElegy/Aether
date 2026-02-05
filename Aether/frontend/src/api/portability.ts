import axios from 'axios';

const API_URL = '/api/portability';

export interface ExportSection {
  name: string;
  count: number;
  details: string;
}

export interface ExportSummary {
  total_items: number;
  estimated_size: string;
  sections: ExportSection[];
}

export interface ProgressEvent {
  task_id: string;
  stage: string;
  percent: number;
  message: string;
  error?: string;
}

export const portabilityApi = {
  analyzeExport: async (kbId: string): Promise<ExportSummary> => {
    const response = await axios.get(`${API_URL}/${kbId}/export/preview`);
    return response.data;
  },

  startExport: async (kbId: string): Promise<string> => {
    const response = await axios.post(`${API_URL}/${kbId}/export/start`);
    return response.data.task_id;
  },

  // Helper to connect to SSE
  connectProgress: (taskId: string, onEvent: (event: ProgressEvent) => void, onError: (err: any) => void) => {
    const eventSource = new EventSource(`${API_URL}/tasks/${taskId}/progress`);
    
    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onEvent(data);
        if (data.stage === 'Completed' || data.error) {
          eventSource.close();
        }
      } catch (e) {
        console.error("Failed to parse SSE event", e);
      }
    };

    eventSource.onerror = (err) => {
      onError(err);
      eventSource.close();
    };

    return eventSource;
  },
  
  // Helper for download URL
  getDownloadUrl: (taskId: string) => `${API_URL}/tasks/${taskId}/download`
};
