use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;
use crate::domain::blocks::models::Block;
use crate::domain::blocks::strategies::extract_references;

pub struct ComputedTreeService;

impl ComputedTreeService {
    /// Given a flat list of blocks, compute a topological sort based on dependencies.
    /// Returns blocks in order: Independent (Axioms) -> Dependent (Theorems) -> Highly Dependent (Proofs).
    pub fn compute_topological_sort(blocks: Vec<Block>) -> Vec<Block> {
        let mut adj_list: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut block_map: HashMap<Uuid, Block> = HashMap::new();

        // 1. Initialize
        for block in &blocks {
            block_map.insert(block.id, block.clone());
            in_degree.entry(block.id).or_insert(0);
        }

        // 2. Build Graph
        for block in &blocks {
            let refs = extract_references(&block.type_name, &block.payload);
            for target_id in refs {
                if block_map.contains_key(&target_id) {
                    // target_id must come BEFORE block.id (Dependency)
                    // Edge: target -> block
                    adj_list.entry(target_id).or_default().push(block.id);
                    *in_degree.entry(block.id).or_insert(0) += 1;
                }
            }
        }

        // 3. Kahn's Algorithm
        let mut queue: VecDeque<Uuid> = VecDeque::new();
        
        // Push 0 in-degree nodes (Axioms, Independent Definitions)
        // Sort by ordinal to keep original author intent for ties
        let mut independent: Vec<Uuid> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
            
        // Sort key: Ordinal from original block
        independent.sort_by_key(|id| block_map.get(id).unwrap().ordinal);
        
        for id in independent {
            queue.push_back(id);
        }

        let mut sorted_blocks = Vec::new();

        while let Some(u) = queue.pop_front() {
            if let Some(blk) = block_map.get(&u) {
                sorted_blocks.push(blk.clone());
            }

            if let Some(neighbors) = adj_list.get(&u) {
                // Determine order for neighbors too
                 let mut next_batch = Vec::new();
                 
                for &v in neighbors {
                    if let Some(deg) = in_degree.get_mut(&v) {
                        *deg -= 1;
                        if *deg == 0 {
                            next_batch.push(v);
                        }
                    }
                }
                
                // Sort next batch by ordinal
                next_batch.sort_by_key(|id| block_map.get(id).unwrap().ordinal);
                
                for v in next_batch {
                    queue.push_back(v);
                }
            }
        }
        
        // TODO: Detect cycles? If sorted_blocks.len() != blocks.len(), cycle exists.
        // For now, append remaining blocks (cycle participants) at the end.
        if sorted_blocks.len() != blocks.len() {
             let processed: HashSet<Uuid> = sorted_blocks.iter().map(|b| b.id).collect();
             for block in blocks {
                 if !processed.contains(&block.id) {
                     sorted_blocks.push(block);
                 }
             }
        }

        sorted_blocks
    }
}
