<template>
  <div class="diff-viewer border border-gray-200 rounded-lg overflow-hidden bg-white shadow-sm font-mono text-sm leading-6">
    <div v-if="changes && changes.length > 0" class="overflow-x-auto">
      <table class="w-full border-collapse table-fixed">
        <colgroup>
           <col class="w-12 bg-gray-50 border-r border-gray-100 select-none" />
           <col class="w-full" />
        </colgroup>
        <tbody>
          <tr v-for="(change, idx) in changes" :key="idx"
              class="relative transition-colors duration-75"
              :class="{
                  'bg-emerald-50': change.tag === 'Insert',
                  'bg-rose-50': change.tag === 'Delete',
                  'hover:bg-gray-50': change.tag === 'Equal'
              }"
              :style="{
                  backgroundColor: change.tag === 'Insert' ? '#ecfdf5' : (change.tag === 'Delete' ? '#fff1f2' : undefined)
              }">

            <!-- Sidebar: Symbol -->
            <td class="select-none text-right px-3 py-0.5 text-xs font-semibold border-r border-gray-100 align-top opacity-70">
               <span v-if="change.tag === 'Insert'" class="text-emerald-600 select-none block w-full" style="color: #059669">+</span>
               <span v-else-if="change.tag === 'Delete'" class="text-rose-600 select-none block w-full" style="color: #e11d48">-</span>
               <span v-else class="text-gray-300 select-none block w-full">&nbsp;</span>
            </td>

            <!-- Content -->
            <td class="px-4 py-0.5 whitespace-pre-wrap break-all align-top font-medium"
                :class="{
                    'text-gray-900': change.tag !== 'Equal',
                    'text-gray-400 opacity-60': change.tag === 'Equal' && dimUnchanged,
                    'text-gray-600': change.tag === 'Equal' && !dimUnchanged
                }">
                <span>{{ change.value }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-else class="flex flex-col items-center justify-center py-12 text-gray-400 bg-gray-50/50">
       <span class="italic text-sm">{{ emptyMessage || 'No changes detected' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
interface DiffChange {
    tag: 'Equal' | 'Insert' | 'Delete';
    value: string;
}

defineProps<{
    changes: DiffChange[] | null;
    dimUnchanged?: boolean;
    emptyMessage?: string;
}>();
</script>

<style scoped>
.diff-viewer {
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas, Menlo, monospace;
}
</style>
