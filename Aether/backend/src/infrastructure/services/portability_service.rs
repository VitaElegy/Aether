use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use uuid::Uuid;
use tokio::sync::mpsc::{self, Sender, Receiver};
use crate::domain::portability::ports::PortabilityProvider;
use crate::domain::portability::models::{ExportSummary, ImportSummary, ProgressEvent};

pub struct PortabilityService {
    providers: HashMap<String, Arc<dyn PortabilityProvider>>,
    // Simple in-memory task tracking for MVP. 
    // In production, this might need Redis or DB to survive restarts, 
    // but for "Download" tasks, memory is usually fine.
    active_tasks: Arc<RwLock<HashMap<Uuid, Receiver<ProgressEvent>>>>,
}

impl PortabilityService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_provider(&mut self, provider: Arc<dyn PortabilityProvider>) {
        self.providers.insert(provider.provider_id(), provider);
    }

    fn get_provider(&self, renderer_id: &str) -> Result<Arc<dyn PortabilityProvider>, String> {
        self.providers.get(renderer_id)
            .cloned()
            .ok_or_else(|| format!("No portability provider found for type: {}", renderer_id))
    }

    pub async fn analyze_export(&self, kb_type: &str, kb_id: Uuid) -> Result<ExportSummary, String> {
        let provider = self.get_provider(kb_type)?;
        provider.analyze_export(kb_id).await
    }

    pub async fn start_export(&self, kb_type: &str, kb_id: Uuid) -> Result<Uuid, String> {
        let provider = self.get_provider(kb_type)?;
        let task_id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(100);

        // Store receiver
        self.active_tasks.write().unwrap().insert(task_id, rx);

        // Spawn task
        tokio::spawn(async move {
            // We need to handle the result, maybe store it somewhere or just log it
            // For file downloads, we usually need the final path.
            // The ProgressEvent should probably carry the final result URL/Path in the "Completed" stage.
            let _ = provider.export(kb_id, task_id, tx.clone()).await;
        });

        Ok(task_id)
    }

    // This is a simplified polling mechanism. 
    // Real-world would use SSE, but for now we'll just pop the latest event or peek?
    // MPSC consumes messages. So "polling" means "give me all events since last check".
    // Actually, for a simple HTTP polling API, we might want a broadcast channel or just store the *latest* state.
    // Let's assume the client connects via SSE to `api/portability/tasks/:id/progress`.
    // Then we can just hand them the Receiver? No, Receiver is not cloneable.
    // We need to bridge the Receiver to the SSE stream.
    
    pub fn get_task_receiver(&self, task_id: Uuid) -> Option<Receiver<ProgressEvent>> {
        self.active_tasks.write().unwrap().remove(&task_id)
    }
}
