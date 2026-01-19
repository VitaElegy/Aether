import requests
import json
import sys

BASE_URL = "http://localhost:3000" # Assuming default port 3000, checking main.rs next to be sure

def register_and_login():
    # randomized user
    import random
    import string
    username = "test_user_" + ''.join(random.choices(string.ascii_lowercase, k=6))
    password = "password123"
    
    print(f"Registering {username}...")
    try:
        res = requests.post(f"{BASE_URL}/api/auth/register", json={
            "username": username,
            "password": password,
            "email": f"{username}@example.com"
        })
        if res.status_code == 200:
             print("Register success")
        else:
             print(f"Register failed: {res.status_code} {res.text}")
             # Try login if already exists (unlikely with random)
    except Exception as e:
        print(f"Connection failed: {e}")
        return None

    print(f"Logging in {username}...")
    res = requests.post(f"{BASE_URL}/api/auth/login", json={
        "username": username,
        "password": password
    })
    
    if res.status_code == 200:
        token = res.json().get("token")
        print("Login success, token acquired.")
        return token
    else:
        print(f"Login failed: {res.status_code} {res.text}")
        return None

def test_create_english_article(token):
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "title": "Test English Article " + str(json.dumps({"complex": "title"})), # Test safe title
        "body": json.dumps({
            "text": "This is a test sentence. This is another one.",
            "background": "http://example.com/bg.png"
        }),
        "category": "English Analysis",
        "tags": ["test", "english"],
        "status": "Draft",
        "visibility": "Private"
    }
    
    print("Creating English Analysis article...")
    res = requests.post(f"{BASE_URL}/api/content", json=payload, headers=headers)
    if res.status_code == 200:
        print("Create success.")
        return res.json()
    else:
        print(f"Create failed: {res.status_code} {res.text}")
        return None

def test_list_articles(token):
    headers = {"Authorization": f"Bearer {token}"}
    print("Listing articles...")
    res = requests.get(f"{BASE_URL}/api/content?category=English%20Analysis", headers=headers)
    if res.status_code == 200:
        print("List success.")
        items = res.json()
        print(f"Found {len(items)} articles.")
    else:
        print(f"List failed: {res.status_code} {res.text}")

def main():
    token = register_and_login()
    if token:
        article_id = test_create_english_article(token)
        test_list_articles(token)

if __name__ == "__main__":
    main()
