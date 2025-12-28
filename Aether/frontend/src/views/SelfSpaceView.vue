<script setup lang="ts">
import { ref, shallowRef } from 'vue';
import ModuleSwitcher from '../components/self-space/ModuleSwitcher.vue';
import ArticlesModule from '../components/self-space/modules/ArticlesModule.vue';
import KnowledgeModule from '../components/self-space/modules/KnowledgeModule.vue';
import MemosModule from '../components/self-space/modules/MemosModule.vue';

const currentModuleId = ref('articles');

const moduleMap: Record<string, any> = {
    articles: ArticlesModule,
    knowledge: KnowledgeModule,
    memos: MemosModule,
};

const CurrentComponent = shallowRef(ArticlesModule);

const switchModule = (id: string) => {
    currentModuleId.value = id;
    CurrentComponent.value = moduleMap[id];
};
</script>

<template>
    <div class="min-h-screen bg-paper text-ink selection:bg-accent/20 flex flex-col relative overflow-hidden">
        <!-- Ambient Background Elements -->
        <div class="absolute top-0 left-0 w-full h-96 bg-gradient-to-b from-ash/10 to-transparent pointer-events-none">
        </div>

        <!-- Main Content Area -->
        <main class="flex-1 relative z-10 w-full h-full">
            <Transition mode="out-in" enter-active-class="transition duration-300 ease-out"
                enter-from-class="opacity-0 translate-y-4" enter-to-class="opacity-100 translate-y-0"
                leave-active-class="transition duration-200 ease-in" leave-from-class="opacity-100 translate-y-0"
                leave-to-class="opacity-0 -translate-y-4">
                <component :is="CurrentComponent" />
            </Transition>
        </main>

        <!-- Dock Navigation -->
        <ModuleSwitcher :active-module="currentModuleId" @switch="switchModule" />
    </div>
</template>
