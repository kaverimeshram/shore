import { test, expect } from "@playwright/test";

// Fails the test if the page ever touches a wallet or a write endpoint in replay.
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
  await page.getByTestId("fresh").click();
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
  await page.getByTestId("fresh").click();
  await expect(page.getByTestId("recipient")).toContainText("u1", { timeout: 20_000 });
  await page.locator("#amt").fill("0.0014");
  await page.getByRole("button", { name: "Get quote" }).click();
  await expect(page.getByTestId("quote")).toContainText("0.00032 ZEC", { timeout: 30_000 });
  await expect(page.getByTestId("approve")).toBeDisabled();
  expect(rpc.length).toBe(1);
  expect(Array.isArray(rpc[0].result?.value), JSON.stringify(rpc[0]).slice(0, 200)).toBe(true);
});
