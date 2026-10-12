// node scripts/overlap-shots.mjs <base> <dir>: full-page + scroll-step shots at 1440/768/390.
import { chromium } from "@playwright/test";
const [base, dir] = process.argv.slice(2);
const b = await chromium.launch();
for (const [w, h] of [[1440, 900], [768, 1024], [390, 844]]) {
  const p = await (await b.newContext({ viewport: { width: w, height: h } })).newPage();
  await p.goto(base + "/shore/"); await p.waitForTimeout(1200);
  await p.screenshot({ path: `${dir}/full-${w}.png`, fullPage: true });
  const total = await p.evaluate(() => document.documentElement.scrollHeight);
  for (let y = 0, i = 0; y < total && i < 8; y += Math.round(h * 0.6), i++) { await p.evaluate((y) => scrollTo(0, y), y); await p.waitForTimeout(150); await p.screenshot({ path: `${dir}/scroll-${w}-${i}.png` }); }
}
await b.close(); console.log("ok");
