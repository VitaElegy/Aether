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
      <div v-if="nodes.length === 0" class="text-ink/30 italic mt-20 font-serif">No semantic content extracted.</div>
      
      <div v-else class="w-full max-w-4xl space-y-12 pb-24">
          <div v-for="node in nodes" :key="node.id" :id="node.client_id" class="relative group pl-8">
              <!-- Organic Connection Line -->
              <div class="absolute left-0 top-0 bottom-0 w-[1px] bg-gradient-to-b from-ink/5 via-ink/10 to-transparent group-hover:via-accent/30 transition-all duration-500"></div>
              <div class="absolute left-[-3px] top-6 w-1.5 h-1.5 rounded-full bg-ink/20 group-hover:bg-accent group-hover:shadow-[0_0_8px_rgba(var(--color-accent),0.6)] transition-all duration-500"></div>

              <SemanticNodeCard 
                :node="mapToSemanticNode(node)"
                :interactive="true"
                class="shadow-xl"
              />
          </div>
      </div>
  </div>
</template>
<style scoped>
.bg-pattern {
    background-color: rgb(var(--color-paper));
    background-image: radial-gradient(circle at 1px 1px, rgba(var(--color-ink), 0.05) 1px, transparent 0);
    background-size: 32px 32px;
    transition: background-color 0.3s;
}

.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(var(--color-ink), 0.1);
    border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--color-ink), 0.2);
}
</style>
