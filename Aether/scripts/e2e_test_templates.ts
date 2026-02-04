async function testTemplates() {
    const url = 'http://localhost:3000/api/templates';
    console.log(`Testing GET ${url}...`);

    try {
        const res = await fetch(url);
        if (!res.ok) {
            throw new Error(`Status ${res.status}: ${res.statusText}`);
        }

        const data = await res.json();
        console.log('Response:', JSON.stringify(data, null, 2));

        if (!Array.isArray(data)) {
            throw new Error('Response is not an array');
        }

        if (data.length === 0) {
            throw new Error('Template list is empty (Seeding failed?)');
        }

        const expected = ['default', 'math_v3', 'vrkb', 'memo', 'admin_system'];
        const found = data.map((t: any) => t.renderer_id);

        // Check if at least some expected templates are present
        const missing = expected.filter(id => !found.includes(id));

        if (missing.length > 0) {
            console.warn('⚠️ Missing standard templates:', missing);
            // We strictly require at least 'default' and 'math_v3'
            if (missing.includes('default') || missing.includes('math_v3')) {
                throw new Error('Critical templates missing');
            }
        }

        console.log('✅ Templates Verification Passed.');
    } catch (error) {
        console.error('❌ Test Failed:', error);
        process.exit(1);
    }
}

testTemplates();
