<template>
  <Transition name="fade">
    <div class="fixed inset-0 bg-black/70 backdrop-blur-[4px] z-[100] flex justify-center items-center" @click.self="$emit('close')">
      
      <Transition name="slide-up">
        <div class="w-[700px] max-h-[90vh] flex flex-col bg-paper shadow-2xl rounded-xl overflow-hidden border border-ash/20 relative">
          
          <!-- Close Button (Floating) -->
           <button @click="$emit('close')" class="absolute top-4 right-4 p-2 text-ink/40 hover:text-ink transition-colors z-10 rounded-full hover:bg-black/5">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
           </button>

          <!-- Header -->
          <header class="px-8 pt-8 pb-4 bg-paper shrink-0">
             <div class="flex items-center gap-2 mb-2">
                <div :class="severityPillClass" class="w-2 h-2 rounded-full"></div>
                <span class="text-[10px] font-black uppercase tracking-[0.2em] text-ink/40">{{ localFinding.status }}</span>
             </div>
             <input 
               v-model="localFinding.title"
               placeholder="Enter Finding Title..." 
               class="w-full text-3xl font-serif font-bold text-ink bg-transparent placeholder-ink/20 outline-none leading-tight"
             />
          </header>

          <!-- Content Body -->
          <div class="flex-1 overflow-y-auto px-8 py-4 space-y-8 custom-scrollbar">
            
            <!-- Controls Row -->
            <div class="grid grid-cols-4 gap-6 border-b border-ash/30 pb-6">
               <div class="space-y-1">
                  <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 block">Severity</label>
                  <div class="flex items-center gap-2">
                     <select v-model="localFinding.severity" class="bg-transparent text-sm font-medium text-ink outline-none cursor-pointer hover:text-accent transition-colors">
                        <option value="Low">Low</option>
                        <option value="Medium">Medium</option>
                        <option value="High">High</option>
                        <option value="Critical">Critical</option>
                        <option value="Info">Info</option>
                     </select>
                     <i class="ri-arrow-down-s-line text-ink/40 text-xs"></i>
                  </div>
               </div>

               <div class="space-y-1">
                  <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 block">Status</label>
                  <div class="flex items-center gap-2">
                     <select v-model="localFinding.status" class="bg-transparent text-sm font-medium text-ink outline-none cursor-pointer hover:text-accent transition-colors">
                        <option value="triage">Triage</option>
                        <option value="confirmed">Confirmed</option>
                        <option value="exploiting">Exploiting</option>
                        <option value="fixing">Fixing</option>
                        <option value="verifying">Verifying</option>
                        <option value="closed">Closed</option>
                        <option value="risk_accepted">Risk Accepted</option>
                     </select>
                     <i class="ri-arrow-down-s-line text-ink/40 text-xs"></i>
                  </div>
               </div>

               <div class="space-y-1">
                  <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 block">Confidence</label>
                  <div class="flex items-center gap-2">
                     <select v-model="localFinding.confidence" class="bg-transparent text-sm font-medium text-ink outline-none cursor-pointer hover:text-accent transition-colors">
                        <option :value="undefined">—</option>
                        <option value="certain">Certain</option>
                        <option value="firm">Firm</option>
                        <option value="tentative">Tentative</option>
                     </select>
                     <i class="ri-arrow-down-s-line text-ink/40 text-xs"></i>
                  </div>
               </div>

               <div class="space-y-1">
                  <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 block">Due Date</label>
                  <input 
                    type="date" 
                    :value="localFinding.due_date ? localFinding.due_date.substring(0, 10) : ''"
                    @input="localFinding.due_date = ($event.target as HTMLInputElement)?.value ? new Date(($event.target as HTMLInputElement).value).toISOString() : undefined"
                    class="bg-transparent text-sm font-medium text-ink outline-none cursor-pointer hover:text-accent transition-colors"
                  />
               </div>
            </div>

            <!-- Repro Steps -->
            <div class="space-y-3">
               <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 block">Reproduction Steps</label>
               <textarea 
                 v-model="localFinding.repro_steps" 
                 class="w-full min-h-[120px] bg-bg-surface rounded-lg p-4 outline-none text-ink text-sm leading-relaxed font-mono resize-none border border-transparent focus:border-accent/20 transition-all placeholder-ink/20"
                 placeholder="Step-by-step instructions to reproduce this vulnerability..."
               ></textarea>
            </div>

            <!-- Remediation -->
            <div class="space-y-3">
               <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 block">Remediation Guidance</label>
               <textarea 
                 v-model="localFinding.remediation" 
                 class="w-full min-h-[100px] bg-bg-surface rounded-lg p-4 outline-none text-ink text-sm leading-relaxed resize-none border border-transparent focus:border-accent/20 transition-all placeholder-ink/20"
                 placeholder="Recommended fix, mitigation, or remediation steps..."
               ></textarea>
            </div>

            <!-- Verification Note -->
            <div class="space-y-3">
               <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 block">Verification Note</label>
               <textarea 
                 v-model="localFinding.verification_note" 
                 class="w-full min-h-[80px] bg-bg-surface rounded-lg p-4 outline-none text-ink text-sm leading-relaxed resize-none border border-transparent focus:border-accent/20 transition-all placeholder-ink/20"
                 placeholder="Notes from fix verification (filled when verifying)..."
               ></textarea>
            </div>

            <!-- Editor Area (Audit Blueprint) -->
            <div class="space-y-3">
               <label class="text-[9px] font-black uppercase tracking-widest text-ink/40 flex justify-between">
                 <span>Audit Blueprint</span>
                 <span class="text-accent cursor-pointer hover:underline">Insert Template</span>
               </label>
               <div class="relative group min-h-[200px]">
                 <textarea 
                   v-model="textContent" 
                   class="w-full h-full min-h-[200px] bg-bg-surface rounded-lg p-6 outline-none text-ink text-base leading-relaxed font-serif resize-none border border-transparent focus:border-accent/20 transition-all placeholder-ink/20"
                   placeholder="Describe the vulnerability, evidence, and additional context..."
                 ></textarea>
               </div>
            </div>

          </div>

          <!-- Footer Action -->
          <footer class="p-6 bg-surface border-t border-ash/20 flex justify-end gap-3 shrink-0">
             <button @click="save" :disabled="isSaving" class="px-8 py-2.5 bg-accent text-white rounded-lg shadow-lg shadow-accent/20 hover:shadow-accent/40 hover:-translate-y-0.5 transition-all text-xs font-black uppercase tracking-widest disabled:opacity-50 disabled:translate-y-0">
               {{ isSaving ? 'Saving...' : 'Save Finding' }}
             </button>
          </footer>

        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { VrkbFinding } from '@/api/vrkb';

const props = defineProps<{
  finding: VrkbFinding;
}>();

const emit = defineEmits(['close', 'save']);

const localFinding = ref<VrkbFinding>({ ...props.finding });
const isSaving = ref(false);
const textContent = ref('');

watch(() => props.finding, (newVal) => {
  localFinding.value = { ...newVal };
  if (newVal.content && typeof newVal.content === 'object' && 'text' in newVal.content) {
      textContent.value = (newVal.content as any).text || '';
  }
}, { immediate: true });

const severityPillClass = computed(() => {
  switch (localFinding.value.severity) {
    case 'Critical': return 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]';
    case 'High': return 'bg-orange-500';
    case 'Medium': return 'bg-yellow-500';
    case 'Low': return 'bg-blue-500';
    default: return 'bg-gray-400';
  }
});

const save = async () => {
  isSaving.value = true;
  try {
    localFinding.value.content = { text: textContent.value };
    emit('save', localFinding.value);
  } finally {
    isSaving.value = false;
  }
};
</script>

<style scoped>
/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.98);
}
</style>
