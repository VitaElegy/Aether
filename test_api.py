import urllib.request
import json

url = "http://localhost:3000/api/content"

try:
    req = urllib.request.Request(url)
    with urllib.request.urlopen(req) as response:
        status = response.getcode()
        data = response.read()
        print(f"Status: {status}")
        try:
            json_data = json.loads(data)
            print(f"Data: {json.dumps(json_data, indent=2)}")
        except json.JSONDecodeError:
            print(f"Raw Body: {data.decode('utf-8')}")
except Exception as e:
    print(f"Error: {e}")
