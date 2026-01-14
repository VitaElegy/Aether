<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
const isHovered = ref(false);

const goHome = () => {
    router.push('/');
};

const goBack = () => {
    if (window.history.state && window.history.state.back) {
        router.back();
    } else {
        // Smart fallback: if no history, go up one level or home
        const current = router.currentRoute.value.path;
        const segments = current.split('/').filter(p => p);
        if (segments.length > 0) {
            // Remove last segment
            const up = '/' + segments.slice(0, -1).join('/');
            router.push(up || '/');
        } else {
            router.push('/');
        }
    }
};
</script>

<template>
    <div 
        class="flex items-center relative z-50 h-10"
        @mouseenter="isHovered = true"
        @mouseleave="isHovered = false"
    >
        <div 
            class="flex items-center bg-white/70 dark:bg-[#0d1117]/70 backdrop-blur-md border border-gray-200 dark:border-white/10 rounded-full shadow-lg transition-all duration-300 overflow-hidden"
            :class="isHovered ? 'pl-2 pr-1' : 'px-0'"
        >
            <!-- Back Button (Slides in) -->
            <div 
                class="overflow-hidden transition-all duration-300 flex items-center"
                :class="isHovered ? 'w-8 opacity-100 mr-2' : 'w-0 opacity-0'"
            >
                <button 
                    @click.stop="goBack"
                    class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-black/5 dark:hover:bg-white/10 text-gray-600 dark:text-gray-300 transition-colors"
                >
                    <i class="ri-arrow-left-line text-lg"></i>
                </button>
            </div>

            <!-- Vertical Divider -->
            <div 
                class="w-[1px] h-4 bg-gray-300 dark:bg-white/20 transition-opacity duration-300"
                :class="isHovered ? 'opacity-100 mr-2' : 'opacity-0 w-0'"
            ></div>

            <!-- Home Logo -->
            <button 
                @click="goHome"
                class="w-10 h-10 flex items-center justify-center rounded-full text-gray-900 dark:text-white hover:text-cyan-600 dark:hover:text-cyan-400 transition-colors"
            >
                <!-- Aether Triangle Logo (Simple SVG) -->
                <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                    <path d="M12 2L2 22H22L12 2ZM12 6.5L17.5 18H6.5L12 6.5Z" />
                </svg>
            </button>
        </div>
    </div>
</template>

<style scoped>
/* Optional refined styling */
</style>
