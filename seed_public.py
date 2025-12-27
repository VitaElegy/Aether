import urllib.request
import json
import time

BASE_URL = "http://localhost:3000/api"

def request(method, endpoint, data=None, headers={}):
    url = f"{BASE_URL}{endpoint}"
    req = urllib.request.Request(url, method=method)
    for k, v in headers.items():
        req.add_header(k, v)

    if data:
        body = json.dumps(data).encode('utf-8')
        req.add_header('Content-Type', 'application/json')
        req.data = body

    try:
        with urllib.request.urlopen(req) as res:
            return json.load(res)
    except urllib.error.HTTPError as e:
        print(f"HTTP Error {e.code}: {e.read().decode()}")
        return None
    except Exception as e:
        print(f"Error: {e}")
        return None

# 1. Register/Login
username = f"guest_hero_{int(time.time())}"
print(f"Creating user {username}...")
auth = request("POST", "/auth/register", {
    "username": username,
    "email": f"{username}@example.com",
    "password": "password123"
})

if not auth and "token" not in (auth or {}):
    # Try login if exists
    print("Login...")
    auth = request("POST", "/auth/login", {
        "username": username,
        "password": "password123"
    })

if not auth or "token" not in auth:
    print("Failed to authenticate")
    exit(1)

token = auth["token"]
print("Authenticated.")

# 2. Create Public Article
print("Creating public article...")
article = request("POST", "/content", {
    "title": "Welcome to Aether (Public)",
    "slug": f"welcome-public-{int(time.time())}",
    "status": "Published",
    "visibility": "Public",
    "category": "Announcements",
    "body": "This is a public article visible to guests.",
    "tags": ["welcome", "public"],
    "reason": "Initial seed"
}, {"Authorization": f"Bearer {token}"})

if article:
    print(f"Article created: {article.get('id')}")
else:
    print("Failed to create article")

# 3. Check Feed (Guest)
print("Checking guest feed...")
feed = request("GET", "/content")
if feed:
    print(f"Feed count: {len(feed)}")
    # Check if our article is there
    found = any(a['title'] == "Welcome to Aether (Public)" for a in feed)
    print(f"Found our article: {found}")
else:
    print("Feed failed")
