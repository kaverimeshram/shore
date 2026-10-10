// Round shots for the critic: node scripts/rshots.mjs <round> [baseURL]
import { chromium } from "@playwright/test";
const [round, base = "http://127.0.0.1:8799"] = process.argv.slice(2);
const D = "../docs/design";
const b = await chromium.launch();
for (const [w, hgt] of [[1440, 900], [390, 844]]) {
  const p = await b.newPage({ viewport: { width: w, height: hgt } });
  await p.goto(`${base}/shore/?replay=1&speed=3`); await p.evaluate(() => document.fonts.ready); await p.waitForTimeout(800);
  await p.screenshot({ path: `${D}/r${round}-${w}.png` });
  await p.getByTestId("connect").click(); await p.getByTestId("fresh").click();
  await p.getByRole("button", { name: "Get quote" }).click(); await p.locator(".qt").waitFor(); await p.waitForTimeout(300);
  await p.locator("#s-quote").scrollIntoViewIfNeeded();
  await p.screenshot({ path: `${D}/r${round}-quote-${w}.png` });
  await p.getByTestId("approve").click(); await p.waitForTimeout(9000);
  await p.screenshot({ path: `${D}/r${round}-crossing-${w}.png` });
  await p.getByTestId("rb-amount").waitFor({ state: "attached", timeout: 60000 }); await p.locator("#readback[data-landed]").waitFor(); await p.waitForTimeout(1200);
  await p.locator("#readback").scrollIntoViewIfNeeded();
  await p.screenshot({ path: `${D}/r${round}-readback-${w}.png` });
  await p.screenshot({ path: `${D}/r${round}-full-${w}.png`, fullPage: true });
  console.log(w, "height", await p.evaluate(() => document.documentElement.scrollHeight));
}
await b.close();
