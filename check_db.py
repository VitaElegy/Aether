import sqlite3
import os

try:
    conn = sqlite3.connect('Aether/backend/aether.db')
    cursor = conn.cursor()
    cursor.execute("SELECT id, title, status, visibility, author_id FROM contents")
    rows = cursor.fetchall()
    print(f"Total rows: {len(rows)}")
    for row in rows:
        print(row)
    conn.close()
except Exception as e:
    print(e)

