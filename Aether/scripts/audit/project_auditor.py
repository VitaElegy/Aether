#!/usr/bin/env python3
import os
import re
import sys

# Configuration
REQUIRED_DIRS = [
    "AI/const",
    "AI/skills",
    "AI/context/specs",
    "AI/memory",
    "doc/architecture",
    "scripts/audit",
    "scripts/scaffold",
    ".agent/workflows"
]

FORBIDDEN_PATTERNS = [
    {
        "file_pattern": r".*\.vue$",
        "regex": r"import.*from ['\"]axios['\"]",
        "message": "CRITICAL: Direct axios import in Vue component. Use Composables (useContent, useAuth)."
    },
    {
        "file_pattern": r".*\.rs$",
        "regex": r"\bpanic!\(",
        "message": "CRITICAL: Explicit panic! found in Rust code. Use Result<T, AppError>."
    }
]

def check_structure():
    print(f"🔍 Auditing Directory Structure...")
    missing = []
    for d in REQUIRED_DIRS:
        if not os.path.exists(d):
            missing.append(d)
    
    if missing:
        print(f"❌ Missing Directories: {missing}")
        return False
    print(f"✅ Directory Structure OK")
    return True

def scan_files():
    print(f"🔍 Scanning Code Constraints...")
    violations = 0
    
    for root, _, files in os.walk("."):
        if "node_modules" in root or "target" in root:
            continue
            
        for file in files:
            path = os.path.join(root, file)
            for rule in FORBIDDEN_PATTERNS:
                if re.match(rule["file_pattern"], file):
                    try:
                        with open(path, "r", encoding="utf-8") as f:
                            content = f.read()
                            if re.search(rule["regex"], content):
                                print(f"❌ Violation in {path}: {rule['message']}")
                                violations += 1
                    except Exception:
                        pass # Binary files etc
    
    if violations == 0:
        print(f"✅ Constraints OK (No axios in Vue, No panic in Rust)")
        return True
    return False

if __name__ == "__main__":
    print("👮 Aether Project Auditor v1.0")
    print("-----------------------------")
    struct_ok = check_structure()
    code_ok = scan_files()
    
    if struct_ok and code_ok:
        print("\n✅ AUDIT PASSED. System is compliant.")
        sys.exit(0)
    else:
        print("\n❌ AUDIT FAILED. Please fix violations.")
        sys.exit(1)
