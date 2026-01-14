<script setup lang="ts">
import SemanticNodeCard from '@/components/math/SemanticNodeCard.vue';

const props = defineProps<{
  nodes: any[]
}>();

const mapToSemanticNode = (n: any) => ({
    id: n.client_id, // Map client_id to id for frontend compatibility
    type: n.r_type || n.type, // Handle Rust r#type serialization usually 'type' field in JSON
    title: n.title,
    content: n.content || "",
    metrics: n.metrics,
    children: [],
    level: 0
});
</script>

<template>
  <div class="w-full h-full flex flex-col items-center overflow-y-auto p-12 bg-pattern custom-scrollbar">
      <div v-if="nodes.length === 0" class="text-gray-400 dark:text-white/20 italic mt-20">No semantic content extracted.</div>
      
      <div v-else class="w-full max-w-4xl space-y-12 pb-24">
          <div v-for="node in nodes" :key="node.id" :id="node.client_id" class="relative group pl-8">
              <!-- Organic Connection Line -->
              <div class="absolute left-0 top-0 bottom-0 w-[1px] bg-gradient-to-b from-black/5 via-black/10 to-transparent dark:from-white/5 dark:via-white/10 dark:to-white/5 group-hover:via-cyan-500/30 transition-all duration-500"></div>
              <div class="absolute left-[-3px] top-6 w-1.5 h-1.5 rounded-full bg-black/20 dark:bg-white/20 group-hover:bg-cyan-500 dark:group-hover:bg-cyan-500 group-hover:shadow-[0_0_8px_rgba(6,182,212,0.6)] transition-all duration-500"></div>

              <SemanticNodeCard 
                :node="mapToSemanticNode(node)"
                :interactive="true"
                class="shadow-xl dark:shadow-2xl shadow-black/5 dark:shadow-black/50"
              />
          </div>
      </div>
  </div>
</template>
<style scoped>
.bg-pattern {
    /* Light Mode Default */
    background-color: #f9fafb; /* gray-50 */
    background-image: radial-gradient(circle at 1px 1px, rgba(0,0,0,0.05) 1px, transparent 0);
    background-size: 32px 32px;
}

:global(.dark) .bg-pattern {
    /* Dark Mode Override */
    background-color: #0d1117;
    background-image: radial-gradient(circle at 1px 1px, rgba(255,255,255,0.03) 1px, transparent 0);
}

.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(0,0,0,0.1); /* Light mode scrollbar */
    border-radius: 3px;
}
:global(.dark) .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255,255,255,0.05); /* Dark mode scrollbar */
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(0,0,0,0.2);
}
:global(.dark) .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(255,255,255,0.1);
}
</style>
