import { defineStore } from 'pinia';
import { shallowRef } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';

export const usePluginStore = defineStore('plugins', () => {
    // using shallowRef avoids deep reactivity overhead for components within the plugin objects
    const plugins = shallowRef<SelfSpacePlugin[]>([]);

    const registerPlugin = (plugin: SelfSpacePlugin) => {
        console.log(`[PluginStore] Registering plugin: ${plugin.id}`, plugin);
        if (!plugins.value.some(p => p.id === plugin.id)) {
            // Re-assign to trigger reactivity and sort
            plugins.value = [...plugins.value, plugin].sort((a, b) => (a.dock.order ?? 99) - (b.dock.order ?? 99));
            console.log(`[PluginStore] Total plugins: ${plugins.value.length}`);
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
