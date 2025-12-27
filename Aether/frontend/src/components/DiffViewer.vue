<template>
  <div class="diff-viewer border border-neutral-200 rounded-md overflow-hidden bg-white">
    <div v-if="changes && changes.length > 0" class="overflow-x-auto">
      <table class="w-full border-collapse table-fixed text-sm font-mono">
        <colgroup>
           <col class="w-12 bg-gray-50 border-r border-neutral-100" />
           <col class="w-full" />
        </colgroup>
        <tbody>
          <tr v-for="(change, idx) in changes" :key="idx"
              class="group hover:bg-black/5 transition-colors duration-75 relative"
              :class="{
                  'bg-[#e6ffec]': change.tag === 'Insert',
                  'bg-[#ffebe9]': change.tag === 'Delete',
              }">

            <!-- Line Number / Symbol Column -->
            <td class="select-none text-right px-2 py-0.5 text-xs text-neutral-400 font-bold border-r border-neutral-100/50 align-top leading-6">
               <span v-if="change.tag === 'Insert'" class="text-[#2da44e]">+</span>
               <span v-else-if="change.tag === 'Delete'" class="text-[#cf222e]">-</span>
               <span v-else>&nbsp;</span>
            </td>

            <!-- Content Column -->
            <td class="px-4 py-0.5 whitespace-pre-wrap break-all leading-6 align-top text-neutral-800"
                :class="{
                    'text-neutral-400 opacity-50': change.tag === 'Equal' && dimUnchanged
                }">
                <!-- Render spaces explicitly if needed, but whitespace-pre-wrap handles most -->
                <span>{{ change.value }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-else class="text-neutral-400 italic p-8 text-center bg-gray-50 text-sm">
      No differences.
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface DiffChange {
    tag: 'Equal' | 'Insert' | 'Delete';
    value: string;
}

const props = defineProps<{
    changes: DiffChange[];
    dimUnchanged?: boolean;
}>();
</script>

<style scoped>
.diff-viewer {
  font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
}
</style>
