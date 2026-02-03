<script setup lang="ts">
/**
 * LoadingState.vue - Skeleton UI for KB Loading
 * 
 * Layer 3 of Error Defense: Shows skeleton during lazy load.
 */
defineProps<{
    kbTitle?: string;
    timeout?: number;
}>();
</script>

<template>
    <div class="flex-1 flex flex-col items-center justify-center bg-paper text-ink/60 p-8">
        <!-- Skeleton Animation -->
        <div class="w-full max-w-2xl space-y-6">
            <!-- Header Skeleton -->
            <div class="flex items-center gap-4">
                <div class="w-12 h-12 rounded-lg bg-ink/5 animate-pulse"></div>
                <div class="flex-1 space-y-2">
                    <div class="h-6 w-48 bg-ink/5 rounded animate-pulse"></div>
                    <div class="h-4 w-32 bg-ink/5 rounded animate-pulse"></div>
                </div>
            </div>

            <!-- Content Skeleton -->
            <div class="space-y-4">
                <div class="h-4 w-full bg-ink/5 rounded animate-pulse"></div>
                <div class="h-4 w-5/6 bg-ink/5 rounded animate-pulse"></div>
                <div class="h-4 w-4/6 bg-ink/5 rounded animate-pulse"></div>
            </div>

            <!-- Cards Skeleton -->
            <div class="grid grid-cols-3 gap-4">
                <div class="h-24 bg-ink/5 rounded-lg animate-pulse"></div>
                <div class="h-24 bg-ink/5 rounded-lg animate-pulse delay-75"></div>
                <div class="h-24 bg-ink/5 rounded-lg animate-pulse delay-150"></div>
            </div>
        </div>

        <!-- Loading Text -->
        <div class="mt-8 flex items-center gap-2 text-sm">
            <i class="ri-loader-4-line animate-spin"></i>
            <span>{{ kbTitle ? `Loading ${kbTitle}...` : 'Loading...' }}</span>
        </div>
    </div>
</template>

<style scoped>
.delay-75 {
    animation-delay: 75ms;
}
.delay-150 {
    animation-delay: 150ms;
}
</style>
