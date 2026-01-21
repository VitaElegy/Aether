use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, 
    QueryFilter, Set, TransactionTrait
};
use uuid::Uuid;
use regex::Regex;
use serde_json::{Value, json};
use crate::infrastructure::persistence::entities::{semantic_node};
use anyhow::Result;

#[derive(Clone)]
pub struct IndexerService {
    db: DatabaseConnection,
    block_regex: Regex,
}

impl IndexerService {
    pub fn new(db: DatabaseConnection) -> Self {
        // Rust regex for block start: ^:::\s*(\w+)\s*(\{.*?\})?$
        // Use recursive mode if needed but standard is fine
        let block_regex = Regex::new(r"(?m)^:::\s*(\w+)\s*(\{.*?\})?$").expect("Invalid Regex");
        Self { db, block_regex }
    }

    pub async fn index_article(&self, article_id: Uuid, content: &str) -> Result<()> {
        let nodes = self.parse_markdown(content);
        
        // Transaction: Delete old nodes -> Insert new nodes
        let txn = self.db.begin().await?;
        
        // 1. Delete existing for this article
        // Note: Using delete_many might need careful execution
        semantic_node::Entity::delete_many()
            .filter(semantic_node::Column::ArticleId.eq(article_id))
            .exec(&txn)
            .await?;
            
        // 2. Insert new
        let count = nodes.len();
        for node in nodes {
            let active_node = semantic_node::ActiveModel {
                id: Set(Uuid::new_v4()),
                article_id: Set(article_id),
                client_id: Set(node.client_id),
                r#type: Set(node.r#type),
                title: Set(node.title),
                content: Set(Some(node.content)),
                metrics: Set(node.metrics),
                ..Default::default()
            };
            active_node.insert(&txn).await?;
        }
        
        txn.commit().await?;
        
        tracing::info!("Indexed article {}: {} nodes created", article_id, count);
        Ok(())
    }
    
    fn parse_markdown(&self, content: &str) -> Vec<ParsedNode> {
        let mut nodes = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_block: Option<ParsedNodeBuilder> = None;
        let mut current_id_counter = 0;
        
        for (_i, line) in lines.iter().enumerate() {
            let trim_line = line.trim();
            
            // Check Start
            if let Some(caps) = self.block_regex.captures(trim_line) {
                 if let Some(b) = current_block.take() {
                     nodes.push(b.build(current_id_counter));
                     current_id_counter += 1;
                 }
                 
                 let type_ = caps.get(1).map_or("", |m| m.as_str()).to_string();
                 let metrics_str = caps.get(2).map_or("{}", |m| m.as_str());
                 let metrics: Value = serde_json::from_str(metrics_str).unwrap_or(json!({}));
                 
                 current_block = Some(ParsedNodeBuilder {
                     type_,
                     metrics,
                     content_lines: Vec::new(),
                 });
                 continue;
            }
            
            // Check End
            if trim_line == ":::" {
                if let Some(b) = current_block.take() {
                    nodes.push(b.build(current_id_counter));
                    current_id_counter += 1;
                }
                continue;
            }
            
            // Accumulate Content
            if let Some(ref mut b) = current_block {
                b.content_lines.push(line.to_string());
            }
        }
        
        nodes
    }
}

struct ParsedNode {
    client_id: String,
    r#type: String,
    title: Option<String>,
    content: String,
    metrics: Value,
}

struct ParsedNodeBuilder {
    type_: String,
    metrics: Value,
    content_lines: Vec<String>,
}

impl ParsedNodeBuilder {
    fn build(self, counter: usize) -> ParsedNode {
        let client_id = self.metrics.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("{}-{}", self.type_, counter))
            .to_string();
            
        let title = self.metrics.get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        ParsedNode {
            client_id,
            r#type: self.type_,
            title,
            content: self.content_lines.join("\n"),
            metrics: self.metrics,
        }
    }
}
