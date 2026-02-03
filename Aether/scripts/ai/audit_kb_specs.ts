import * as fs from 'fs';
import * as path from 'path';

// Config
const DASHBOARD_DIR = path.resolve(__dirname, '../../frontend/src/views/dashboard');
const IGNORE_FILES = ['.DS_Store'];

// Color codes
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const RESET = '\x1b[0m';

/**
 * Audit a single Vue file for compliance.
 */
function auditFile(filePath: string): boolean {
    const content = fs.readFileSync(filePath, 'utf-8');
    const fileName = path.basename(filePath);
    let valid = true;
    const errors: string[] = [];

    // 1. Check for Loading Logic (Skeleton)
    // We look for v-if="!isReady" or v-if="loading" or similar patterns in the template.
    // This is a naive regex check but effective for standard patterns.
    const hasLoadingCheck = /v-if=["'].*(!isReady|loading|isLoading).*["']/.test(content);

    // Also accept v-else on the content if v-if was checking loading
    const hasSkeletonComponent = /<.*Skeleton.*>/.test(content) || /class=["'].*skeleton.*["']/.test(content);

    if (!hasLoadingCheck && !hasSkeletonComponent) {
        valid = false;
        errors.push(`Missing Loading State: Template should have 'v-if="!isReady"' or render a Skeleton.`);
    }

    // 2. Check for Keep-Alive Hook
    // We look for onActivated(() => ...)
    const hasActivatedHook = /onActivated\(/.test(content);

    if (!hasActivatedHook) {
        valid = false;
        errors.push(`Missing Keep-Alive Hook: Component must import and use 'onActivated' to handle re-entry.`);
    }

    if (valid) {
        console.log(`${GREEN}PASS${RESET}: ${fileName}`);
    } else {
        console.log(`${RED}FAIL${RESET}: ${fileName}`);
        errors.forEach(e => console.log(`  - ${e}`));
    }

    return valid;
}

/**
 * Main Runner
 */
function run() {
    console.log(`${YELLOW}=== Aether KB Compliance Audit ===${RESET}`);
    console.log(`Scanning: ${DASHBOARD_DIR}\n`);

    if (!fs.existsSync(DASHBOARD_DIR)) {
        console.error(`${RED}Error: Directory not found: ${DASHBOARD_DIR}${RESET}`);
        process.exit(1);
    }

    const files = fs.readdirSync(DASHBOARD_DIR);
    let hasFailures = false;

    files.forEach(file => {
        if (IGNORE_FILES.includes(file)) return;
        if (!file.endsWith('.vue')) return;

        const filePath = path.join(DASHBOARD_DIR, file);
        if (!auditFile(filePath)) {
            hasFailures = true;
        }
    });

    console.log('\n' + '-'.repeat(30));
    if (hasFailures) {
        console.log(`${RED}AUDIT FAILED: Some components violate the Navigation Spec.${RESET}`);
        console.log(`Please read: AI/context/specs/navigation_lifecycle_spec.md`);
        process.exit(1);
    } else {
        console.log(`${GREEN}AUDIT PASSED: All KBs are compliant.${RESET}`);
    }
}

run();
