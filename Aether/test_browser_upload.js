const { chromium } = require('playwright');
const path = require('path');

(async () => {
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    
    page.on('console', msg => {
        if (msg.type() === 'error' || msg.type() === 'warning' || msg.text().includes('[KnowledgeModule]')) {
            console.log(`[Browser ${msg.type().toUpperCase()}] ${msg.text()}`);
        }
    });

    try {
        await page.goto('http://localhost:5173');
        await page.waitForTimeout(2000); // Let it load
        
        console.log("Evaluating browser state...");
        const result = await page.evaluate(async () => {
            // Check if Axios and the auth token are accessible in window
            const token = localStorage.getItem('token');
            if (!token) return { error: "No token in localStorage. The user needs to log in on this browser profile." };
            return { token: "Token exists", length: token.length };
        });
        
        console.log("Browser State:", result);

        // Dump HTML structure to figure out where we are
        const html = await page.content();
        console.log("Found HTML markers:", {
             hasApp: html.includes('id="app"'),
             hasSignIn: html.includes('Sign In') || html.includes('Login'),
             hasImport: html.includes('Import') || html.includes('import-akb')
        });

    } catch(e) {
        console.error("Test execution failed:", e);
    }
    
    await browser.close();
})();
