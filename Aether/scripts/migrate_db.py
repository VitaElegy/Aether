import sqlite3
import os

DB_PATH = 'backend/aether.db'
MIGRATION_FILE = 'backend/migrations/001_create_blocks.sql'

def migrate():
    if not os.path.exists(DB_PATH):
        print(f"Error: Database not found at {DB_PATH}")
        return

    if not os.path.exists(MIGRATION_FILE):
        print(f"Error: Migration file not found at {MIGRATION_FILE}")
        return

    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        with open(MIGRATION_FILE, 'r') as f:
            sql_script = f.read()
        
        cursor.executescript(sql_script)
        conn.commit()
        print("✅ Migration executed successfully: created 'blocks' table.")
        
        # Verify
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='blocks';")
        if cursor.fetchone():
            print("Verified: Table 'blocks' exists.")
        else:
            print("Error: Table 'blocks' verification failed.")

        conn.close()

    except Exception as e:
        print(f"Migration Failed: {e}")

if __name__ == "__main__":
    migrate()
