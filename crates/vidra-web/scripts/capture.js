import { chromium } from "playwright";
import readline from "readline";

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
});

let browser = null;
let page = null;
let mode = "realtime";

async function startSession(config) {
    mode = config.mode; // "realtime" or "frame-accurate"
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
        viewport: { width: config.viewport_width, height: config.viewport_height },
    });
    page = await context.newPage();

    if (mode === "frame-accurate") {
        await page.addInitScript(() => {
            window.__vidra = {
                capturing: true,
                frame: 0,
                time: 0,
                fps: window.__vidra?.fps || 60,
                vars: {},
                _callbacks: [],
                emit: (event, data) => {}, // Stub
                requestAdvance: (cb) => { window.__vidra._callbacks.push(cb); }
            };

            const originalDateNow = Date.now;
            const originalPerfNow = performance.now.bind(performance);
            const originalRequestAnimationFrame = window.requestAnimationFrame;

            let virtualTimeMs = 0;

            Date.now = () => virtualTimeMs;
            performance.now = () => virtualTimeMs;

            let rafCallbacks = [];
            window.requestAnimationFrame = (cb) => {
                rafCallbacks.push(cb);
                return Math.random();
            };

            window.__vidra_advance_frame = (timeSeconds, vars) => {
                window.__vidra.time = timeSeconds;
                window.__vidra.vars = vars;
                virtualTimeMs = timeSeconds * 1000;
                
                const callbacks = rafCallbacks;
                rafCallbacks = [];
                callbacks.forEach(cb => {
                    try { cb(virtualTimeMs); } catch(e) { console.error(e); }
                });
            };
        });
    } else {
        await page.addInitScript(() => {
            window.__vidra = {
                capturing: true,
                frame: 0,
                time: 0,
                fps: window.__vidra?.fps || 60,
                vars: {},
                emit: () => {}
            };
        });
    }

    await page.goto(config.source, { waitUntil: "networkidle" });
    
    console.log(JSON.stringify({ type: "ready" }));
}

rl.on("line", async (line) => {
    try {
        const msg = JSON.parse(line);
        if (msg.type === "start") {
            await startSession(msg.config);
        } else if (msg.type === "capture") {
            if (!page) {
                console.log(JSON.stringify({ type: "error", error: "Not started" }));
                return;
            }

            if (mode === "frame-accurate") {
                await page.evaluate(({time, vars}) => {
                    window.__vidra_advance_frame(time, vars);
                }, msg);
                // Simple wait for layout to settle
                await page.waitForTimeout(5);
            } else {
                await page.evaluate((msg) => {
                    window.__vidra.time = msg.time;
                    window.__vidra.vars = msg.vars;
                }, msg);
            }

            const buffer = await page.screenshot({ type: "jpeg", quality: 90 });
            console.log(JSON.stringify({ type: "frame", data: buffer.toString("base64") }));
        } else if (msg.type === "stop") {
            if (browser) await browser.close();
            console.log(JSON.stringify({ type: "stopped" }));
            process.exit(0);
        }
    } catch (e) {
        console.error(JSON.stringify({ type: "error", error: e.toString() }));
    }
});
