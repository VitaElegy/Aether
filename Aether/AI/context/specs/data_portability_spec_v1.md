# Data Portability & Progress Specification V1

## 1. Objective
To provide a specialized, visualized, and verifiable import/export mechanism for different Knowledge Base types (e.g., English, Math, VRKB). This system goes beyond system-level backups by offering domain-specific data structuring, preview capabilities, and real-time progress tracking.

## 2. Architecture

### 2.1 The Portability Provider Pattern
The backend will implement a Strategy Pattern where each Knowledge Base Type (renderer_id) handles its own data serialization logic.

```rust
#[async_trait]
pub trait PortabilityProvider: Send + Sync {
    /// Return a human-readable summary of what will be exported
    async fn analyze_export(&self, kb_id: Uuid) -> Result<ExportSummary>;
    
    /// Execute the export, reporting progress via the channel
    async fn export(&self, kb_id: Uuid, progress: Sender<ProgressEvent>) -> Result<PathBuf>;
    
    /// Analyze an uploaded file for import
    async fn analyze_import(&self, file_path: PathBuf) -> Result<ImportSummary>;
    
    /// Execute import
    async fn import(&self, kb_id: Uuid, file_path: PathBuf, progress: Sender<ProgressEvent>) -> Result<ImportStats>;
}
```

### 2.2 Data Structures

**ExportSummary (The Preview)**
Returned before the actual download starts to let users know what they are getting.
```json
{
  "total_items": 150,
  "sections": [
    { "name": "Vocabulary", "count": 120, "details": "Words with 500+ examples" },
    { "name": "Reading", "count": 30, "details": "Articles and parsing metadata" }
  ],
  "estimated_size": "5.2 MB"
}
```

**ProgressEvent**
```json
{
  "task_id": "uuid",
  "stage": "Exporting Vocabulary",
  "percent": 45,
  "message": "Processing word: 'Ephemeral'"
}
```

## 3. Domain Specific Implementations

### 3.1 English Knowledge Base (EnglishProvider)
*   **Export Format**: Structured ZIP.
    *   `_PREVIEW.md`: Auto-generated human-readable summary (Markdown table of contents).
    *   `vocabulary.csv`: Standard CSV for easy viewing/import into Anki.
    *   `content/`: Folder containing Markdown files for articles.
    *   `raw_data.json`: Full data dump for lossless restore.
*   **Preview**: Shows counts of Words, Phrases, and Articles.

### 3.2 Generic KB (DefaultProvider)
*   Standard Backup format (already implemented, but wrapped in this new UI).

## 4. API Design

*   `GET /api/portability/:kb_id/export/preview` -> Returns `ExportSummary`
*   `POST /api/portability/:kb_id/export/start` -> Returns `{ task_id }`
*   `GET /api/portability/tasks/:task_id/progress` -> Returns `ProgressEvent` (SSE or Polling)
*   `GET /api/portability/tasks/:task_id/download` -> Serves the file

## 5. Frontend UI

1.  **Export Dialog**:
    *   Step 1: "Analyzing..." (Call Preview)
    *   Step 2: Show Preview Card (e.g., "Found 120 Words. Ready to export.")
    *   Step 3: "Exporting..." with **Progress Bar** (0-100%).
    *   Step 4: "Download Complete". Show "Open Preview" button (if possible, or just download).

2.  **Import Dialog**:
    *   Step 1: Upload File.
    *   Step 2: Show Analysis (e.g., "File contains 50 new words").
    *   Step 3: "Importing..." with Progress Bar.

## 6. Implementation Plan
1.  Define `PortabilityProvider` trait in Backend.
2.  Implement `EnglishPortabilityProvider`.
3.  Create `TaskRegistry` in AppState to hold active progress channels.
4.  Implement API Endpoints.
5.  Create Vue `ExportModal` and `ImportModal` components.
