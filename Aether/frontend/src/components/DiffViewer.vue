<template>
  <div class="diff-viewer border border-gray-200 rounded-lg overflow-hidden bg-white shadow-sm">
    <div v-if="changes && changes.length > 0" class="overflow-x-auto">
      <table class="w-full border-collapse text-sm font-mono leading-relaxed">
        <colgroup>
           <col class="w-12 bg-gray-50 border-r border-gray-200 select-none" />
           <col class="w-full" />
        </colgroup>
        <tbody>
          <tr v-for="(change, idx) in changes" :key="idx"
              class="transition-colors duration-75 hover:bg-opacity-80"
              :class="{
                  'bg-green-50': change.tag === 'Insert',
                  'bg-red-50': change.tag === 'Delete',
                  'hover:bg-gray-50': change.tag === 'Equal'
              }">

            <!-- Line Number / Symbol Column -->
            <td class="px-2 py-0.5 text-right text-xs text-gray-400 font-semibold border-r border-gray-200 align-top select-none">
               <span v-if="change.tag === 'Insert'" class="text-green-600 block w-full">+</span>
               <span v-else-if="change.tag === 'Delete'" class="text-red-600 block w-full">-</span>
               <span v-else class="block w-full">&nbsp;</span>
            </td>

            <!-- Content Column -->
            <td class="px-4 py-0.5 whitespace-pre-wrap break-all align-top"
                :class="{
                    'text-gray-900': change.tag !== 'Equal',
                    'text-gray-500': change.tag === 'Equal' && dimUnchanged,
                    'text-gray-800': change.tag === 'Equal' && !dimUnchanged
                }">
                <span>{{ change.value }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-else class="flex flex-col items-center justify-center p-12 text-gray-400 bg-gray-50">
       <svg class="w-12 h-12 mb-3 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path></svg>
       <span class="text-sm font-medium">No differences found</span>
    </div>
  </div>
</template>

<script setup lang="ts">
interface DiffChange {
    tag: 'Equal' | 'Insert' | 'Delete';
    value: string;
}

defineProps<{
    changes: DiffChange[];
    dimUnchanged?: boolean;
}>();
</script>

<style scoped>
.diff-viewer {
  font-family: 'JetBrains Mono', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}
</style>
