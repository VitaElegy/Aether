<template>
    <div class="h-full w-full flex flex-col items-center justify-center bg-ash/5 p-8 text-center">
        <!-- Error Icon -->
        <div class="w-20 h-20 rounded-2xl bg-gradient-to-br from-red-500/10 to-red-500/5 flex items-center justify-center text-red-500 mb-6 shadow-lg">
            <i class="ri-error-warning-line text-4xl"></i>
        </div>

        <!-- Title -->
        <h2 class="text-2xl font-bold text-ink mb-3">Application Error</h2>
        
        <!-- Description -->
        <p class="text-ink/60 mb-2 max-w-md">
            The application "<span class="font-mono text-ink/80">{{ kbId || 'Unknown' }}</span>" failed to load.
        </p>
        <p v-if="errorMessage" class="text-sm text-red-500/80 mb-6 font-mono max-w-md truncate">
            {{ errorMessage }}
        </p>
        <p v-else class="text-sm text-ink/40 mb-6">
            The required plugin renderer could not be found or crashed during initialization.
        </p>
        
        <!-- Action Buttons -->
        <div class="flex gap-4 mb-8">
            <button 
                @click="handleRetry" 
                class="px-6 py-2.5 bg-ink text-white rounded-lg hover:bg-ink/80 transition-all font-medium flex items-center gap-2 shadow-md hover:shadow-lg"
            >
                <i class="ri-refresh-line"></i>
                Retry
            </button>
            <button 
                @click="handleGoHome" 
                class="px-6 py-2.5 bg-white border border-ink/10 rounded-lg hover:bg-ink/5 transition-all text-ink/80 font-medium flex items-center gap-2"
            >
                <i class="ri-home-4-line"></i>
                Return Home
            </button>
        </div>

        <!-- Report Issue (Collapsible) -->
        <button 
            @click="showDetails = !showDetails"
            class="text-xs text-ink/40 hover:text-ink/60 transition-colors flex items-center gap-1"
        >
            <i :class="showDetails ? 'ri-arrow-up-s-line' : 'ri-arrow-down-s-line'"></i>
            {{ showDetails ? 'Hide Details' : 'Show Details / Report Issue' }}
        </button>

        <Transition
            enter-active-class="transition duration-200 ease-out"
            enter-from-class="opacity-0 -translate-y-2"
            enter-to-class="opacity-100 translate-y-0"
            leave-active-class="transition duration-150 ease-in"
            leave-from-class="opacity-100 translate-y-0"
            leave-to-class="opacity-0 -translate-y-2"
        >
            <div v-if="showDetails" class="mt-4 p-4 bg-ink/5 rounded-lg max-w-lg text-left">
                <p class="text-xs text-ink/60 mb-2">
                    <strong>KB ID:</strong> <span class="font-mono">{{ kbId || 'N/A' }}</span>
                </p>
                <p class="text-xs text-ink/60 mb-2">
                    <strong>Renderer:</strong> <span class="font-mono">{{ rendererId || 'N/A' }}</span>
                </p>
                <p v-if="errorStack" class="text-xs text-ink/40 font-mono whitespace-pre-wrap break-all max-h-32 overflow-auto">
                    {{ errorStack }}
                </p>
                
                <button 
                    @click="copyErrorDetails"
                    class="mt-3 text-xs px-3 py-1.5 bg-ink/10 hover:bg-ink/20 rounded transition-colors flex items-center gap-1"
                >
                    <i :class="copied ? 'ri-check-line text-green-500' : 'ri-clipboard-line'"></i>
                    {{ copied ? 'Copied!' : 'Copy to Clipboard' }}
                </button>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { ref, inject } from 'vue';
import { useRouter } from 'vue-router';

const props = defineProps<{
    kbId?: string;
    rendererId?: string;
    errorMessage?: string;
    errorStack?: string;
}>();

const router = useRouter();
const os = inject<{ launchApp: (id: string) => void; closeApp: (id: string) => void }>('os');

const showDetails = ref(false);
const copied = ref(false);

const handleGoHome = () => {
    if (props.kbId && os?.closeApp) {
        os.closeApp(props.kbId);
    }
    router.replace('/self-space');
};

const handleRetry = () => {
    if (props.kbId && os?.launchApp) {
        os.launchApp(props.kbId);
    } else {
        window.location.reload();
    }
};

const copyErrorDetails = async () => {
    const details = `
KB ID: ${props.kbId || 'N/A'}
Renderer: ${props.rendererId || 'N/A'}
Error: ${props.errorMessage || 'Unknown'}
Stack: ${props.errorStack || 'N/A'}
Timestamp: ${new Date().toISOString()}
    `.trim();

    try {
        await navigator.clipboard.writeText(details);
        copied.value = true;
        setTimeout(() => { copied.value = false; }, 2000);
    } catch (e) {
        console.error('Failed to copy:', e);
    }
};
</script>
