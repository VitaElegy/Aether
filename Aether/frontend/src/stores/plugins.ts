import { defineStore } from 'pinia';
import { shallowRef } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';

export const usePluginStore = defineStore('plugins', () => {
    // using shallowRef avoids deep reactivity overhead for components within the plugin objects
    const plugins = shallowRef<SelfSpacePlugin[]>([]);
    const manifests = shallowRef<Record<string, import('../types/plugin_manifest').SkbPluginManifest>>({});

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

    const registerManifest = (manifest: import('../types/plugin_manifest').SkbPluginManifest) => {
        console.log(`[PluginStore] Registering manifest for: ${manifest.sys_id}`);
        manifests.value = { ...manifests.value, [manifest.sys_id]: manifest };
    };

    const getPlugin = (id: string) => plugins.value.find(p => p.id === id);
    const getManifest = (sysId: string) => manifests.value[sysId];

    return {
        plugins,
        manifests,
        registerPlugin,
        registerManifest,
        getPlugin,
        getManifest
    };
});
