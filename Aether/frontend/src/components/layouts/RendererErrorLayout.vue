<script setup lang="ts">

const props = defineProps<{
    rendererId?: string;
    error?: Error;
}>();
</script>

<template>
  <div class="h-screen w-full flex flex-col items-center justify-center bg-[#0a0f14] text-white">
      <div class="max-w-md text-center space-y-4 p-8 border border-red-500/20 rounded bg-red-900/5">
          <div class="text-red-500 text-4xl mb-2">⚠️</div>
          <h2 class="text-xl font-bold tracking-widest uppercase text-red-500">Renderer Malfunction</h2>
          
          <p class="text-sm text-gray-400">
              The requested visualization engine <span v-if="rendererId" class="font-mono text-cyan-500">{{ rendererId }}</span> could not be initialized.
          </p>

          <p v-if="error" class="text-xs font-mono text-red-400/70 bg-black/20 p-2 rounded text-left overflow-auto max-h-32">
              {{ error.message }}
          </p>

          <div class="pt-4">
              <button 
                @click="$router.push('/')"
                class="px-4 py-2 bg-white/5 hover:bg-white/10 text-xs uppercase tracking-widest transition-colors"
              >
                  Return to Safety
              </button>
          </div>
      </div>
  </div>
</template>
