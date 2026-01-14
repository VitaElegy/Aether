<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import * as d3 from 'd3';
import { SemanticBlockParser, type SemanticNode } from '@/utils/SemanticBlockParser';

interface Props {
    article: any;
}

const props = defineProps<Props>();
const emit = defineEmits(['node-selected']);
const container = ref<HTMLElement | null>(null);

const initGraph = () => {
    if (!container.value || !props.article) return;

    const width = container.value.clientWidth;
    const height = container.value.clientHeight;

    // Clear previous
    d3.select(container.value).selectAll('*').remove();

    const svg = d3.select(container.value)
        .append('svg')
        .attr('width', width)
        .attr('height', height)
        .attr('viewBox', [0, 0, width, height]);

    // Parse Content
    let content = '';
    if (typeof props.article.body === 'string') {
        content = props.article.body;
    } else if (props.article.body && typeof props.article.body === 'object') {
        // Handle ContentBody enum structure
        if (props.article.body.type === 'Markdown' && props.article.body.data) {
             content = props.article.body.data;
        } else if (props.article.body.content) {
             // Fallback for some legacy JSON structures
             content = props.article.body.content;
        }
    }



    console.log("ComputedTreeGraph: Content Length:", content.length);
    const semanticNodes = SemanticBlockParser.parse(content);
    console.log("ComputedTreeGraph: Parsed Nodes:", semanticNodes);
    
    // Convert flat list to Tree Structure for D3
    // 1. Root
    const data = {
        id: 'root',
        type: 'Article',
        title: props.article.title,
        content: '',
        children: semanticNodes.map(n => ({ ...n, children: [] })) // Flat children for now
    };

    const root = d3.hierarchy(data);
    
    // treeLayout(root);
    // TypeScript fix: d3 types can be strict about the data shape. Casting to any for flexibility.
    const treeLayout = d3.tree<any>().size([height, width - 200]);
    treeLayout(root as any);

    // Links (Orthogonal)
    const linkGenerator = d3.linkHorizontal()
        .x((d: any) => d.y)
        .y((d: any) => d.x);

    const g = svg.append('g').attr('transform', 'translate(80,0)');

    // Render Links
    g.selectAll('path')
        .data(root.links())
        .join('path')
        .attr('d', linkGenerator as any)
        .attr('fill', 'none')
        .attr('stroke', '#ccc')
        .attr('stroke-width', 1.5);

    // Render Nodes
    const nodes = g.selectAll('g')
        .data(root.descendants())
        .join('g')
        .attr('transform', (d: any) => `translate(${d.y},${d.x})`)
        .style('cursor', 'pointer')
        .on('click', (e, d: any) => {
             // Emit selected node
             // If root, we construct a fake semantic node. 
             // Ideally we pass full object.
             if (d.data.id === 'root') return;
             emit('node-selected', d.data);
        });

    nodes.append('circle')
        .attr('r', 5)
        .attr('fill', '#fff')
        .attr('stroke', '#000')
        .attr('stroke-width', 2);

    nodes.append('text')
        .attr('dy', '0.31em')
        .attr('x', (d: any) => d.children ? -10 : 10)
        .attr('text-anchor', (d: any) => d.children ? 'end' : 'start')
        .text((d: any) => d.data.title || d.data.type)
        .clone(true).lower()
        .attr('stroke', 'white');
};

onMounted(() => {
    initGraph();
});

watch(() => props.article, initGraph);
</script>

<template>
    <div ref="container" class="w-full h-full relative">
        <div class="absolute bottom-4 right-4 bg-paper/80 p-2 text-[10px] font-mono border border-ash">
            D3 Engine Active
        </div>
    </div>
</template>
