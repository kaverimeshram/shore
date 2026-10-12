import { test, expect } from "@playwright/test";
import { probeOverlap } from "./lib/overlap.js";
// Scrolls in steps at 1440, 768 and 390 and checks no text collides with sticky layers or other text.
for (const [w, h] of [[1440, 900], [768, 1024], [390, 844]]) {
  test(`no overlapping text while scrolling at ${w}px`, async ({ page }, info) => {
    test.skip(info.project.name !== "desktop", "viewport set per test");
    await page.setViewportSize({ width: w, height: h });
    for (const url of ["/shore/", "/shore/?replay=1&tour=1&speed=40"]) {
      await page.goto(url); await page.waitForTimeout(url.includes("tour") ? 9000 : 1200);
      const total = await page.evaluate(() => document.documentElement.scrollHeight);
      const problems = [];
      for (let y = 0; y < total; y += Math.round(h * 0.6)) {
        await page.evaluate((y) => window.scrollTo(0, y), y); await page.waitForTimeout(120);
        for (const p of await page.evaluate(probeOverlap)) problems.push(`${url} y=${y}: ${p}`);
      }
      expect(problems, problems.join("\n")).toEqual([]);
    }
  });
}
