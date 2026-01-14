<script setup lang="ts">

const props = defineProps<{
  nodes: any[]
}>();

const scrollToNode = (id: string) => {
    // Try to find the element by ID (assuming SemanticNodeCards are rendered with id={client_id})
    // In SemanticNodeCard, it might need an ID attribute.
    // Let's assume the DOM element has the ID corresponding to client_id
    const element = document.getElementById(id);
    if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    } else {
        console.warn(`Node with id ${id} not found in DOM`);
    }
};
</script>

<template>
  <div class="h-full overflow-y-auto px-4 py-8">
     <h3 class="text-xs font-bold text-gray-400 dark:text-white/30 uppercase tracking-widest mb-6 border-b border-black/5 dark:border-white/5 pb-2">Structure</h3>
     <ul class="space-y-3">
         <li 
            v-for="node in nodes" 
            :key="node.id" 
            @click="scrollToNode(node.client_id)"
            class="group flex items-center text-sm text-gray-500 dark:text-white/40 hover:text-cyan-600 dark:hover:text-cyan-400 cursor-pointer transition-colors duration-300"
         >
             <div class="w-1.5 h-1.5 rounded-full bg-black/10 dark:bg-white/10 group-hover:bg-cyan-600 dark:group-hover:bg-cyan-400 mr-3 transition-colors"></div>
             <span class="font-mono text-[10px] uppercase opacity-50 mr-2 w-12 text-right">{{ node.type }}</span>
             <span class="truncate">{{ node.title || node.client_id }}</span>
         </li>
     </ul>
  </div>
</template>
