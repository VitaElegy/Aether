// PLAT-04: Portability Runtime — Frontend API Client
import axios from 'axios';

const getAuthHeaders = () => ({
  headers: {
    'Authorization': `Bearer ${localStorage.getItem('aether_token')}`
  }
});

export interface ExportSummary {
  total_items: number;
  estimated_size: string;
  sections: { name: string; count: number; details: string }[];
}

export interface ImportSummary {
  total_items: number;
  sections: { name: string; count: number; action: string }[];
  conflicts: string[];
}

// Backend returns ImportPreview which wraps ImportSummary
export interface ImportConflict {
  item_id: string;
  item_name: string;
  conflict_type: string;
  existing_value?: string;
  incoming_value?: string;
}

export interface SuggestedAction {
  item_id: string;
  action: string;
  reason: string;
}

export interface ImportPreview {
  summary: ImportSummary;
  conflicts: ImportConflict[];
  suggested_actions: SuggestedAction[];
}

export interface TaskProgress {
  task_id: string;
  stage: string;
  percent: number;
  message: string;
  error?: string;
}

// Alias for backward compatibility
export type ProgressEvent = TaskProgress;

export const portabilityApi = {
  // Export
  analyzeExport: async (kbId: string): Promise<ExportSummary> => {
    const response = await axios.get(`/api/portability/${kbId}/export/preview`, getAuthHeaders());
    return response.data;
  },

  startExport: async (kbId: string): Promise<{ task_id: string }> => {
    const response = await axios.post(`/api/portability/${kbId}/export/start`, {}, getAuthHeaders());
    return response.data;
  },

  // Import
  analyzeImport: async (kbId: string, file: File): Promise<ImportPreview> => {
    const formData = new FormData();
    formData.append('file', file);
    const response = await axios.post(`/api/portability/${kbId}/import/analyze`, formData, {
      headers: { ...getAuthHeaders().headers, 'Content-Type': 'multipart/form-data' }
    });
    return response.data;
  },

  startImport: async (kbId: string, file: File): Promise<{ task_id: string }> => {
    const formData = new FormData();
    formData.append('file', file);
    const response = await axios.post(`/api/portability/${kbId}/import/start`, formData, {
      headers: { ...getAuthHeaders().headers, 'Content-Type': 'multipart/form-data' }
    });
    return response.data;
  },

  // Progress (SSE)
  getProgressUrl: (taskId: string): string => {
    return `/api/portability/tasks/${taskId}/progress`;
  },

  // Download
  getDownloadUrl: (taskId: string): string => {
    return `/api/portability/tasks/${taskId}/download`;
  },

  downloadExport: async (taskId: string): Promise<Blob> => {
    const response = await axios.get(`/api/portability/tasks/${taskId}/download`, {
      ...getAuthHeaders(),
      responseType: 'blob'
    });
    return response.data;
  },

  // SSE Progress connection
  connectProgress: (taskId: string, onEvent: (event: TaskProgress) => void, onError?: (err: Event) => void): EventSource => {
    const url = `/api/portability/tasks/${taskId}/progress`;
    const es = new EventSource(url);
    es.onmessage = (event: MessageEvent<string>) => {
      const data = JSON.parse(event.data) as TaskProgress;
      onEvent(data);
      if (data.stage === 'Completed' || data.error) {
        es.close();
      }
    };
    es.onerror = (err: Event) => {
      onError?.(err);
      es.close();
    };
    return es;
  },
};
