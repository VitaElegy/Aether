import { defineStore } from 'pinia';
import { shallowRef } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';

export const usePluginStore = defineStore('plugins', () => {
    // using shallowRef avoids deep reactivity overhead for components within the plugin objects
    const plugins = shallowRef<SelfSpacePlugin[]>([]);

    const registerPlugin = (plugin: SelfSpacePlugin) => {
        if (!plugins.value.some(p => p.id === plugin.id)) {
            // Re-assign to trigger reactivity and sort
            plugins.value = [...plugins.value, plugin].sort((a, b) => (a.order ?? 99) - (b.order ?? 99));
        } else {
            console.warn(`Plugin with id ${plugin.id} is already registered.`);
        }
    };

    const getPlugin = (id: string) => plugins.value.find(p => p.id === id);

    return {
        plugins,
        registerPlugin,
        getPlugin
    };
});
