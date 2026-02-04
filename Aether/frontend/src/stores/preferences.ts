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

    // -- Pinned Knowledge Bases --
    const pinnedKbIds = ref<string[]>([]);
    try {
        const stored = localStorage.getItem('aether_pinned_kbs');
        if (stored) {
            const parsed = JSON.parse(stored);
            if (Array.isArray(parsed)) pinnedKbIds.value = parsed;
        }
    } catch (e) {
        console.error('Failed to load pinned kbs', e);
    }

    watch(pinnedKbIds, (newVal) => {
        localStorage.setItem('aether_pinned_kbs', JSON.stringify(newVal));
    }, { deep: true });

    const pinKb = (id: string) => {
        if (!pinnedKbIds.value.includes(id)) pinnedKbIds.value.push(id);
    };

    const unpinKb = (id: string) => {
        pinnedKbIds.value = pinnedKbIds.value.filter(x => x !== id);
    };

    const isPinned = (id: string) => pinnedKbIds.value.includes(id);

    const togglePin = (id: string) => {
        if (isPinned(id)) unpinKb(id);
        else pinKb(id);
    };



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
        pinnedKbIds,
        toggleTheme,
        toggleSidebar,
        pinKb,
        unpinKb,
        isPinned,
        togglePin
    };
});
