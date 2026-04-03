// PLAT-03: Header Action Protocol — Composable
import { ref, computed, readonly } from 'vue';
import type { HeaderAction, HeaderActionGroup } from '@/types/header-actions';

const actionGroups = ref<Map<string, HeaderActionGroup>>(new Map());

export function useHeaderActions() {
  const registerActions = (moduleId: string, actions: HeaderAction[], priority: number = 0) => {
    actionGroups.value.set(moduleId, { moduleId, actions, priority });
  };

  const unregisterActions = (moduleId: string) => {
    actionGroups.value.delete(moduleId);
  };

  const getActions = (moduleId: string): HeaderAction[] => {
    return actionGroups.value.get(moduleId)?.actions ?? [];
  };

  const allActions = computed(() => {
    const groups = Array.from(actionGroups.value.values());
    groups.sort((a, b) => b.priority - a.priority);
    return groups;
  });

  const activeModuleActions = (moduleId: string) => {
    return computed(() => getActions(moduleId));
  };

  const updateBadge = (moduleId: string, actionId: string, badge: HeaderAction['badge']) => {
    const group = actionGroups.value.get(moduleId);
    if (group) {
      const action = group.actions.find(a => a.id === actionId);
      if (action) {
        action.badge = badge;
      }
    }
  };

  const clearAllActions = () => {
    actionGroups.value.clear();
  };

  // For testing
  const _resetForTesting = () => {
    actionGroups.value.clear();
  };

  return {
    registerActions,
    unregisterActions,
    getActions,
    allActions: readonly(allActions),
    activeModuleActions,
    updateBadge,
    clearAllActions,
    _resetForTesting,
  };
}
