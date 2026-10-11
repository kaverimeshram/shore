// Live production audit, no spend: real Pages + real Fly API, stand-in Phantom (a real extension cannot be driven
// headless) that refuses to sign. Env: U1/K1 (throwaway wallet A u1 + key), K2 (wallet B key), U3 (wallet C u1 for the UI run).
import { chromium } from "@playwright/test";
const API = "https://airlock-zec.fly.dev", APP = "https://shore-zec.pages.dev/shore/";
const SENDER = "ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5";
const { U1, K1, K2, U3 } = process.env;
const out = {};
const j = async (path, body) => { const r = await fetch(API + path, body ? { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) } : {}); let v = null; try { v = await r.json(); } catch {} return [r.status, v]; };
const rpc = (method, params) => j("/api/rpc", { jsonrpc: "2.0", id: 1, method, params });
let [c, v] = await rpc("getTokenAccountsByOwner", [SENDER, { mint: "A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS" }, { encoding: "jsonParsed" }]);
out.balance = [c, v?.result?.value?.[0]?.account?.data?.parsed?.info?.tokenAmount?.uiAmountString];
out.rpcWrite = (await rpc("sendTransaction", []))[0];
out.transparent = (await j("/api/shield/address", { mode: "own", address: "t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs" }))[1]?.code;
[c, v] = await j("/api/shield/address", { mode: "own", address: U1 }); out.own = [c, v?.source, v?.readbackAvailable]; const id = v?.id;
out.reuse = await j("/api/shield/address", { mode: "own", address: U1 }).then(([c, v]) => [c, v?.code]);
[c, v] = await j("/api/quote", { id, amount: "140000", sender: SENDER, dry: true }); out.dry = [c, v?.amountOut, v?.fees];
[c, v] = await j("/api/quote", { id, amount: "140000", sender: SENDER, dry: false }); out.nondry = [c, !!v?.depositAddress, v?.deadline, v?.fees?.shoreFeeBps]; const dep = v?.depositAddress;
out.nondryAgain = (await j("/api/quote", { id, amount: "140000", sender: SENDER, dry: false }))[1]?.code;
out.status = await j("/api/status/" + dep).then(([c, v]) => [c, v?.status ?? v?.code]);
out.readbackGet = (await j("/api/readback/" + id))[1]?.state;
out.readbackWrongKey = (await j("/api/readback/" + id, { ufvk: K2 }))[1]?.code;
out.readbackSeed = (await j("/api/readback/" + id, { ufvk: "abandon abandon abandon" }))[1]?.code;
out.readbackRightKey = await j("/api/readback/" + id, { ufvk: K1 }).then(([c, v]) => [c, v?.state, v?.status ?? v?.reason]);
out.exposure = await j("/api/exposure/" + id).then(([c, v]) => [c, (v?.rows || []).length]);

const b = await chromium.launch(); const p = await (await b.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const signs = []; await p.exposeFunction("__sign", () => signs.push(1));
await p.addInitScript((pk) => { const w = { isPhantom: true, connect: async () => ({ publicKey: { toString: () => pk } }), signAndSendTransaction: async () => { window.__sign(); throw new Error("audit: signing refused, no spend"); } }; window.phantom = { solana: w }; window.solana = w; }, SENDER);
// The real wallet ${SENDER} holds less than the 1Click minimum, so only the token-balance answer is raised to 0.005 ZEC
// in the page (after the real /api/rpc call succeeded); everything else, including the non-dry quote, is the real API.
await p.route("**/api/rpc", async (r) => {
  const res = await r.fetch(); const body = await res.json();
  if (r.request().postDataJSON().method === "getTokenAccountsByOwner" && body.result?.value?.[0]) { out.uiRealBalanceSeen = body.result.value[0].account.data.parsed.info.tokenAmount.amount; const t = body.result.value[0].account.data.parsed.info.tokenAmount; t.amount = "500000"; t.uiAmount = 0.005; t.uiAmountString = "0.005"; }
  return r.fulfill({ response: res, json: body });
});
await p.goto(APP + "?live=1");
await p.getByTestId("connect").click();
await p.getByTestId("balance").waitFor(); await p.waitForFunction(() => /ZEC/.test(document.querySelector('[data-testid="balance"]')?.textContent || ""), null, { timeout: 30000 });
out.uiBalance = await p.getByTestId("balance").innerText();
out.uiFreshOffered = await p.getByTestId("fresh").count();
await p.locator("#own").fill(U1); await p.getByRole("button", { name: "Use this address" }).click(); await p.waitForTimeout(2500);
out.uiReuse = await p.locator("#s-address").innerText().then((t) => /used|reus/i.test(t));
await p.locator("#own").fill(U3); await p.getByRole("button", { name: "Use this address" }).click();
await p.getByTestId("recipient").waitFor({ timeout: 20000 });
await p.locator("#amt").fill("0.0014"); await p.getByRole("button", { name: "Get quote" }).click();
try { await p.getByTestId("approve").waitFor({ timeout: 30000 }); } catch (e) { out.quoteStep = await p.locator("#s-quote").innerText(); out.addrStep = await p.locator("#s-address").innerText(); console.log(JSON.stringify(out, null, 1)); process.exit(1); }
out.uiApproveText = await p.getByTestId("approve").innerText();
out.uiDeposit = await p.locator("#s-approve").innerText().then((t) => /Deposit address/.test(t));
await p.locator("#ack").check(); await p.getByTestId("approve").click(); await p.waitForTimeout(2500);
out.uiSignAttempts = signs.length; out.uiAfter = (await p.locator("#s-approve").innerText()).slice(0, 200);
await p.screenshot({ path: "../docs/submission/live-audit.png", fullPage: true });
await b.close();
console.log(JSON.stringify(out, null, 1));
