<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import * as d3 from 'd3';


const emit = defineEmits<{
    (e: 'select-topic', topic: string | null): void,
    (e: 'node-click', node: TreeData): void
}>();

export interface TreeData {
    name: string;
    tag?: string;
    id?: string; // Phase 7: Tracking ID for editing
    children?: TreeData[];
}

const props = defineProps<{
    kbId?: string;
    treeData: TreeData | null;
}>();

const svgRef = ref<SVGSVGElement | null>(null);
const selectedTopic = ref<string | null>(null);

const draw = () => {
    if (!svgRef.value) return;
    if (!props.treeData) return; // Wait for data

    const width = svgRef.value.clientWidth;
    const height = svgRef.value.clientHeight;
    
    // Clear
    d3.select(svgRef.value).selectAll("*").remove();

    const svg = d3.select(svgRef.value)
        .attr("viewBox", [0, 0, width, height])
        .style("font", "10px sans-serif");
    
    // Margins
    const margin = { top: 40, right: 40, bottom: 40, left: 80 };
    const innerWidth = width - margin.left - margin.right;
    const innerHeight = height - margin.top - margin.bottom;

    // Draw Grid Background
    const defs = svg.append("defs");
    const pattern = defs.append("pattern")
        .attr("id", "grid")
        .attr("width", 40)
        .attr("height", 40)
        .attr("patternUnits", "userSpaceOnUse");
    
    pattern.append("path")
        .attr("d", "M 40 0 L 0 0 0 40")
        .attr("fill", "none")
        .attr("stroke", "currentColor")
        .attr("stroke-width", 0.5)
        .attr("stroke-opacity", 0.1);

    svg.append("rect")
        .attr("width", "100%")
        .attr("height", "100%")
        .attr("fill", "url(#grid)");

    const g = svg.append("g")
        .attr("transform", `translate(${margin.left},${margin.top})`);
    
    // Zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.5, 2])
        .on("zoom", (event) => {
            g.attr("transform", event.transform);
        });
    
    svg.call(zoom as any);
    svg.call(zoom.transform as any, d3.zoomIdentity.translate(margin.left, margin.top));


    // Layout
    const root = d3.hierarchy(props.treeData as any);
    
    // Tree layout
    const treeLayout = d3.tree().size([innerHeight, innerWidth]);
    treeLayout(root as any);

    // Links - PCB Style (Strict Orthogonal)
    g.selectAll(".link")
        .data(root.links())
        .join("path")
        .attr("class", "link")
        .attr("d", (d: any) => {
             // PCB Trace: Source -> Horizontal -> Vertical -> Target
             const midX = (d.source.y + d.target.y) / 2;
             return `M ${d.source.y} ${d.source.x} 
                     L ${midX} ${d.source.x} 
                     L ${midX} ${d.target.x} 
                     L ${d.target.y} ${d.target.x}`;
        })
        .attr("fill", "none")
        .attr("stroke", "currentColor")
        .attr("stroke-opacity", 0.2)
        .attr("stroke-width", (d: any) => 2 - d.target.depth * 0.5) // Thinner at leaves
        .attr("stroke-dasharray", (d: any) => d.target.depth === 1 ? "none" : "4 2") // Dashed for lower levels? Maybe solid is better for PCB.
        .attr("stroke-dasharray", "none")
        .transition().duration(1000)
        .attr("stroke-opacity", 0.4);

    // Nodes
    const nodes = g.selectAll(".node")
        .data(root.descendants())
        .join("g")
        .attr("class", "node cursor-pointer group")
        .attr("transform", (d: any) => `translate(${d.y},${d.x})`)
        .on("click", (event, d: any) => {
             event.stopPropagation();
             const tag = d.data.tag;
             if (selectedTopic.value === tag) {
                 selectedTopic.value = null; 
             } else {
                 selectedTopic.value = tag || null;
             }
             emit('select-topic', selectedTopic.value);
             emit('node-click', d.data);
             updateSelection();
        });

    // Geometric Semantics (Shapes)
    // Depth 0: Hexagon (Root)
    // Depth 1: Square (Domain)
    // Depth 2+: Circle (Item)
    
    nodes.each(function(d: any) {
        const el = d3.select(this);
        if (d.depth === 0) {
            // Hexagonish
             el.append("path")
               .attr("d", "M-6,-4 L0,-8 L6,-4 L6,4 L0,8 L-6,4 Z")
               .attr("fill", "rgb(var(--color-paper))")
               .attr("stroke", "currentColor")
               .attr("stroke-width", 2);
        } else if (d.depth === 1) {
            // Square
            el.append("rect")
              .attr("x", -6).attr("y", -6)
              .attr("width", 12).attr("height", 12)
              .attr("fill", "rgb(var(--color-paper))")
              .attr("stroke", "currentColor")
              .attr("stroke-width", 2);
        } else {
            // Circle or Diamond
            el.append("circle")
              .attr("r", 4)
              .attr("fill", "rgb(var(--color-paper))")
              .attr("stroke", "currentColor")
              .attr("stroke-width", 1.5);
        }
    });

    // Labels (Tech Style)
    nodes.append("text")
        .attr("dy", (d: any) => d.depth === 0 ? -15 : 0.35 + "em")
        .attr("x", (d: any) => d.depth === 0 ? 0 : (d.children ? -12 : 12))
        .attr("text-anchor", (d: any) => d.depth === 0 ? "middle" : (d.children ? "end" : "start"))
        .text((d: any) => d.data.name)
        .attr("class", "font-mono text-[9px] uppercase tracking-wider fill-current opacity-80 select-none")
        .style("font-family", "'JetBrains Mono', monospace") // Explicit Font
        .style("text-shadow", "0 0 2px rgba(0,0,0,0.8)"); // Readability
    
    // Animate Text In
    nodes.selectAll("text")
         .style("opacity", 0)
         .transition().duration(800).delay((d, i) => i * 50 + 200)
         .style("opacity", 0.7);


    // Update Selection Logic
    const updateSelection = () => {
         // Reset
         g.selectAll("path, rect, circle").attr("fill", "rgb(var(--color-paper))").attr("stroke", "currentColor");
         
         // Highlight
         nodes.each(function(d: any) {
             if (d.data.tag === selectedTopic.value) {
                 d3.select(this).selectAll("path, rect, circle")
                   .attr("fill", "rgb(var(--color-accent))")
                   .attr("stroke", "rgb(var(--color-accent))")
                   .style("filter", "drop-shadow(0 0 6px rgba(var(--color-accent), 0.6))");
             }
         });
    };

    // Initial check
    updateSelection();
};

onMounted(() => {
    draw();
    window.addEventListener('resize', draw);
});

watch(() => props.treeData, () => {
    draw();
}, { deep: true });
</script>

<template>
    <div class="w-full h-full bg-paper relative overflow-hidden text-ink font-mono transition-colors duration-300">
        <svg ref="svgRef" class="w-full h-full text-ink opacity-60"></svg>
        
        <!-- Decoration: Watermark -->
        <div class="absolute bottom-[-10%] right-[-5%] text-[200px] leading-none font-serif opacity-[0.03] select-none pointer-events-none text-ink">
            ∫
        </div>
    </div>
</template>
