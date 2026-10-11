import { test, expect } from "@playwright/test";

// Fails the test if the page ever touches a wallet or a write endpoint in replay.
const OWN = "u1qrdwfnpakf2xhw9rpsuvsxmnk6qkz2cm7xmk4m2r2hf7ug4fhz9kjv5x9tmuqhx6wysmhhqw3edpa0yepl4lhjkvd7fys09rrhj7ud4sfu9dg";

async function guard(page) {
  const writes = [];
  page.on("request", (r) => { if (r.method() !== "GET" && !r.url().startsWith(new URL(page.url() === "about:blank" ? "http://127.0.0.1:4174" : page.url()).origin)) writes.push(r.url()); });
  await page.addInitScript(() => {
    const trap = { isPhantom: true, connect() { throw new Error("wallet touched"); }, signAndSendTransaction() { throw new Error("wallet touched"); } };
    window.phantom = { solana: trap }; window.solana = trap;
  });
  return writes;
}

test("replay plays the recorded mainnet run end to end", async ({ page }) => {
  const writes = await guard(page);
  await page.goto("/shore/?replay=1&speed=20");
  await expect(page.locator("#mode")).toContainText("Replay");
  await page.getByTestId("connect").click();
  await expect(page.getByTestId("balance")).toContainText("0.0014");
  await page.getByTestId("fresh").click();
  await expect(page.getByTestId("recipient")).toContainText("u1uufrl3tucq9");
  await page.getByRole("button", { name: "Get quote" }).click();
  const q = page.getByTestId("quote");
  await expect(q).toContainText("0.00108 ZEC");
  await expect(q).toContainText("0.00032 ZEC");      // withdraw fee 32000 zat
  await expect(q).toContainText("22.9% of what you send");
  await expect(page.getByTestId("fee-honest")).toBeVisible();
  await page.getByTestId("approve").click();
  await expect(page.getByTestId("rb-amount")).toContainText("0.00115", { timeout: 30_000 });
  const rb = page.locator("#readback");
  await expect(rb).toContainText("3,512,504");
  await expect(rb.locator('a[href*="fa8d4d6bd210c6150e98142ec1ae9a25706ec19bd591a775823f6616f95fa408"]')).toHaveCount(1);
  await expect(rb.locator('a[href*="solscan.io/tx/2LLVDBnV"]')).toHaveCount(1);
  await expect(rb).toContainText("7000 zat more");
  const pub = page.locator("#public");
  await expect(pub).toContainText("Recipient u1 via 1Click status");
  await expect(pub).toContainText("frozen");
  expect(writes).toEqual([]);
});

test("refuses transparent, Sapling and reused recipients", async ({ page }) => {
  await guard(page);
  const u1 = "u1uufrl3tucq9vjgvssws978y0nm6d6cncqk75qs62htzctwytgpqqsdwlpe5jvl83xqyn2vtkptk4cw535n575yf3nrwpcrydpgrkpgmk";
  await page.addInitScript((a) => localStorage.setItem("shore.usedRecipients", JSON.stringify([a])), u1);
  await page.goto("/shore/?mock=1");
  await page.getByTestId("connect").click();
  const own = page.locator("#own"), use = page.getByRole("button", { name: "Use this address" }), err = page.locator("#s-address .err");
  await own.fill("t1Kx9d7bQjzS6dZ8WxwB4bS9mV2fH3bK9nP"); await use.click();
  await expect(err).toContainText("transparent");
  await own.fill("zs1" + "q".repeat(75)); await use.click();
  await expect(err).toContainText("Sapling");
  await own.fill(u1); await use.click();
  await expect(err).toContainText("used for an earlier shield");
});

test("what's still public is stated before anything is signed", async ({ page }) => {
  await page.goto("/shore/");
  const pub = page.locator("#public");
  await expect(pub).toContainText("The hop in is public");
  await expect(pub).toContainText("1Click status links your Phantom address to your u1");
  await expect(pub).toContainText("NEAR can decrypt the payout");
  await expect(pub).toContainText("frozen");
});

test("live mode keeps approve locked until mainnet is acknowledged", async ({ page }) => {
  await page.route("**/api/**", (r) => {
    const u = r.request().url();
    if (u.includes("/shield/address")) return r.fulfill({ json: { id: "t1", recipient: "u1" + "a".repeat(110), receivers: ["orchard"], source: "fresh", uaIndex: 9, readbackAvailable: true } });
    if (u.includes("/quote")) return r.fulfill({ json: { id: "t1", depositAddress: "DMDdCzumrZBaPPjKBP33o1RgDhW2F2BZPzyYZEk9SjbM", mint: "A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS", decimals: 8, amountIn: "140000", amountOut: "108000", minAmountOut: "106920", fees: { withdrawFee: "32000", refundFee: "844", totalFeeBps: 2286 }, minAmount: "133334", deadline: "2026-10-13T06:47:48.000Z", timeEstimateSecs: 135, recipient: "u1" + "a".repeat(110) } });
    return r.fulfill({ status: 404, json: { error: "x" } });
  });
  await page.route("**/api/rpc", (r) => r.fulfill({ json: { jsonrpc: "2.0", id: 1, result: { value: [{ account: { data: { parsed: { info: { tokenAmount: { amount: "500000" } } } } } }] } } }));
  const signed = [];
  await page.exposeFunction("__signed", () => signed.push(1));
  await page.addInitScript(() => {
    const p = { isPhantom: true, connect: async () => ({ publicKey: { toString: () => "ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5" } }), signAndSendTransaction: async () => { window.__signed(); throw new Error("blocked in test"); } };
    window.phantom = { solana: p };
  });
  await page.goto("/shore/?live=1");
  await page.getByTestId("connect").click();
  await expect(page.getByTestId("balance")).toContainText("0.005");
  await page.locator("#own").fill(OWN); await page.getByRole("button", { name: "Use this address" }).click();
  await page.locator("#amt").fill("0.0014");
  await page.getByRole("button", { name: "Get quote" }).click();
  await expect(page.getByTestId("approve")).toBeDisabled();
  await page.locator("#ack").check();
  await expect(page.getByTestId("approve")).toBeEnabled();
  expect(signed).toEqual([]);
});

test("a judge lands on the replay by default, live is one link away", async ({ page }) => {
  const writes = await guard(page);
  await page.goto("/shore/");
  await expect(page.locator("#mode")).toContainText("Replay");
  await expect(page.getByRole("link", { name: /live/i })).toHaveAttribute("href", "?live=1");
  expect(writes).toEqual([]);
});

// Real SHORE service (local or deployed), opt-in: SHORE_LIVE_API=1. Fresh u1 and a dry quote from the
// real API; the balance lookup goes to the real /api/rpc and must answer, then the page is handed a
// fixed 0.005 ZEC so the test never depends on a funded wallet. No deposit address is ever created.
test("live flow against the real API (dry quote only)", async ({ page }) => {
  test.skip(!process.env.SHORE_LIVE_API, "set SHORE_LIVE_API=1 to hit the real service");
  const rpc = [];
  await page.route("**/api/rpc", async (r) => {
    const real = await (await r.fetch()).json();
    rpc.push(real);
    await r.fulfill({ json: { jsonrpc: "2.0", id: 1, result: { value: [{ account: { data: { parsed: { info: { tokenAmount: { amount: "500000" } } } } } }] } } });
  });
  await page.route("**/api/quote", (r) => r.continue({ postData: JSON.stringify({ ...JSON.parse(r.request().postData()), dry: true }) }));
  await page.addInitScript(() => {
    const p = { isPhantom: true, connect: async () => ({ publicKey: { toString: () => "ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5" } }), signAndSendTransaction: async () => { throw new Error("never signs in tests"); } };
    window.phantom = { solana: p };
  });
  await page.goto("/shore/?live=1");
  await page.getByTestId("connect").click();
  await expect(page.getByTestId("balance")).toContainText("ZEC", { timeout: 20_000 });
  await page.locator("#own").fill(process.env.SHORE_TEST_U1 || OWN); await page.getByRole("button", { name: "Use this address" }).click();
  await expect(page.getByTestId("recipient")).toContainText("u1", { timeout: 20_000 });
  await page.locator("#amt").fill("0.0014");
  await page.getByRole("button", { name: "Get quote" }).click();
  await expect(page.getByTestId("quote")).toContainText("0.00032 ZEC", { timeout: 30_000 });
  await expect(page.getByTestId("approve")).toBeDisabled();
  expect(rpc.length).toBe(1);
  expect(Array.isArray(rpc[0].result?.value), JSON.stringify(rpc[0]).slice(0, 200)).toBe(true);
});

test("after signing, no Change link is offered", async ({ page }) => {
  await guard(page);
  await page.goto("/shore/?replay=1&speed=20");
  await page.getByTestId("connect").click();
  await page.getByTestId("fresh").click();
  await page.getByRole("button", { name: "Get quote" }).click();
  await expect(page.getByRole("button", { name: /^Change/ })).toHaveCount(2);
  await page.getByTestId("approve").click();
  await expect(page.locator("#s-approve")).toHaveAttribute("data-state", "done");
  for (const b of await page.getByRole("button", { name: /^Change/ }).all()) await expect(b).toBeHidden();
});

test("live mode never offers the demo wallet as a destination", async ({ page }) => {
  await page.route("**/api/rpc", (r) => r.fulfill({ json: { jsonrpc: "2.0", id: 1, result: { value: [] } } }));
  await page.addInitScript(() => { window.phantom = { solana: { isPhantom: true, connect: async () => ({ publicKey: { toString: () => "ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5" } }) } }; });
  await page.goto("/shore/?live=1");
  await page.getByTestId("connect").click();
  await expect(page.locator("#own")).toBeVisible();
  await expect(page.getByTestId("fresh")).toHaveCount(0);
});

test("invite link opens the live flow pre-filled, note shown as plain text, nothing sent", async ({ page }) => {
  const writes = await guard(page);
  const note = '<img src=x onerror="window.__xss=1">Welcome to Zcash, from the ZecHub bounty';
  await page.goto("/shore/?invite=zechub-42&amount=0.002&note=" + encodeURIComponent(note));
  await expect(page.locator("#mode")).toContainText("Live");
  await expect(page.locator("#invite")).toContainText("Invite zechub-42");
  await expect(page.getByTestId("invite-note")).toHaveText(note);
  await expect(page.getByTestId("invite-amount")).toHaveText("0.002 ZEC");
  await expect(page.locator("#invite img")).toHaveCount(0);
  expect(await page.evaluate(() => window.__xss)).toBeUndefined();
  await expect(page.locator("#invite")).toContainText("SHORE did not write this note");
  expect(writes).toEqual([]);
});

test("invite amount is clamped to the 1Click minimum and the launch cap; junk is ignored", async ({ page }) => {
  await page.goto("/shore/?invite=x&amount=5");
  await expect(page.getByTestId("invite-amount")).toHaveText("0.02 ZEC");
  await expect(page.getByTestId("invite-amount-note")).toContainText("lowered to 0.02");
  await page.goto("/shore/?invite=x&amount=0.0001");
  await expect(page.getByTestId("invite-amount")).toHaveText("0.00133334 ZEC");
  await expect(page.getByTestId("invite-amount-note")).toContainText("raised");
  for (const bad of ["-1", "abc", "1e3", "0", "0x10", "0.000000001"]) {
    await page.goto("/shore/?invite=x&amount=" + encodeURIComponent(bad));
    await expect(page.getByTestId("invite-amount"), bad).toHaveCount(0);
    await expect(page.getByTestId("invite-amount-note"), bad).toContainText("ignored");
  }
});

test("unknown, empty, oversized and hostile invite params degrade gracefully", async ({ page }) => {
  const writes = await guard(page);
  await page.goto("/shore/?invite=bad%20code!&note=hi");
  await expect(page.locator("#invite")).toContainText("Invite link");
  await expect(page.locator("#invite")).toContainText("not recognized");
  await expect(page.getByTestId("invite-note")).toHaveText("hi");
  await page.goto("/shore/?invite=");
  await expect(page.locator("#invite")).toContainText("Invite link");
  await expect(page.getByTestId("connect")).toBeVisible();
  await page.goto("/shore/?invite=" + "a".repeat(40));
  await expect(page.locator("#invite")).toContainText("not recognized");
  await page.goto("/shore/?invite=x&note=" + "n".repeat(2000));
  await expect(page.locator("#invite")).toContainText("too long");
  await page.goto("/shore/?invite=x&note=" + encodeURIComponent("a\u202eb\u200b\nc   d " + "z".repeat(300)));
  const t = await page.getByTestId("invite-note").innerText();
  expect(t.length).toBeLessThanOrEqual(140);
  expect(t.startsWith("a b c d z")).toBe(true);
  expect(writes).toEqual([]);
});

test("judge variant ?invite=&replay=1 shows the banner, then plays the replay without mainnet", async ({ page }) => {
  const writes = await guard(page);
  const hits = [];
  page.on("request", (r) => { if (/fly\.dev|\/api\/|mainnet-beta|helius/.test(r.url())) hits.push(r.url()); });
  await page.goto("/shore/?invite=zechub&amount=0.005&note=Welcome&replay=1&speed=30");
  await expect(page.locator("#mode")).toContainText("Replay");
  await expect(page.getByTestId("invite-judge")).toContainText("nothing sent");
  await expect(page.getByTestId("invite-note")).toHaveText("Welcome");
  await expect(page.locator("#readback[data-landed]")).toBeAttached({ timeout: 40_000 });
  await expect(page.locator("html[data-tour='done']")).toBeAttached({ timeout: 20_000 });
  expect(writes).toEqual([]);
  expect(hits).toEqual([]);
});

test("invite amount pre-fills the quote form and stays editable", async ({ page }) => {
  await page.route("**/api/**", (r) => {
    const u = r.request().url();
    if (u.includes("/api/rpc")) return r.fulfill({ json: { jsonrpc: "2.0", id: 1, result: { value: [{ account: { data: { parsed: { info: { tokenAmount: { amount: "500000" } } } } } }] } } });
    if (u.includes("/shield/address")) return r.fulfill({ json: { id: "t1", recipient: OWN, receivers: ["orchard"], source: "own", readbackAvailable: false } });
    return r.fulfill({ status: 404, json: { error: "x" } });
  });
  await page.addInitScript(() => { window.phantom = { solana: { isPhantom: true, connect: async () => ({ publicKey: { toString: () => "ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5" } }), signAndSendTransaction: async () => { throw new Error("never signs"); } } }; });
  await page.goto("/shore/?invite=r2&amount=0.0021");
  await page.getByTestId("connect").click();
  await page.locator("#own").fill(OWN); await page.getByRole("button", { name: "Use this address" }).click();
  await expect(page.locator("#amt")).toHaveValue("0.0021");
  await expect(page.locator("#amt")).toBeEditable();
});

test("judge path: 'Try it in 60s' plays the whole replay hands-free and opens what's still public", async ({ page }) => {
  const writes = await guard(page);
  await page.goto("/shore/");
  const t = page.getByTestId("tour");
  await expect(t).toBeVisible();
  await expect(page.locator(".judge-steps li")).toHaveCount(3);
  await page.goto("/shore/?replay=1&tour=1&speed=30");
  await expect(page.getByTestId("recipient")).toContainText("u1uufrl3tucq9", { timeout: 20_000 });
  await expect(page.getByTestId("quote")).toContainText("0.00032 ZEC", { timeout: 20_000 });
  await expect(page.locator("#readback[data-landed]")).toBeAttached({ timeout: 40_000 });
  await expect(page.locator("html[data-tour='done']")).toBeAttached({ timeout: 20_000 });
  await expect(page.locator("#public")).toHaveAttribute("open", "");
  expect(writes).toEqual([]);
});

test("demand strip cites its sources and the business model is stated", async ({ page }) => {
  await page.goto("/shore/");
  const hrefs = await page.locator("#demand a").evaluateAll((as) => as.map((a) => a.href));
  for (const u of ["forum.zcashcommunity.com/t/38-000", "zechub.wiki/using-zcash/solana-zec-to-shielded", "x.com/zooko/status/2105044462397485491", "bounties.zechub.wiki", "x.com/zksnarks_"])
    expect(hrefs.some((h) => h.includes(u)), u).toBe(true);
  const biz = page.locator("#business .biz");
  await expect(biz).toContainText("appFees");
  await expect(biz).toContainText("invite links");
  expect((await biz.innerText()).split(/(?<=\.)\s+/).length).toBe(3);
});

// Full live flow with a stand-in Phantom (a real extension cannot run headless) and every API answer mocked:
// own u1 -> quote -> one signature -> status -> UNAVAILABLE read-back -> the user pastes a viewing key -> LANDED.
test("own-address read-back with the user's viewing key, end to end (mocked API, stand-in wallet)", async ({ page }) => {
  const SIG = "2LLVDBnVAqnG99PSbUVqapKqZFHjPPET9HsJhV2tqS29BHR4wNmUzYZbQzLf2BXjpFKMrRJCYxbVZq6ZJNQQ86au", ZTX = "fa8d4d6bd210c6150e98142ec1ae9a25706ec19bd591a775823f6616f95fa408";
  const KEY = "uview1" + "q".repeat(300); let keyBody = null;
  await page.route("**/api/**", async (r) => {
    const u = r.request().url(), m = r.request().method();
    if (u.endsWith("/api/rpc")) {
      const b = r.request().postDataJSON();
      if (b.method === "getLatestBlockhash") return r.fulfill({ json: { jsonrpc: "2.0", id: b.id, result: { context: { slot: 1 }, value: { blockhash: "EETubP5AKHgjPAhzPAFcb8BAY1hMH639CWCFTqi3hq1k", lastValidBlockHeight: 9e9 } } } });
      if (b.method === "getSignatureStatuses") return r.fulfill({ json: { jsonrpc: "2.0", id: b.id, result: { value: [{ confirmationStatus: "finalized", err: null }] } } });
      return r.fulfill({ json: { jsonrpc: "2.0", id: b.id, result: { value: [{ pubkey: "4oTPb1tUJ3Ed5D7ccD5hvJ6U5JQ6pRZ6Ntw5e2YMkTWS", account: { data: { parsed: { info: { tokenAmount: { amount: "500000" } } } } } }] } } });
    }
    if (u.includes("/shield/address")) return r.fulfill({ json: { id: "t1", recipient: OWN, receivers: ["orchard"], source: "own", readbackAvailable: false } });
    if (u.includes("/quote")) return r.fulfill({ json: { id: "t1", dry: false, depositAddress: "DMDdCzumrZBaPPjKBP33o1RgDhW2F2BZPzyYZEk9SjbM", mint: "A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS", decimals: 8, amountIn: "140000", amountOut: "108000", minAmountOut: "106920", fees: { withdrawFee: "32000", refundFee: "844", totalFeeBps: 2286, shoreFeeBps: 0 }, minAmount: "133334", deadline: "2030-10-13T06:47:48.000Z", timeEstimateSecs: 135, recipient: OWN } });
    if (u.includes("/tx")) return r.fulfill({ json: { ok: true } });
    if (u.includes("/api/status/")) return r.fulfill({ json: { status: "SUCCESS", terminal: true, solanaTx: SIG, zcashTx: ZTX, landed: { txid: ZTX, height: 3512504 } } });
    if (u.includes("/api/readback/") && m === "POST") { keyBody = r.request().postDataJSON(); return r.fulfill({ json: { id: "t1", state: "LANDED", source: "your-key", pool: "ironwood", valueZat: 115000, height: 3512504, txid: ZTX, quotedOutZat: 108000, deltaZat: 7000, scannedTo: 3512510 } }); }
    if (u.includes("/api/readback/")) return r.fulfill({ json: { id: "t1", state: "UNAVAILABLE" } });
    if (u.includes("/api/exposure/")) return r.fulfill({ json: { rows: [], addressReused: false, publicOnStatus: [] } });
    return r.fulfill({ status: 404, json: { error: "x" } });
  });
  await page.addInitScript((sig) => {
    const p = { isPhantom: true, connect: async () => ({ publicKey: { toString: () => "ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5" } }), signAndSendTransaction: async () => ({ signature: sig }) };
    window.phantom = { solana: p };
  }, SIG);
  await page.goto("/shore/?live=1");
  await page.getByTestId("connect").click();
  await page.locator("#own").fill(OWN); await page.getByRole("button", { name: "Use this address" }).click();
  await page.locator("#amt").fill("0.0014");
  await page.getByRole("button", { name: "Get quote" }).click();
  await page.locator("#ack").check();
  await page.getByTestId("approve").click();
  await expect(page.getByTestId("ufvk-go")).toBeVisible({ timeout: 30_000 });
  await page.locator("#ufvk").fill("abandon abandon abandon");
  await page.getByTestId("ufvk-go").click();
  await expect(page.getByTestId("ufvk-msg")).toContainText("not a unified viewing key");
  expect(keyBody).toBeNull();
  await page.locator("#ufvk").fill(KEY);
  await page.getByTestId("ufvk-go").click();
  await expect(page.locator("#readback[data-landed]")).toBeAttached({ timeout: 20_000 });
  expect(keyBody).toEqual({ ufvk: KEY });
  expect(await page.evaluate(() => JSON.stringify(localStorage) + document.cookie)).not.toContain("uview1");
});
