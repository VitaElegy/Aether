/**
 * PLAT-06: useErrorBoundary Composable
 * 
 * Captures and manages module-level errors with structured reporting.
 * Provides error isolation per module and recovery mechanisms.
 * 
 * Usage:
 *   const { captureError, getModuleErrors, isModuleHealthy } = useErrorBoundary();
 *   
 *   try { ... } catch (err) {
 *     captureError('vrkb', err, { operation: 'save' });
 *   }
 */

import { ref, computed, type ComputedRef } from 'vue';

export type ErrorType =
  | 'crash'
  | 'network_error'
  | 'validation_error'
  | 'auth_error'
  | 'not_found'
  | 'rate_limited'
  | 'plugin_error'
  | 'unknown';

export interface ErrorBoundaryEntry {
  /** Unique error ID */
  errorId: string;
  /** Module that encountered the error */
  moduleId: string;
  /** Error classification */
  errorType: ErrorType;
  /** Human-readable message */
  message: string;
  /** Stack trace (if available) */
  stackTrace?: string;
  /** Additional context */
  context: Record<string, string>;
  /** When the error occurred */
  timestamp: number;
  /** Whether the error has been acknowledged/dismissed */
  acknowledged: boolean;
}

// ============================================================
// SINGLETON ERROR REGISTRY
// ============================================================
const MAX_ERRORS_PER_MODULE = 50;
const errorRegistry = ref<Map<string, ErrorBoundaryEntry[]>>(new Map());
let errorCounter = 0;

function generateErrorId(): string {
  return `err_${Date.now()}_${++errorCounter}`;
}

function classifyError(error: unknown): ErrorType {
  if (error instanceof TypeError) return 'validation_error';
  if (error instanceof Error) {
    const msg = error.message.toLowerCase();
    if (msg.includes('network') || msg.includes('fetch') || msg.includes('timeout')) return 'network_error';
    if (msg.includes('unauthorized') || msg.includes('forbidden') || msg.includes('401') || msg.includes('403')) return 'auth_error';
    if (msg.includes('not found') || msg.includes('404')) return 'not_found';
    if (msg.includes('rate limit') || msg.includes('429')) return 'rate_limited';
  }
  return 'unknown';
}

export function useErrorBoundary() {
  /**
   * Capture an error for a module.
   */
  function captureError(
    moduleId: string,
    error: unknown,
    context?: Record<string, string>,
    errorType?: ErrorType,
  ): ErrorBoundaryEntry {
    const entry: ErrorBoundaryEntry = {
      errorId: generateErrorId(),
      moduleId,
      errorType: errorType || classifyError(error),
      message: error instanceof Error ? error.message : String(error),
      stackTrace: error instanceof Error ? error.stack : undefined,
      context: context || {},
      timestamp: Date.now(),
      acknowledged: false,
    };

    // Store in registry
    const moduleErrors = errorRegistry.value.get(moduleId) || [];
    moduleErrors.push(entry);

    // Evict old errors
    if (moduleErrors.length > MAX_ERRORS_PER_MODULE) {
      moduleErrors.splice(0, moduleErrors.length - MAX_ERRORS_PER_MODULE);
    }

    errorRegistry.value.set(moduleId, [...moduleErrors]);

    // Log in development
    if (import.meta.env.DEV) {
      console.error(`[ErrorBoundary:${moduleId}] ${entry.errorType}: ${entry.message}`, context || '');
    }

    return entry;
  }

  /**
   * Get errors for a specific module (most recent first).
   */
  function getModuleErrors(moduleId: string): ErrorBoundaryEntry[] {
    return (errorRegistry.value.get(moduleId) || []).slice().reverse();
  }

  /**
   * Get unacknowledged errors for a module.
   */
  function getUnacknowledgedErrors(moduleId: string): ErrorBoundaryEntry[] {
    return getModuleErrors(moduleId).filter(e => !e.acknowledged);
  }

  /**
   * Check if a module is "healthy" (no recent unacknowledged errors).
   */
  function isModuleHealthy(moduleId: string): boolean {
    return getUnacknowledgedErrors(moduleId).length === 0;
  }

  /**
   * Acknowledge (dismiss) an error.
   */
  function acknowledgeError(errorId: string): void {
    for (const [, errors] of errorRegistry.value) {
      const error = errors.find(e => e.errorId === errorId);
      if (error) {
        error.acknowledged = true;
        break;
      }
    }
  }

  /**
   * Acknowledge all errors for a module.
   */
  function acknowledgeAllForModule(moduleId: string): void {
    const errors = errorRegistry.value.get(moduleId);
    if (errors) {
      errors.forEach(e => e.acknowledged = true);
      errorRegistry.value.set(moduleId, [...errors]);
    }
  }

  /**
   * Clear all errors for a module (e.g., after recovery/retry).
   */
  function clearModuleErrors(moduleId: string): void {
    errorRegistry.value.delete(moduleId);
  }

  /**
   * Get all errors across all modules.
   */
  const allErrors: ComputedRef<ErrorBoundaryEntry[]> = computed(() => {
    const all: ErrorBoundaryEntry[] = [];
    for (const errors of errorRegistry.value.values()) {
      all.push(...errors);
    }
    return all.sort((a, b) => b.timestamp - a.timestamp);
  });

  /**
   * Get error count across all modules.
   */
  const totalErrorCount = computed(() => {
    let count = 0;
    for (const errors of errorRegistry.value.values()) {
      count += errors.filter(e => !e.acknowledged).length;
    }
    return count;
  });

  /**
   * Get list of modules with unacknowledged errors.
   */
  const unhealthyModules: ComputedRef<string[]> = computed(() => {
    const modules: string[] = [];
    for (const [moduleId, errors] of errorRegistry.value) {
      if (errors.some(e => !e.acknowledged)) {
        modules.push(moduleId);
      }
    }
    return modules;
  });

  /**
   * Clear all errors globally.
   */
  function clearAll(): void {
    errorRegistry.value.clear();
  }

  return {
    // Core API
    captureError,
    getModuleErrors,
    getUnacknowledgedErrors,
    isModuleHealthy,

    // Acknowledgement
    acknowledgeError,
    acknowledgeAllForModule,

    // Cleanup
    clearModuleErrors,
    clearAll,

    // Computed
    allErrors,
    totalErrorCount,
    unhealthyModules,
  };
}
