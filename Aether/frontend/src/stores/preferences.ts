import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const usePreferencesStore = defineStore('preferences', () => {
    // State
    const theme = ref<'light' | 'dark'>(localStorage.getItem('aether_theme') as 'light' | 'dark' || 'light');
    const isSidebarCollapsed = ref<boolean>(localStorage.getItem('aether_sidebar_collapsed') === 'true');

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

    return {
        theme,
        isSidebarCollapsed,
        toggleTheme,
        toggleSidebar
    };
});
