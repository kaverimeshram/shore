import { test, expect } from "@playwright/test";

// Every error a holder can hit reads in plain words with a next step. Mocked API, stand-in Phantom, nothing sent.
const OWN = "u1qrdwfnpakf2xhw9rpsuvsxmnk6qkz2cm7xmk4m2r2hf7ug4fhz9kjv5x9tmuqhx6wysmhhqw3edpa0yepl4lhjkvd7fys09rrhj7ud4sfu9dg";
const SIG = "2LLVDBnVAqnG99PSbUVqapKqZFHjPPET9HsJhV2tqS29BHR4wNmUzYZbQzLf2BXjpFKMrRJCYxbVZq6ZJNQQ86au", ZTX = "fa8d4d6bd210c6150e98142ec1ae9a25706ec19bd591a775823f6616f95fa408";
const fail = (status, code, error) => ({ status, json: { code, error } });

async function setup(page, o = {}) {
  const hits = { quote: 0 };
  await page.route("**/api/**", async (r) => {
    const u = r.request().url(), m = r.request().method();
    if (u.endsWith("/api/rpc")) {
      const b = r.request().postDataJSON();
      if (b.method === "getLatestBlockhash") return r.fulfill({ json: { jsonrpc: "2.0", id: b.id, result: { context: { slot: 1 }, value: { blockhash: "EETubP5AKHgjPAhzPAFcb8BAY1hMH639CWCFTqi3hq1k", lastValidBlockHeight: 9e9 } } } });
      if (b.method === "getSignatureStatuses") return r.fulfill({ json: { jsonrpc: "2.0", id: b.id, result: { value: [{ confirmationStatus: "finalized", err: null }] } } });
      return r.fulfill({ json: { jsonrpc: "2.0", id: b.id, result: { value: [{ pubkey: "4oTPb1tUJ3Ed5D7ccD5hvJ6U5JQ6pRZ6Ntw5e2YMkTWS", account: { data: { parsed: { info: { tokenAmount: { amount: "500000" } } } } } }] } } });
    }
    if (u.includes("/shield/address")) return r.fulfill(o.address || { json: { id: "t1", recipient: OWN, receivers: ["orchard"], source: "own", readbackAvailable: false } });
    if (u.includes("/quote")) { hits.quote++; return r.fulfill({ json: { id: "t1", dry: false, depositAddress: "DMDdCzumrZBaPPjKBP33o1RgDhW2F2BZPzyYZEk9SjbM", mint: "A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS", decimals: 8, amountIn: "140000", amountOut: "108000", minAmountOut: "106920", fees: { withdrawFee: "32000", refundFee: "844", totalFeeBps: 2286, shoreFeeBps: 0 }, minAmount: "133334", deadline: "2030-10-13T06:47:48.000Z", timeEstimateSecs: 135, recipient: OWN } }); }
    if (u.includes("/tx")) return r.fulfill({ json: { ok: true } });
    if (u.includes("/api/status/")) return r.fulfill({ json: { status: "SUCCESS", terminal: true, solanaTx: SIG, zcashTx: ZTX, landed: { txid: ZTX, height: 3512504 } } });
    if (u.includes("/api/readback/") && m === "POST") return r.fulfill(o.readback);
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
  return hits;
}
const useAddr = async (page, a = OWN) => { await page.locator("#own").fill(a); await page.getByRole("button", { name: "Use this address" }).click(); };

test("bad u1 and transparent addresses say what to paste instead", async ({ page }) => {
  await setup(page);
  const err = page.locator("#s-address .err");
  await useAddr(page, "u1notanaddress");
  await expect(err).toContainText("not a Zcash unified address");
  await expect(err).toContainText("Receive screen");
  await useAddr(page, "t1Kx9d7bQjzS6dZ8WxwB4bS9mV2fH3bK9nP");
  await expect(err).toContainText("transparent address");
  await expect(err).toContainText("u1");
});

test("server-side reuse (409 ADDRESS_REUSED) reads in plain words with a next step", async ({ page }) => {
  await setup(page, { address: fail(409, "ADDRESS_REUSED", "this address already received a shield; reuse links the two deposits") });
  await useAddr(page);
  const err = page.locator("#s-address .err");
  await expect(err).toContainText("already received a shield");
  await expect(err).toContainText("Get a new receive address");
});

test("below-minimum, above-cap and balance-too-low amounts are caught before any quote", async ({ page }) => {
  const hits = await setup(page);
  await useAddr(page);
  const amt = page.locator("#amt"), go = page.getByRole("button", { name: "Get quote" }), err = page.locator("#s-quote .err");
  await amt.fill("0.001"); await go.click();
  await expect(err).toContainText("below 1Click's minimum of 0.00133334 ZEC");
  await amt.fill("0.05"); await go.click();
  await expect(err).toContainText("above SHORE's launch cap of 0.02 ZEC");
  await amt.fill("0.01"); await go.click();
  await expect(err).toContainText("holds 0.005");
  await expect(err).toContainText("add ZEC to Phantom");
  expect(hits.quote).toBe(0);
});

for (const [code, want] of [["BAD_UFVK", "export the viewing key"], ["KEY_MISMATCH", "belongs to a different wallet"]]) {
  test(`read-back ${code} from the API reads in plain words with a next step`, async ({ page }) => {
    await setup(page, { readback: fail(400, code, "raw server text") });
    await useAddr(page);
    await page.locator("#amt").fill("0.0014");
    await page.getByRole("button", { name: "Get quote" }).click();
    await page.locator("#ack").check();
    await page.getByTestId("approve").click();
    await expect(page.getByTestId("ufvk-go")).toBeVisible({ timeout: 30_000 });
    await page.locator("#ufvk").fill("uview1" + "q".repeat(300));
    await page.getByTestId("ufvk-go").click();
    const msg = page.getByTestId("ufvk-msg");
    await expect(msg).toContainText(want);
    await expect(msg).toContainText(code);
    await expect(msg).not.toContainText("raw server text");
  });
}
