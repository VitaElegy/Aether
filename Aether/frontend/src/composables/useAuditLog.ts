/**
 * PLAT-06: useAuditLog Composable
 * 
 * Records frontend user actions for audit trail.
 * Provides a lightweight, in-memory audit log with optional backend sync.
 * 
 * Usage:
 *   const { logAction, recentEvents } = useAuditLog();
 *   logAction('article', articleId, 'view', { source: 'search' });
 */

import { ref, computed, type ComputedRef } from 'vue';

export interface AuditEntry {
  /** Unique event ID */
  eventId: string;
  /** Actor (defaults to 'user') */
  actor: string;
  /** Target entity type */
  targetType: string;
  /** Target entity ID */
  targetId: string;
  /** Action performed */
  action: string;
  /** Additional context */
  context: Record<string, string>;
  /** Result */
  result: 'success' | 'failure' | 'partial';
  /** Error message if result is failure */
  errorMessage?: string;
  /** Timestamp */
  timestamp: number;
}

// ============================================================
// SINGLETON LOG BUFFER
// ============================================================
const LOG_BUFFER_SIZE = 200;
const logBuffer = ref<AuditEntry[]>([]);
let eventCounter = 0;

function generateEventId(): string {
  return `fe_${Date.now()}_${++eventCounter}`;
}

export function useAuditLog() {
  /**
   * Log an action to the audit trail.
   */
  function logAction(
    targetType: string,
    targetId: string,
    action: string,
    context?: Record<string, string>,
    result?: 'success' | 'failure' | 'partial',
    errorMessage?: string,
  ): AuditEntry {
    const entry: AuditEntry = {
      eventId: generateEventId(),
      actor: 'user',
      targetType,
      targetId,
      action,
      context: context || {},
      result: result || 'success',
      errorMessage,
      timestamp: Date.now(),
    };

    logBuffer.value.push(entry);

    // Evict old entries if buffer overflows
    if (logBuffer.value.length > LOG_BUFFER_SIZE) {
      logBuffer.value = logBuffer.value.slice(-LOG_BUFFER_SIZE);
    }

    // Debug logging in development
    if (import.meta.env.DEV) {
      console.debug(`[Audit] ${action} ${targetType}:${targetId}`, context || '');
    }

    return entry;
  }

  /**
   * Log a navigation event.
   */
  function logNavigation(from: string, to: string, context?: Record<string, string>): void {
    logAction('navigation', to, 'navigate', {
      from,
      ...context,
    });
  }

  /**
   * Log an error event.
   */
  function logError(
    targetType: string,
    targetId: string,
    action: string,
    errorMessage: string,
    context?: Record<string, string>,
  ): AuditEntry {
    return logAction(targetType, targetId, action, context, 'failure', errorMessage);
  }

  /**
   * Get recent events (most recent first).
   */
  const recentEvents: ComputedRef<AuditEntry[]> = computed(() => {
    return [...logBuffer.value].reverse();
  });

  /**
   * Get events filtered by target type.
   */
  function getEventsByType(targetType: string): AuditEntry[] {
    return logBuffer.value.filter(e => e.targetType === targetType).reverse();
  }

  /**
   * Get events filtered by action.
   */
  function getEventsByAction(action: string): AuditEntry[] {
    return logBuffer.value.filter(e => e.action === action).reverse();
  }

  /**
   * Get the total event count.
   */
  const eventCount = computed(() => logBuffer.value.length);

  /**
   * Clear the log buffer.
   */
  function clearLog(): void {
    logBuffer.value = [];
  }

  /**
   * Export the current log as JSON (for debugging or backend sync).
   */
  function exportLog(): string {
    return JSON.stringify(logBuffer.value, null, 2);
  }

  return {
    logAction,
    logNavigation,
    logError,
    recentEvents,
    getEventsByType,
    getEventsByAction,
    eventCount,
    clearLog,
    exportLog,
  };
}
