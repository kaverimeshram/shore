// Records the judge path at 1440x900 against production: node web/scripts/demo-capture.mjs [baseURL]
import { chromium } from "@playwright/test";
import fs from "node:fs";
const BASE = process.argv[2] || "https://shore-zec.pages.dev";
const out = new URL("../../docs/pitch/", import.meta.url).pathname;
const b = await chromium.launch();
const ctx = await b.newContext({ viewport: { width: 1440, height: 900 }, recordVideo: { dir: "/tmp/shore-vid", size: { width: 1440, height: 900 } } });
const p = await ctx.newPage();
const wait = (ms) => p.waitForTimeout(ms);
const scrollTo = (sel) => p.locator(sel).evaluate((e) => e.scrollIntoView({ behavior: "smooth", block: "start" }));
// Paced to docs/pitch/demo-walkthrough.md (10 beats, about 2:52). Same path as the tour, pressed by hand.
const click = async (sel, pre = 1500) => { const l = p.locator(sel).first(); await l.scrollIntoViewIfNeeded(); await l.hover(); await wait(pre); await l.click(); };
await p.goto(BASE + "/shore/"); await wait(3000); await p.getByTestId("tour").hover(); await wait(12000);              // 1 landing
await scrollTo("#demand"); await wait(12000);                                                                     // 2 demand
await scrollTo("#business"); await wait(3000);
await p.goto(BASE + "/shore/?replay=1&speed=3"); await wait(2000);
await click('[data-testid="connect"]'); await wait(11000);                                                       // 3 connect
await click('[data-testid="fresh"]'); await wait(13000);                                                         // 4 fresh u1
await click('#s-quote .btn:not(.ghost)'); await wait(8000);                                                      // 5 quote with fees
const fb = p.locator("summary", { hasText: "Fee breakdown" }).first();
if (await fb.count()) { await fb.click(); await wait(4000); await fb.click(); }
await wait(6000);
await click('[data-testid="approve"]', 3000); await wait(9000);                                                  // 6 one approval
await p.evaluate(() => window.scrollTo({ top: 0, behavior: "smooth" }));                                         // 7 crossing tide band
await p.locator("#readback[data-landed]").waitFor({ timeout: 120000 });
await scrollTo("#readback"); await wait(16000);                                                                  // 8 read-back
await p.locator("#public summary").click(); await scrollTo("#public"); await wait(17000);                        // 9 what's still public
await p.goto(BASE + "/shore/?invite=zechub&amount=0.005&note=Welcome%20to%20shielded%20ZEC"); await wait(14000); // 10 invite link
const v = p.video(); await ctx.close(); await b.close();
fs.copyFileSync(await v.path(), out + "demo-capture.webm");
console.log("saved", out + "demo-capture.webm");
