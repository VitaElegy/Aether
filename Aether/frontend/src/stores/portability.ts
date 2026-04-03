// PLAT-04: Portability Runtime — Store
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { portabilityApi, type ExportSummary, type ImportSummary, type TaskProgress } from '@/api/portability';

export interface PortabilityTask {
  id: string;
  type: 'export' | 'import';
  kbId: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;
  message: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
}

export const usePortabilityStore = defineStore('portability', () => {
  const tasks = ref<PortabilityTask[]>([]);
  const currentExportSummary = ref<ExportSummary | null>(null);
  const currentImportSummary = ref<ImportSummary | null>(null);
  const isAnalyzing = ref(false);
  const eventSources = new Map<string, EventSource>();

  const activeTasks = computed(() => tasks.value.filter(t => t.status === 'running'));
  const completedTasks = computed(() => tasks.value.filter(t => t.status === 'completed'));

  const analyzeExport = async (kbId: string) => {
    isAnalyzing.value = true;
    try {
      currentExportSummary.value = await portabilityApi.analyzeExport(kbId);
      return currentExportSummary.value;
    } finally {
      isAnalyzing.value = false;
    }
  };

  const startExport = async (kbId: string) => {
    const result = await portabilityApi.startExport(kbId);
    const task: PortabilityTask = {
      id: result.task_id,
      type: 'export',
      kbId,
      status: 'running',
      progress: 0,
      message: 'Starting export...',
      startedAt: new Date().toISOString(),
    };
    tasks.value.push(task);
    // Start listening to SSE progress
    listenToProgress(result.task_id);
    return task;
  };

  const listenToProgress = (taskId: string) => {
    // Close existing EventSource for this task if any
    const existing = eventSources.get(taskId);
    if (existing) {
      existing.close();
    }

    const url = portabilityApi.getProgressUrl(taskId);
    const token = localStorage.getItem('token');
    const eventSource = new EventSource(`${url}?token=${token}`);
    eventSources.set(taskId, eventSource);

    eventSource.onmessage = (event) => {
      try {
        const data: TaskProgress = JSON.parse(event.data);
        const task = tasks.value.find(t => t.id === taskId);
        if (task) {
          task.progress = data.percent;
          task.message = data.message;
          if (data.stage === 'Completed') {
            task.status = 'completed';
            task.completedAt = new Date().toISOString();
            eventSource.close();
            eventSources.delete(taskId);
          } else if (data.stage === 'Error') {
            task.status = 'failed';
            task.error = data.error;
            eventSource.close();
            eventSources.delete(taskId);
          }
        }
      } catch (e) {
        console.error('Failed to parse progress event', e);
      }
    };

    eventSource.onerror = () => {
      const task = tasks.value.find(t => t.id === taskId);
      if (task && task.status === 'running') {
        task.status = 'failed';
        task.error = 'Connection lost';
      }
      eventSource.close();
      eventSources.delete(taskId);
    };
  };

  const clearTasks = () => {
    // Close all EventSources for non-running tasks
    for (const [taskId, es] of eventSources) {
      const task = tasks.value.find(t => t.id === taskId);
      if (!task || task.status !== 'running') {
        es.close();
        eventSources.delete(taskId);
      }
    }
    tasks.value = tasks.value.filter(t => t.status === 'running');
  };

  const cleanup = () => {
    // Close all EventSources
    for (const es of eventSources.values()) {
      es.close();
    }
    eventSources.clear();
  };

  return {
    tasks,
    activeTasks,
    completedTasks,
    currentExportSummary,
    currentImportSummary,
    isAnalyzing,
    analyzeExport,
    startExport,
    clearTasks,
    cleanup,
  };
});
