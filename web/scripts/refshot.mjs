// Playwright stand-in for ui-craft refshot.py (its bhn deepsurge capture hangs on this Mac).
// Same contract: wait for load, network quiet, fonts.ready and a settle delay; refuse error, challenge
// and blank pages (exit 2); write <png>.tokens.json with computed type/colour/radius tokens.
// usage: node scripts/refshot.mjs URL OUT.png WIDTH HEIGHT [--full]
import { chromium } from "@playwright/test";
const [url, out, w, h, full] = process.argv.slice(2);
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: +w, height: +h }, deviceScaleFactor: 1, userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0 Safari/537.36" });
try {
  await p.goto(url, { waitUntil: "load", timeout: 45000 });
  await p.waitForLoadState("networkidle", { timeout: 15000 }).catch(() => {});
  await p.evaluate(() => document.fonts.ready);
  await p.waitForTimeout(1500);
  const text = (await p.evaluate(() => document.body.innerText || "")).toLowerCase();
  if (/can.t be reached|just a moment|dns_probe|access denied|attention required|verify you are human/.test(text) || text.trim().length < 40) { console.error("refused: error/challenge/blank"); process.exit(2); }
  await p.screenshot({ path: out, fullPage: full === "--full" });
  const tokens = await p.evaluate(() => {
    const pick = (sel) => { const e = document.querySelector(sel); if (!e) return null; const c = getComputedStyle(e);
      return { text: (e.innerText || "").trim().slice(0, 60), font: c.fontFamily, size: c.fontSize, weight: c.fontWeight, tracking: c.letterSpacing, leading: c.lineHeight, color: c.color, background: c.backgroundColor, radius: c.borderRadius }; };
    return { url: location.href, body: pick("body"), h1: pick("h1"), h2: pick("h2"), h3: pick("h3"), p: pick("p"), a: pick("a"), button: pick("button"), input: pick("input"),
      fonts: [...new Set([...document.fonts].filter((f) => f.status === "loaded").map((f) => `${f.family} ${f.weight}`))] };
  });
  (await import("node:fs")).writeFileSync(out.replace(/\.png$/, ".tokens.json"), JSON.stringify(tokens, null, 1));
  console.log("ok", out);
} catch (e) { console.error("refused:", e.message); process.exit(2); } finally { await b.close(); }
