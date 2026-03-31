import { defineStore } from 'pinia';
import { shallowRef } from 'vue';
import type { SelfSpacePlugin } from '../core/plugin';
import {
    normalizeRendererId,
    resolvePluginIdForRenderer,
    resolveSpecialKbRenderer,
} from '../registries/special_kb_registry';

export const usePluginStore = defineStore('plugins', () => {
    // using shallowRef avoids deep reactivity overhead for components within the plugin objects
    const plugins = shallowRef<SelfSpacePlugin[]>([]);
    const manifests = shallowRef<Record<string, import('../types/plugin_manifest').SkbPluginManifest>>({});

    const registerPlugin = (plugin: SelfSpacePlugin) => {
        if (!plugins.value.some(p => p.id === plugin.id)) {
            // Re-assign to trigger reactivity and sort
            plugins.value = [...plugins.value, plugin].sort((a, b) => (a.dock.order ?? 99) - (b.dock.order ?? 99));
            // console.debug(`[PluginStore] Registered: ${plugin.id}`);
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

    // [NEW] Strict Resolution Logic
    const resolvePlugin = (rendererId: string | null | undefined): SelfSpacePlugin | undefined => {
        const normalizedRendererId = normalizeRendererId(rendererId);
        if (!normalizedRendererId) return undefined;

        const resolution = resolveSpecialKbRenderer(normalizedRendererId);
        const pluginId = resolvePluginIdForRenderer(normalizedRendererId) ?? normalizedRendererId;
        const found = plugins.value.find(p => p.id === pluginId);

        if (found) {
            if (resolution?.migrated) {
                console.info(
                    `[PluginStore] Migrated legacy renderer '${normalizedRendererId}' -> '${resolution.canonicalRendererId}' using plugin '${pluginId}'.`,
                );
            }
            return found;
        }

        // 2. Strict Error (Loud Failure)
        if (!found) {
            console.error(
                `[PluginStore] MISSING PLUGIN for renderer '${normalizedRendererId}' (resolved plugin '${pluginId}'). Check registry.`,
            );
            console.warn('[PluginStore] Available Plugins:', plugins.value.map(p => p.id));
        }
        return undefined;
    };

    return {
        plugins,
        manifests,
        registerPlugin,
        registerManifest,
        getPlugin,
        getManifest,
        resolvePlugin // Export
    };
});
