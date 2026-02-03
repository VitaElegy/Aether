/**
 * Lightweight Event Bus for Self Space extensions.
 * Used for loose coupling between Orchestrator and plugins.
 */

type EventHandler<T = unknown> = (payload: T) => void;

interface EventBus {
    on<T>(event: string, handler: EventHandler<T>): void;
    off<T>(event: string, handler: EventHandler<T>): void;
    emit<T>(event: string, payload: T): void;
}

class EventBusImpl implements EventBus {
    private handlers: Map<string, Set<EventHandler>> = new Map();

    on<T>(event: string, handler: EventHandler<T>): void {
        if (!this.handlers.has(event)) {
            this.handlers.set(event, new Set());
        }
        this.handlers.get(event)!.add(handler as EventHandler);
    }

    off<T>(event: string, handler: EventHandler<T>): void {
        this.handlers.get(event)?.delete(handler as EventHandler);
    }

    emit<T>(event: string, payload: T): void {
        this.handlers.get(event)?.forEach((handler) => {
            try {
                handler(payload);
            } catch (e) {
                console.error(`[EventBus] Handler error for ${event}:`, e);
            }
        });
    }
}

// Singleton
export const eventBus: EventBus = new EventBusImpl();

// Type-safe event definitions
export interface SelfSpaceEvents {
    'kb:activated': { kbId: string; rendererId: string };
    'kb:deactivated': { kbId: string };
    'kb:error': { kbId: string; error: Error };
}
