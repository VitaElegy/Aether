<script setup lang="ts">
/**
 * AppErrorBoundary — Vue Error Boundary for Self Space modules.
 *
 * Wraps any module component. If the child throws during render/lifecycle,
 * this component catches the error and shows BrokenState instead of crashing
 * the entire Shell.
 *
 * Usage:
 *   <AppErrorBoundary :kbId="id" :rendererId="renderer" @crash="onCrash">
 *     <SomeModuleComponent />
 *   </AppErrorBoundary>
 */
import { ref, onErrorCaptured, watch } from 'vue';
import BrokenState from './BrokenState.vue';

const props = defineProps<{
    kbId?: string;
    rendererId?: string;
}>();

const emit = defineEmits<{
    crash: [kbId: string | undefined, error: Error];
}>();

const crashed = ref(false);
const errorMessage = ref('');
const errorStack = ref('');

onErrorCaptured((err: unknown, _instance, info) => {
    const error = err instanceof Error ? err : new Error(String(err));
    console.error(`[ErrorBoundary] Caught error in module '${props.kbId}':`, error, info);

    crashed.value = true;
    errorMessage.value = error.message;
    errorStack.value = error.stack ?? '';

    emit('crash', props.kbId, error);

    // Prevent the error from propagating further up the tree
    return false;
});

// Reset error state when the kbId changes (user switches module)
watch(() => props.kbId, () => {
    crashed.value = false;
    errorMessage.value = '';
    errorStack.value = '';
});

function handleRetry() {
    crashed.value = false;
    errorMessage.value = '';
    errorStack.value = '';
}

defineExpose({ crashed, handleRetry });
</script>

<template>
    <BrokenState
        v-if="crashed"
        :kbId="kbId"
        :rendererId="rendererId"
        :errorMessage="errorMessage"
        :errorStack="errorStack"
        @retry="handleRetry"
    />
    <slot v-else />
</template>
