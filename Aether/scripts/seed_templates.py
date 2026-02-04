import sqlite3
import uuid
import json
import datetime

DB_PATH = 'backend/aether.db'

def get_uuid_bytes():
    return uuid.uuid4().bytes

def now():
    return datetime.datetime.now().isoformat()

def seed():
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()
    except Exception as e:
        print(f"Failed to connect to DB: {e}")
        return

    print("Seeding Layout Templates...")

    templates = [
        {
            "renderer_id": "default",
            "title": "Blog Standard",
            "description": "Standard single-column layout for general writing.",
            "thumbnail": "bg-gradient-to-br from-blue-500 to-cyan-500",
            "tags": ["General", "Writing"]
        },
        {
            "renderer_id": "math_v3",
            "title": "Math Manuscript V3",
            "description": "Latex-heavy two-column layout for mathematical proofs.",
            "thumbnail": "bg-gradient-to-br from-indigo-500 to-purple-500",
            "tags": ["Math", "Academic"]
        },
        {
            "renderer_id": "vrkb",
            "title": "Vulnerability Research",
            "description": "Kanban-style board for tracking finding and assets.",
            "thumbnail": "bg-gradient-to-br from-orange-500 to-red-500",
            "tags": ["Security", "Workflow"]
        },
        {
            "renderer_id": "memo",
            "title": "Memo Board",
            "description": "Grid layout for quick notes and thoughts.",
            "thumbnail": "bg-gradient-to-br from-yellow-400 to-orange-400",
            "tags": ["Personal", "Notes"]
        },
        {
            "renderer_id": "admin_system",
            "title": "System Control",
            "description": "Protected interface for system administration.",
            "thumbnail": "bg-gradient-to-br from-gray-700 to-black",
            "tags": ["Admin", "System"]
        }
    ]

    for t in templates:
        # Check if exists by renderer_id
        cursor.execute("SELECT id FROM layout_templates WHERE renderer_id = ?", (t['renderer_id'],))
        if cursor.fetchone():
            print(f"  Skipping {t['title']} (Exists)")
            continue
            
        t_id = get_uuid_bytes()
        print(f"  Inserting {t['title']}...")
        
        # Schema: id, renderer_id, title, description, thumbnail, tags, created_at, updated_at
        cursor.execute("""
            INSERT INTO layout_templates (id, renderer_id, title, description, thumbnail, tags, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            t_id, 
            t['renderer_id'], 
            t['title'], 
            t['description'], 
            t['thumbnail'], 
            json.dumps(t['tags']), 
            now(), 
            now()
        ))

    conn.commit()
    conn.close()
    print("✅ Seed Complete.")

if __name__ == "__main__":
    seed()
