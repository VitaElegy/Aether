/*!
 * PLAT-06: Observability Foundation
 *
 * Domain types for audit events, task telemetry, and error boundary events.
 * Provides structured observability for all Special KBs.
 */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================
// AUDIT EVENT
// ============================================================

/// Tracks user/system actions for audit trail purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Who performed the action (user ID or "system")
    pub actor: String,
    /// Type of target entity (e.g., "kb", "article", "vocabulary_entry")
    pub target_type: String,
    /// ID of the target entity
    pub target_id: String,
    /// Action performed (e.g., "create", "update", "delete", "export", "import")
    pub action: String,
    /// Additional context (key-value pairs)
    pub context: HashMap<String, String>,
    /// Result of the action
    pub result: AuditResult,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure(String),
    Partial(String),
}

impl AuditEvent {
    pub fn new(
        actor: impl Into<String>,
        target_type: impl Into<String>,
        target_id: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            actor: actor.into(),
            target_type: target_type.into(),
            target_id: target_id.into(),
            action: action.into(),
            context: HashMap::new(),
            result: AuditResult::Success,
            timestamp: Utc::now(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    pub fn with_failure(mut self, reason: impl Into<String>) -> Self {
        self.result = AuditResult::Failure(reason.into());
        self
    }
}

// ============================================================
// TASK TELEMETRY
// ============================================================

/// Tracks async task execution for performance monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTelemetry {
    /// Unique task identifier
    pub task_id: Uuid,
    /// Type of task (e.g., "export", "import", "indexing", "graph_compute")
    pub task_type: String,
    /// When the task started
    pub started_at: DateTime<Utc>,
    /// When the task completed (None if still running)
    pub completed_at: Option<DateTime<Utc>>,
    /// Duration in milliseconds (computed on completion)
    pub duration_ms: Option<u64>,
    /// Current status
    pub status: TaskStatus,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
    TimedOut,
}

impl TaskTelemetry {
    pub fn start(task_type: impl Into<String>) -> Self {
        Self {
            task_id: Uuid::new_v4(),
            task_type: task_type.into(),
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            status: TaskStatus::Running,
            metadata: HashMap::new(),
        }
    }

    pub fn start_with_id(task_id: Uuid, task_type: impl Into<String>) -> Self {
        Self {
            task_id,
            task_type: task_type.into(),
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            status: TaskStatus::Running,
            metadata: HashMap::new(),
        }
    }

    pub fn complete(&mut self) {
        let now = Utc::now();
        self.completed_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds() as u64);
        self.status = TaskStatus::Completed;
    }

    pub fn fail(&mut self, reason: impl Into<String>) {
        let now = Utc::now();
        self.completed_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds() as u64);
        self.status = TaskStatus::Failed(reason.into());
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

// ============================================================
// ERROR BOUNDARY EVENT
// ============================================================

/// Captures module-level errors for structured error reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBoundaryEvent {
    /// Unique error identifier
    pub error_id: Uuid,
    /// Module that encountered the error (e.g., "vrkb", "english_v1", "portability")
    pub module_id: String,
    /// Classification of error
    pub error_type: ErrorType,
    /// Human-readable error message
    pub message: String,
    /// Optional stack trace
    pub stack_trace: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    /// Runtime panic or unexpected crash
    Crash,
    /// Network/IO failure
    NetworkError,
    /// Data validation failure
    ValidationError,
    /// Permission denied
    AuthError,
    /// Resource not found
    NotFound,
    /// Rate limit or quota exceeded
    RateLimited,
    /// Plugin/module load failure
    PluginError,
    /// Unknown/unclassified
    Unknown,
}

impl ErrorBoundaryEvent {
    pub fn new(
        module_id: impl Into<String>,
        error_type: ErrorType,
        message: impl Into<String>,
    ) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            module_id: module_id.into(),
            error_type,
            message: message.into(),
            stack_trace: None,
            context: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}
