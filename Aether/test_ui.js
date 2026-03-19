const { chromium } = require('playwright');
const path = require('path');

(async () => {
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    try {
        await page.goto('http://localhost:5173');
        console.log("Navigated to frontend");

        // Wait for app to load
        await page.waitForTimeout(2000);

        // 1. Try to login (we need to trigger the UI state)
        // Check if we are on login page, fill user/pass
        const hasLogin = await page.$('input[type="text"]');
        if (hasLogin) {
            console.log("Attempting to login via UI...");
            const inputs = await page.$$('input');
            if (inputs.length >= 2) {
                await inputs[0].fill('admin'); // or whatever
                await inputs[1].fill('password');
                const btn = await page.$('button');
                if (btn) await btn.click();
                await page.waitForTimeout(2000);
            }
        }

        console.log("Looking for New Knowledge Base button...");
        // This relies on knowing the DOM, but let's just dump console errors
        page.on('console', msg => console.log('PAGE LOG:', msg.text()));
        page.on('pageerror', error => console.error('PAGE ERROR:', error.message));

        // Let's just evaluate a script to see if the axios API is reachable under window 
        // to ensure no global JS crash happened.
        const title = await page.title();
        console.log("Page title:", title);

    } catch (e) {
        console.error(e);
    }

    await browser.close();
})();
