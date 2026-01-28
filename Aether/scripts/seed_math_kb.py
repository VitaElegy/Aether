import sqlite3
import uuid
import json
import datetime

DB_PATH = 'backend/aether.db'

def get_uuid_bytes():
    return uuid.uuid4().bytes

def to_bytes(uuid_str):
    return uuid.UUID(uuid_str).bytes

def now():
    return datetime.datetime.now().isoformat()

def seed():
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    # Cleanup old text-based entries (if any)
    # We try to delete by name/title to clean up previous runs
    print("Cleaning up previous seed data...")
    cursor.execute("DELETE FROM knowledge_bases WHERE title = 'Math Demo KB'")
    # Nodes will cascade delete if FK logic works, checking
    cursor.execute("DELETE FROM nodes WHERE title = 'Set Theory Basics'")
    cursor.execute("DELETE FROM users WHERE username = 'math_user'")

    # 1. User
    user_id = get_uuid_bytes()
    # Check if we have users, if not insert one
    cursor.execute("SELECT id FROM users LIMIT 1")
    row = cursor.fetchone()
    if row:
        user_id = row[0] # This will be bytes
    else:
        print("Creating User...")
        cursor.execute("INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)", 
                       (user_id, 'math_user', 'math@example.com', 'hash', now(), now()))

    # 2. Knowledge Base
    kb_uuid = uuid.uuid4()
    kb_id = kb_uuid.bytes
    print(f"Creating KB: {str(kb_uuid)}")
    
    # author_id, title, description, tags, cover_offset_y, visibility, created_at, updated_at
    cursor.execute("""
        INSERT INTO knowledge_bases (id, author_id, title, description, tags, cover_offset_y, visibility, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, (kb_id, user_id, 'Math Demo KB', 'V2 Architecture Demo', '[]', 0, 'Public', now(), now()))

    # 3. Node (Document)
    doc_id = get_uuid_bytes()
    print(f"Creating Document")
    # author_id, knowledge_base_id, type, title, permission_mode, created_at, updated_at
    cursor.execute("""
        INSERT INTO nodes (id, author_id, knowledge_base_id, type, title, permission_mode, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    """, (doc_id, user_id, kb_id, 'article', 'Set Theory Basics', 'Public', now(), now()))

    # 4. Blocks
    
    # Block 1: Axiom
    axiom_id = get_uuid_bytes()
    axiom_payload = json.dumps({
        "label": "ZFC-1",
        "content": "Extensionality: $\\forall x \\forall y (\\forall z (z \\in x \\iff z \\in y) \\implies x = y)$"
    })
    
    # Block 2: Definition
    def_id = get_uuid_bytes()
    def_payload = json.dumps({
        "term": "Subset",
        "content": "A set $A$ is a subset of $B$ ($A \\subseteq B$) if every element of $A$ is in $B$."
    })
    
    # Block 3: Theorem
    thm_uuid = uuid.uuid4()
    thm_id = thm_uuid.bytes
    thm_payload = json.dumps({
        "label": "Theorem 1.1",
        "content": "Every set is a subset of itself."
    })
    
    # Block 4: Proof
    proof_id = get_uuid_bytes()
    # Need string UUID for JSON payload references? 
    # Yes, JSON payload should use STRING UUIDs because frontend/API expects strings in JSON.
    # The Backend `extract_references` parses strings.
    # But the REFERENCE column (if any) is not used here.
    # Wait, `extract_references` parses explicit `theorem_id` from JSON.
    # So `thm_id` in JSON must be the STRING representation of the UUID we inserted as BYTES.
    
    proof_payload = json.dumps({
        "theorem_id": str(thm_uuid),
        "steps": "1. Let $x$ be an arbitrary element of $A$. \n2. Then trivially $x \\in A$. \n3. Therefore, $A \\subseteq A$.",
        "qcd_symbol": "Q.E.D."
    })

    print("Inserting Blocks...")
    blocks = [
        (axiom_id, doc_id, 'axiom', 0, 1, axiom_payload, now(), now()),
        (def_id, doc_id, 'definition', 1, 1, def_payload, now(), now()),
        (thm_id, doc_id, 'theorem', 2, 1, thm_payload, now(), now()),
        (proof_id, doc_id, 'proof', 3, 1, proof_payload, now(), now())
    ]

    cursor.executemany("""
        INSERT INTO blocks (id, document_id, type, ordinal, revision, payload, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    """, blocks)

    conn.commit()
    conn.close()
    
    print("✅ Seed Complete (Binary Mode).")
    print(f"KB ID for Curl (String): {str(kb_uuid)}")

if __name__ == "__main__":
    seed()
