<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import TopNavBar from '../components/TopNavBar.vue';
import ModuleSwitcher from '../components/self-space/ModuleSwitcher.vue';
import { usePluginStore } from '../stores/plugins';
import { useVocabularyStore } from '../stores/vocabulary';
import { useNavigationStore } from '../stores/navigation';

const router = useRouter();
const pluginStore = usePluginStore();
const vocabStore = useVocabularyStore();
const navStore = useNavigationStore();

// Default to the first registered plugin or fallback to 'articles'
const currentModuleId = ref('articles');

const currentPlugin = computed(() => pluginStore.getPlugin(currentModuleId.value));

const CurrentComponent = computed(() => currentPlugin.value?.component);
const currentModuleLabel = computed(() => currentPlugin.value?.label || 'Self Space');

const switchModule = (id: string) => {
    currentModuleId.value = id;
};

// Ensure we have a valid selection on mount if plugins are ready
if (pluginStore.plugins.length > 0 && !pluginStore.getPlugin(currentModuleId.value)) {
    currentModuleId.value = pluginStore.plugins[0].id;
}
// Navigation
const goBack = () => {
    if (window.history.state && window.history.state.back) {
        router.back();
    } else {
        router.push('/');
    }
};
</script>

<template>
    <div class="h-screen bg-paper text-ink selection:bg-accent/20 flex flex-col relative overflow-hidden">
        <!-- Ambient Background Elements -->
        <div class="absolute top-0 left-0 w-full h-96 bg-gradient-to-b from-ash/10 to-transparent pointer-events-none">
        </div>

        <!-- Top Navigation Bar -->
        <TopNavBar>
            <template #left>
                <!-- handled by global beacon -->
            </template>

            <template #center>
                <!-- DYNAMIC CENTER PORTAL -->
                <div id="nav-center-portal" class="contents"></div>

                <!-- Default Center Content -->
                <div v-show="!navStore.hasCustomCenter" class="flex items-center gap-4">
                    <span class="text-[10px] font-black uppercase tracking-[0.3em] text-ink/40">
                        Self Space / {{ currentModuleLabel }}
                    </span>
                </div>
            </template>

            <template #right>
                <!-- DYNAMIC RIGHT PORTAL -->
                <div id="nav-right-portal" class="contents"></div>

                <!-- Default Right Content -->
                <div v-show="!navStore.hasCustomRight" class="flex items-center gap-2">
                     <!-- Default Global Actions (e.g. Settings, Profile) -->
                     <button 
                        @click="router.push('/settings')"
                        class="text-ink/30 hover:text-ink transition-colors px-2 py-1"
                        title="Settings"
                    >
                        <i class="ri-settings-4-line text-lg"></i>
                    </button>
                </div>
            </template>
        </TopNavBar>

        <!-- Main Content Area -->
        <main class="flex-1 relative z-10 w-full h-full flex flex-col pt-16">
            <Transition mode="out-in" enter-active-class="transition duration-300 ease-out"
                enter-from-class="opacity-0 translate-y-4" enter-to-class="opacity-100 translate-y-0"
                leave-active-class="transition duration-200 ease-in" leave-from-class="opacity-100 translate-y-0"
                leave-to-class="opacity-0 -translate-y-4">
                <component :is="CurrentComponent" class="flex-1 h-full" :headless="true" />
            </Transition>
        </main>

        <!-- Dock Navigation -->
        <ModuleSwitcher :active-module="currentModuleId" :modules="pluginStore.plugins" @switch="switchModule" />
    </div>
</template>
