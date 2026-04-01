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

export interface ImportSection {
  name: string;
  count: number;
  action: string;
}

export interface ImportSummary {
  total_items: number;
  sections: ImportSection[];
  conflicts: string[];
}

export interface ProgressEvent {
  task_id: string;
  stage: string;
  percent: number;
  message: string;
  error?: string;
}

/** ENG-07: Conflict item for import preview */
export interface ImportConflict {
  word: string;
  existing_id: string;
  incoming_definition: string;
  existing_definition: string;
  resolution: 'skip' | 'overwrite' | 'merge';
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

  // ENG-07: Import endpoints

  /** Upload a file for import analysis (conflict preview) */
  analyzeImport: async (kbId: string, file: File): Promise<ImportSummary> => {
    const formData = new FormData();
    formData.append('file', file);
    const response = await axios.post(`${API_URL}/${kbId}/import/analyze`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return response.data;
  },

  /** Start the actual import process */
  startImport: async (kbId: string, taskId: string, options?: { mergeStrategy?: string }): Promise<string> => {
    const response = await axios.post(`${API_URL}/${kbId}/import/start`, {
      task_id: taskId,
      merge_strategy: options?.mergeStrategy || 'merge_by_lemma',
    });
    return response.data.task_id;
  },

  /** Import from Anki-like CSV format */
  importAnkiCsv: async (kbId: string, file: File): Promise<ImportSummary> => {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('format', 'anki_csv');
    const response = await axios.post(`${API_URL}/${kbId}/import/anki`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return response.data;
  },

  // SSE Progress
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

  getDownloadUrl: (taskId: string) => `${API_URL}/tasks/${taskId}/download`
};
