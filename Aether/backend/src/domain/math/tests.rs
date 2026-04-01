#[cfg(test)]
mod tests {
    use super::super::models::*;
    use super::super::portability;
    use super::super::service::MathService;
    use uuid::Uuid;

    fn setup_service() -> (MathService, Uuid) {
        let svc = MathService::new();
        let kb_id = Uuid::new_v4();
        (svc, kb_id)
    }

    // ── MATH-01: Formal Object Model ────────────────────────────────────

    #[test]
    fn test_node_types_complete() {
        assert_eq!(MathNodeType::all().len(), 9);
    }

    #[test]
    fn test_relation_types_complete() {
        assert_eq!(MathRelationType::all().len(), 5);
    }

    #[test]
    fn test_add_node() {
        let (svc, kb_id) = setup_service();
        let node = svc.add_node(
            kb_id,
            MathNodeType::Theorem,
            "Bolzano-Weierstrass".into(),
            "Every bounded sequence has a convergent subsequence.".into(),
            Some("thm:bw".into()),
            None,
        );

        assert_eq!(node.node_type, MathNodeType::Theorem);
        assert_eq!(node.label, "Bolzano-Weierstrass");
        assert_eq!(node.ref_label, Some("thm:bw".into()));

        let graph = svc.get_graph(kb_id);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_add_relation() {
        let (svc, kb_id) = setup_service();
        let def = svc.add_node(kb_id, MathNodeType::Definition, "Bounded Sequence".into(), "".into(), None, None);
        let thm = svc.add_node(kb_id, MathNodeType::Theorem, "BW Theorem".into(), "".into(), None, None);

        let rel = svc.add_relation(kb_id, thm.id, def.id, MathRelationType::UsesDefinition, None);
        assert!(rel.is_some());

        let graph = svc.get_graph(kb_id);
        assert_eq!(graph.relations.len(), 1);
    }

    #[test]
    fn test_add_relation_missing_node() {
        let (svc, kb_id) = setup_service();
        let node = svc.add_node(kb_id, MathNodeType::Theorem, "T".into(), "".into(), None, None);
        let rel = svc.add_relation(kb_id, node.id, Uuid::new_v4(), MathRelationType::DependsOn, None);
        assert!(rel.is_none());
    }

    // ── MATH-02: Graph Semantics ────────────────────────────────────────

    #[test]
    fn test_inspect_node() {
        let (svc, kb_id) = setup_service();
        let d = svc.add_node(kb_id, MathNodeType::Definition, "Def".into(), "".into(), None, None);
        let t = svc.add_node(kb_id, MathNodeType::Theorem, "Thm".into(), "".into(), None, None);
        svc.add_relation(kb_id, t.id, d.id, MathRelationType::DependsOn, None);

        let inspection = svc.inspect_node(kb_id, t.id).unwrap();
        assert_eq!(inspection.outgoing.len(), 1);
        assert_eq!(inspection.incoming.len(), 0);
        assert_eq!(inspection.dependency_depth, 1);
    }

    #[test]
    fn test_cycle_detection() {
        let (svc, kb_id) = setup_service();
        let a = svc.add_node(kb_id, MathNodeType::Theorem, "A".into(), "".into(), None, None);
        let b = svc.add_node(kb_id, MathNodeType::Theorem, "B".into(), "".into(), None, None);
        let c = svc.add_node(kb_id, MathNodeType::Theorem, "C".into(), "".into(), None, None);

        // Create cycle: A -> B -> C -> A
        svc.add_relation(kb_id, a.id, b.id, MathRelationType::DependsOn, None);
        svc.add_relation(kb_id, b.id, c.id, MathRelationType::DependsOn, None);
        svc.add_relation(kb_id, c.id, a.id, MathRelationType::DependsOn, None);

        let analysis = svc.analyze_dependencies(kb_id);
        assert!(!analysis.cycles.is_empty(), "Should detect circular dependency");
    }

    #[test]
    fn test_topological_sort_no_cycles() {
        let (svc, kb_id) = setup_service();
        let d = svc.add_node(kb_id, MathNodeType::Definition, "Def".into(), "".into(), None, None);
        let l = svc.add_node(kb_id, MathNodeType::Lemma, "Lem".into(), "".into(), None, None);
        let t = svc.add_node(kb_id, MathNodeType::Theorem, "Thm".into(), "".into(), None, None);

        svc.add_relation(kb_id, l.id, d.id, MathRelationType::DependsOn, None);
        svc.add_relation(kb_id, t.id, l.id, MathRelationType::DependsOn, None);

        let analysis = svc.analyze_dependencies(kb_id);
        assert!(analysis.cycles.is_empty());
        assert_eq!(analysis.topological_order.len(), 3);
    }

    // ── MATH-03: Workspace Commands ─────────────────────────────────────

    #[test]
    fn test_workspace_mark_incomplete() {
        let (svc, kb_id) = setup_service();
        let p = svc.add_node(kb_id, MathNodeType::Proof, "Proof of T1".into(), "...".into(), None, None);

        let result = svc.execute_workspace_command(
            kb_id,
            WorkspaceCommand::MarkIncompleteProof { node_id: p.id },
        );
        assert!(result.success);

        let graph = svc.get_graph(kb_id);
        let node = graph.nodes.iter().find(|n| n.id == p.id).unwrap();
        assert_eq!(node.proof_status, Some(ProofStatus::Incomplete));
    }

    #[test]
    fn test_workspace_highlight_blockers() {
        let (svc, kb_id) = setup_service();
        let d = svc.add_node(kb_id, MathNodeType::Definition, "Def".into(), "".into(), None, None);
        let p = svc.add_node(kb_id, MathNodeType::Proof, "Proof".into(), "".into(), None, None);
        let t = svc.add_node(kb_id, MathNodeType::Theorem, "Thm".into(), "".into(), None, None);

        // Thm depends on incomplete proof
        svc.add_relation(kb_id, t.id, p.id, MathRelationType::DependsOn, None);
        svc.add_relation(kb_id, t.id, d.id, MathRelationType::DependsOn, None);

        let result = svc.execute_workspace_command(
            kb_id,
            WorkspaceCommand::HighlightBlockers { node_id: t.id },
        );
        assert!(result.success);
        // p is incomplete proof, d is not a proof
        let blockers = result.blockers.unwrap();
        assert!(blockers.contains(&p.id));
        assert!(!blockers.contains(&d.id));
    }

    // ── MATH-05: Reference Validation ───────────────────────────────────

    #[test]
    fn test_duplicate_label_detection() {
        let (svc, kb_id) = setup_service();
        svc.add_node(kb_id, MathNodeType::Theorem, "T1".into(), "".into(), Some("thm:main".into()), None);
        svc.add_node(kb_id, MathNodeType::Theorem, "T2".into(), "".into(), Some("thm:main".into()), None);

        let validation = svc.validate_references(kb_id);
        assert_eq!(validation.duplicate_labels.len(), 1);
        assert_eq!(validation.duplicate_labels[0].label, "thm:main");
    }

    #[test]
    fn test_unresolved_ref_detection() {
        let (svc, kb_id) = setup_service();
        svc.add_node(
            kb_id,
            MathNodeType::Theorem,
            "T1".into(),
            "By \\ref{lem:missing} we have...".into(),
            Some("thm:t1".into()),
            None,
        );

        let validation = svc.validate_references(kb_id);
        assert_eq!(validation.unresolved_refs.len(), 1);
        assert_eq!(validation.unresolved_refs[0].ref_label, "lem:missing");
    }

    // ── MATH-06: Portability ────────────────────────────────────────────

    #[test]
    fn test_json_export_import_roundtrip() {
        let (svc, kb_id) = setup_service();
        let d = svc.add_node(kb_id, MathNodeType::Definition, "Def A".into(), "content".into(), Some("def:a".into()), None);
        let t = svc.add_node(kb_id, MathNodeType::Theorem, "Thm B".into(), "content".into(), Some("thm:b".into()), None);
        svc.add_relation(kb_id, t.id, d.id, MathRelationType::UsesDefinition, None);

        let graph = svc.get_graph(kb_id);
        let export = portability::export_json_graph(&graph);
        assert_eq!(export.node_count, 2);
        assert_eq!(export.relation_count, 1);

        // Import into new KB
        let new_kb_id = Uuid::new_v4();
        let empty_graph = MathGraph::new(new_kb_id);
        let (imported, result) = portability::import_json_graph(
            new_kb_id,
            &export.content,
            &empty_graph,
            LabelCollisionStrategy::Skip,
        )
        .unwrap();

        assert_eq!(result.nodes_created, 2);
        assert_eq!(result.relations_created, 1);
        assert_eq!(imported.nodes.len(), 2);
    }

    #[test]
    fn test_label_collision_skip() {
        let (svc, kb_id) = setup_service();
        svc.add_node(kb_id, MathNodeType::Definition, "Existing".into(), "".into(), Some("def:a".into()), None);
        let graph = svc.get_graph(kb_id);

        // Create import content with same label
        let import_svc = MathService::new();
        let import_kb = Uuid::new_v4();
        import_svc.add_node(import_kb, MathNodeType::Definition, "Incoming".into(), "new".into(), Some("def:a".into()), None);
        let import_graph = import_svc.get_graph(import_kb);
        let export = portability::export_json_graph(&import_graph);

        let (merged, result) = portability::import_json_graph(
            kb_id,
            &export.content,
            &graph,
            LabelCollisionStrategy::Skip,
        )
        .unwrap();

        assert_eq!(result.nodes_skipped, 1);
        assert_eq!(result.nodes_created, 0);
        assert_eq!(merged.nodes.len(), 1); // Only existing
    }

    #[test]
    fn test_label_collision_rename() {
        let (svc, kb_id) = setup_service();
        svc.add_node(kb_id, MathNodeType::Definition, "Existing".into(), "".into(), Some("def:a".into()), None);
        let graph = svc.get_graph(kb_id);

        let import_svc = MathService::new();
        let import_kb = Uuid::new_v4();
        import_svc.add_node(import_kb, MathNodeType::Definition, "Incoming".into(), "new".into(), Some("def:a".into()), None);
        let import_graph = import_svc.get_graph(import_kb);
        let export = portability::export_json_graph(&import_graph);

        let (merged, result) = portability::import_json_graph(
            kb_id,
            &export.content,
            &graph,
            LabelCollisionStrategy::Rename,
        )
        .unwrap();

        assert_eq!(result.nodes_renamed, 1);
        assert_eq!(merged.nodes.len(), 2);
        // The imported node should have _imported suffix
        let imported_node = merged.nodes.iter().find(|n| n.label == "Incoming").unwrap();
        assert_eq!(imported_node.ref_label, Some("def:a_imported".into()));
    }

    #[test]
    fn test_markdown_export() {
        let (svc, kb_id) = setup_service();
        svc.add_node(kb_id, MathNodeType::Theorem, "BW".into(), "Every bounded seq...".into(), Some("thm:bw".into()), None);
        svc.add_node(kb_id, MathNodeType::Definition, "Bounded".into(), "A sequence is bounded if...".into(), None, None);

        let graph = svc.get_graph(kb_id);
        let result = portability::export_markdown_manuscript(&graph);
        assert!(result.content.contains("Theorem"));
        assert!(result.content.contains("BW"));
        assert!(result.content.contains("\\label{thm:bw}"));
    }

    #[test]
    fn test_latex_export() {
        let (svc, kb_id) = setup_service();
        svc.add_node(kb_id, MathNodeType::Theorem, "Main".into(), "$\\forall x$".into(), Some("thm:main".into()), None);

        let graph = svc.get_graph(kb_id);
        let result = portability::export_latex_package(&graph);
        assert!(result.content.contains("\\begin{theorem}"));
        assert!(result.content.contains("\\label{thm:main}"));
        assert!(result.content.contains("\\end{theorem}"));
    }
}
