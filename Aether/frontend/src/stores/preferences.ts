import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const usePreferencesStore = defineStore('preferences', () => {
    // State
    const theme = ref<'light' | 'dark'>(localStorage.getItem('aether_theme') as 'light' | 'dark' || 'light');
    const isSidebarCollapsed = ref<boolean>(localStorage.getItem('aether_sidebar_collapsed') === 'true');
    const defaultCommitMessage = ref<string>(localStorage.getItem('aether_default_commit_msg') || 'Update content');

    // Actions
    const toggleTheme = () => {
        theme.value = theme.value === 'light' ? 'dark' : 'light';
    };

    const toggleSidebar = () => {
        isSidebarCollapsed.value = !isSidebarCollapsed.value;
    };

    // Persistence & Effects
    watch(theme, (newVal) => {
        localStorage.setItem('aether_theme', newVal);
        if (newVal === 'dark') {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    }, { immediate: true });

    watch(isSidebarCollapsed, (newVal) => {
        localStorage.setItem('aether_sidebar_collapsed', newVal.toString());
    });

    watch(defaultCommitMessage, (newVal) => {
        localStorage.setItem('aether_default_commit_msg', newVal);
    });

    const readViewShowTree = ref<boolean>(localStorage.getItem('aether_read_show_tree') !== 'false');
    const readViewShowOutline = ref<boolean>(localStorage.getItem('aether_read_show_outline') !== 'false');
    const readViewTreeSide = ref<'left' | 'right'>((localStorage.getItem('aether_read_tree_side') as 'left' | 'right') || 'left');

    // Persistence
    watch(readViewShowTree, (v) => localStorage.setItem('aether_read_show_tree', v.toString()));
    watch(readViewShowOutline, (v) => localStorage.setItem('aether_read_show_outline', v.toString()));
    watch(readViewTreeSide, (v) => localStorage.setItem('aether_read_tree_side', v));

    return {
        theme,
        isSidebarCollapsed,
        defaultCommitMessage,
        readViewShowTree,
        readViewShowOutline,
        readViewTreeSide,
        toggleTheme,
        toggleSidebar
    };
});
