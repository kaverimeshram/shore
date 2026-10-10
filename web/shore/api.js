// SHORE data layer. Three modes:
//   live   : the AIRLOCK server (docs/API.md, contract v1)
//   replay : the recorded mainnet run of 2026-10-10 (replay.json), played back on a clock. No wallet, no network writes.
//   mock   : made-up values for UI work, clearly labelled. No wallet, no network writes.
const qs = new URLSearchParams(location.search);
// Replay is the default so a judge lands on the recorded run; ?live=1 opens the real flow.
export const MODE = qs.has("live") ? "live" : qs.has("mock") ? "mock" : "replay";
export const SPEED = Number(qs.get("speed")) || (MODE === "replay" ? 6 : 1); // replay compresses 84 s
export const API = window.SHORE_API || window.AIRLOCK_API || "";

export class ApiError extends Error {
  constructor(msg, code, status) { super(msg); this.code = code; this.status = status; }
}

async function call(path, body) {
  let res;
  try {
    res = await fetch(API + path, body === undefined ? { cache: "no-store" } : {
      method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body),
    });
  } catch { throw new ApiError("The SHORE service did not answer. Try again, or open the replay.", "NETWORK", 0); }
  let data = null;
  try { data = await res.json(); } catch { /* empty */ }
  if (!res.ok) throw new ApiError((data && data.error) || `HTTP ${res.status}`, data && data.code, res.status);
  return data;
}

let recorded = null;
export async function loadReplay() {
  if (!recorded) recorded = await (await fetch("replay.json", { cache: "no-store" })).json();
  return recorded;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let t0 = 0; // replay/mock clock start (ms), set when the "transfer" is approved
export function startClock() { t0 = performance.now(); }
const elapsed = () => ((performance.now() - t0) / 1000) * SPEED;

function fakeSession(rec) {
  return { id: "replay-0001", recipient: rec.recipient, receivers: ["orchard"], source: "fresh", uaIndex: 0, readbackAvailable: true };
}

export async function shieldAddress(req) {
  if (MODE === "live") return call("/api/shield/address", req);
  const rec = await loadReplay();
  await sleep(350);
  if (req.mode === "own") return { ...fakeSession(rec), recipient: req.address, source: "own", uaIndex: undefined, readbackAvailable: MODE === "replay" };
  return fakeSession(rec);
}

export async function quote(req) {
  if (MODE === "live") return call("/api/quote", req);
  const rec = await loadReplay();
  await sleep(500);
  const q = rec.quote;
  const amt = BigInt(req.amount);
  if (amt < BigInt(q.minAmountIn) - 6666n) throw new ApiError("Below the 1Click minimum of 0.00133334 ZEC.", "BELOW_MINIMUM", 400);
  if (amt > 2000000n) throw new ApiError("Above the 0.02 ZEC launch cap.", "ABOVE_CAP", 400);
  // replay always returns the recorded quote (amount is fixed to 140000 in the UI)
  const out = MODE === "replay" ? BigInt(q.amountOut) : (amt * 771n) / 1000n;
  const fee = amt - out;
  return {
    id: req.id, depositAddress: q.depositAddress, mint: "A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS", decimals: 8,
    amountIn: String(MODE === "replay" ? q.amountIn : amt), amountOut: String(out), minAmountOut: String((out * 99n) / 100n),
    fees: { withdrawFee: q.withdrawFee, refundFee: q.refundFee, totalFeeBps: Number((fee * 10000n) / (MODE === "replay" ? BigInt(q.amountIn) : amt)) },
    minAmount: "133334", deadline: q.deadline, timeEstimateSecs: q.timeEstimate, recipient: rec.recipient,
    correlationId: MODE === "replay" ? "recorded" : "mock",
  };
}

// Recorded timeline (seconds after the Solana transfer). Total send-to-land = 84 s, per the evidence.
export async function status(deposit) {
  if (MODE === "live") return call("/api/status/" + encodeURIComponent(deposit));
  const rec = await loadReplay();
  const s = elapsed();
  const base = { depositAddress: deposit, terminal: false, updatedAt: new Date().toISOString(), quotedOut: rec.quote.amountOut, refundReason: null, solanaTx: null, zcashTx: null, landed: null };
  if (s < 4) return { ...base, status: "PENDING_DEPOSIT" };
  if (s < 18) return { ...base, status: "KNOWN_DEPOSIT_TX", solanaTx: rec.solana_tx };
  if (s < rec.seconds - 6) return { ...base, status: "PROCESSING", solanaTx: rec.solana_tx };
  if (s < rec.seconds) return { ...base, status: "PROCESSING", solanaTx: rec.solana_tx, zcashTx: rec.zcash_txid };
  return { ...base, status: "SUCCESS", terminal: true, solanaTx: rec.solana_tx, zcashTx: rec.zcash_txid, nearTx: rec.near_tx, landed: { txid: rec.zcash_txid, height: rec.zcash_height } };
}

export async function readback(id) {
  if (MODE === "live") return call("/api/readback/" + encodeURIComponent(id));
  const rec = await loadReplay();
  await sleep(300);
  if (elapsed() < rec.seconds) return { id, state: "WAITING" };
  return { id, state: "LANDED", pool: rec.pool, valueZat: rec.received_zat, height: rec.zcash_height, txid: rec.zcash_txid,
    quotedOutZat: Number(rec.quote.amountOut), deltaZat: rec.received_zat - Number(rec.quote.amountOut), scannedTo: rec.zcash_height + 8 };
}

export async function exposure(id, ctx = {}) {
  if (MODE === "live") return call("/api/exposure/" + encodeURIComponent(id));
  const rec = await loadReplay();
  return { id, addressReused: false, rows: [
    { what: "Solana sender", value: ctx.sender || rec.sender, whoCanSee: "anyone (Solana explorer)", source: "solana tx", exposed: true },
    { what: "Deposit address", value: ctx.deposit || rec.deposit, whoCanSee: "anyone (Solana explorer)", source: "solana tx", exposed: true },
    { what: "Amount in and timing", value: "0.0014 ZEC", whoCanSee: "anyone (Solana explorer)", source: "solana tx", exposed: true },
    { what: "Recipient u1 via 1Click status", value: ctx.recipient || rec.recipient, whoCanSee: "anyone holding the deposit address (unauthenticated /v0/status)", source: "1click status", exposed: true },
    { what: "Amount entering the shielded pool", value: String(rec.received_zat), whoCanSee: "anyone (delivery tx has transparent inputs)", source: "zcash tx", exposed: true },
    { what: "Note contents to the payer", whoCanSee: "1Click/NEAR (sender holds the outgoing view key)", source: "protocol", exposed: true },
    { what: "Later spends of the note", whoCanSee: "nobody", source: "protocol", exposed: false },
  ] };
}

export async function notifyTx(id, txHash) {
  if (MODE !== "live") return;
  try { await call(`/api/shield/${encodeURIComponent(id)}/tx`, { txHash }); } catch { /* optional */ }
}
