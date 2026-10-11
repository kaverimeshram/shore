// node scripts/v2shots.mjs <base> <outdir>: landing + read-back state at 1440 and 390.
import { chromium } from "@playwright/test";
const [base, dir] = process.argv.slice(2);
const b = await chromium.launch();
for (const [w, h] of [[1440, 900], [390, 844]]) {
  const p = await (await b.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 1 })).newPage();
  await p.goto(base + "/shore/"); await p.waitForTimeout(1200);
  await p.screenshot({ path: `${dir}/landing-${w}.png` });
  await p.screenshot({ path: `${dir}/landing-${w}-full.png`, fullPage: true });
  await p.goto(base + "/shore/?replay=1&speed=30");
  await p.getByTestId("connect").click(); await p.getByTestId("fresh").click();
  await p.getByRole("button", { name: "Get quote" }).click(); await p.getByTestId("quote").waitFor();
  await p.screenshot({ path: `${dir}/quote-${w}.png` });
  await p.getByTestId("approve").click(); await p.waitForTimeout(1500);
  await p.screenshot({ path: `${dir}/crossing-${w}.png` });
  await p.locator("#readback[data-landed]").waitFor({ timeout: 60000 }); await p.waitForTimeout(1500);
  await p.screenshot({ path: `${dir}/readback-${w}.png` });
  await p.screenshot({ path: `${dir}/readback-${w}-full.png`, fullPage: true });
}
await b.close(); console.log("ok", dir);
