<template>
    <div class="h-full flex flex-col bg-paper relative overflow-hidden">
        <Transition name="fade" mode="out-in">
            <VulnerabilityKanban v-if="currentProject" />
            <ProjectList v-else />
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import ProjectList from './ProjectList.vue';
import VulnerabilityKanban from './VulnerabilityKanban.vue';
import { useVrkbStore } from '@/stores/vrkb';

const store = useVrkbStore();
const currentProject = computed(() => store.currentProject);

// Logic: If no project selected -> Show List.
// If project selected -> Show Kanban (Dashboard).
// The children component can handle "Back" logic by calling store.selectProject(null).
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
