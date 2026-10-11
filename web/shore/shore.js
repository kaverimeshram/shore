import * as api from "./api.js";

const MODE = api.MODE;
const LIVE = MODE === "live";
const MINT = "A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS";
// Solana reads go through the SHORE service, which holds the provider key (POST /api/rpc).
const RPC = window.SHORE_RPC || api.API + "/api/rpc";
const USED_KEY = "shore.usedRecipients";
const EXPL = {
  sol: (s) => `https://solscan.io/tx/${s}`,
  solAcct: (a) => `https://solscan.io/account/${a}`,
  near: (h) => `https://nearblocks.io/txns/${h}`,
  zec: (t) => `https://mainnet.zcashexplorer.app/transactions/${t}`,
  zecBlock: (h) => `https://mainnet.zcashexplorer.app/blocks/${h}`,
  status: (d) => `https://1click.chaindefuser.com/v0/status?depositAddress=${d}`,
};

// ---------- invite links ----------
// /shore/?invite=<code>&amount=<ZEC>&note=<text>: a sender (an onboarder, a bounty host) shares a link that opens the
// same flow pre-filled. Nothing is stored or sent anywhere: the code is a label, the amount is only a suggestion the
// user can change, the note is shown as plain text and marked as the sender's words. No custody, no new trust.
export const INVITE_MIN = 0.00133334, INVITE_CAP = 0.02; // 1Click floor and the launch cap, as in the quote step
export function parseInvite(search) {
  if (typeof search !== "string" || search.length > 1024) return /[?&]invite=/.test(String(search).slice(0, 2048)) ? { error: "This invite link is too long, so it was ignored." } : null;
  const q = new URLSearchParams(search);
  if (!q.has("invite")) return null;
  const rawCode = (q.get("invite") || "").trim();
  // Unknown or empty codes still open the flow; only a well-formed code is shown, and it is never looked up anywhere.
  const code = /^[A-Za-z0-9_-]{1,32}$/.test(rawCode) ? rawCode : null;
  const codeIgnored = !!rawCode && !code;
  const raw = (q.get("amount") || "").trim().slice(0, 16);
  let amount = null, amountNote = null;
  if (raw) {
    const n = /^\d{1,2}(\.\d{1,8})?$|^\.\d{1,8}$/.test(raw) ? Number(raw) : NaN;
    if (!(n > 0)) amountNote = "The amount in this link was not valid, so it was ignored.";
    else if (n < INVITE_MIN) { amount = String(INVITE_MIN); amountNote = `The link asked for less than the 1Click minimum, so it was raised to ${INVITE_MIN} ZEC.`; }
    else if (n > INVITE_CAP) { amount = String(INVITE_CAP); amountNote = `The link asked for more than the launch cap, so it was lowered to ${INVITE_CAP} ZEC.`; }
    else amount = raw;
  }
  const note = (q.get("note") || "").slice(0, 600).replace(/[\u0000-\u001f\u007f-\u009f\u200b-\u200f\u2028-\u202e\u2060-\u2069\ufeff]/g, " ").replace(/\s+/g, " ").trim().slice(0, 140) || null;
  return { code, codeIgnored, amount, amountNote, amountIgnored: !!raw && !amount, note };
}
const INVITE = parseInvite(location.search);

const S = { wallet: null, balance: null, session: null, quote: null, sig: null, status: null, rb: null, exposure: null };

// ---------- tiny DOM helpers ----------
function h(tag, attrs = {}, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs || {})) {
    if (v == null || v === false) continue;
    if (k.startsWith("on")) el.addEventListener(k.slice(2), v);
    else if (k === "class") el.className = v;
    else el.setAttribute(k, v === true ? "" : v);
  }
  for (const k of kids.flat()) if (k != null && k !== false) el.append(k instanceof Node ? k : String(k));
  return el;
}
const $ = (id) => document.getElementById(id);
const body = (id) => $(id).querySelector("[data-body]");
function setStep(id, state, tag = "") {
  const li = $(id); li.dataset.state = state; li.querySelector("[data-tag]").textContent = tag;
}
function put(el, ...kids) { el.replaceChildren(...kids.flat().filter(Boolean)); }
const zec = (zat) => { const n = BigInt(zat); const s = (n < 0n ? -n : n).toString().padStart(9, "0"); return `${n < 0n ? "-" : ""}${s.slice(0, -8)}.${s.slice(-8).replace(/0+$/, "") || "0"}`; };
const short = (s, n = 6) => (s && s.length > 2 * n + 3 ? `${s.slice(0, n)}…${s.slice(-n)}` : s);
const link = (href, text) => h("a", { href, target: "_blank", rel: "noopener" }, text);
const err = (e) => h("p", { class: "err", role: "alert" }, e.message || String(e), e.code ? ` (${e.code})` : "");
function kv(rows) { return h("dl", { class: "kv" }, rows.filter(Boolean).flatMap(([k, v, cls]) => [h("dt", {}, k), h("dd", { class: cls }, v)])); }
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const REDUCED = matchMedia("(prefers-reduced-motion: reduce)").matches;
// ---------- the ledger: the current number is always the biggest thing on screen ----------
const L = {};
function ledger(key, label, value, opts = {}) {
  if (key) L[key] = [label, value];
  $("intro").hidden = true; $("ledger").hidden = false;
  $("crossing").dataset.live = "";
  if (opts.hero) { put($("hero-k"), opts.hero[0]); put($("hero-v"), ...[].concat(opts.hero[1])); $("hero-v").dataset.tone = opts.tone || "";
    put($("strip-k"), opts.strip ? opts.strip[0] : opts.hero[0]); put($("strip-v"), opts.strip ? opts.strip[1] : $("hero-v").textContent); $("strip-now").dataset.tone = opts.tone || ""; }
  const order = ["send", "lands", "fee", "elapsed"];
  put($("lrows"), order.filter((k) => L[k]).flatMap((k) => [h("dt", {}, L[k][0]), h("dd", { "data-k": k }, L[k][1])]));
}
function receipt(id, tag, ...line) {
  put(body(id), h("p", { class: "receipt" }, ...line.filter(Boolean)));
  setStep(id, "done", tag);
}
const STEPS = ["s-connect", "s-address", "s-quote", "s-approve", "s-land"];
function reopen(id, render) {
  for (const s of STEPS.slice(STEPS.indexOf(id) + 1)) { setStep(s, "locked"); put(body(s)); }
  if (id === "s-address") { delete L.send; delete L.lands; delete L.fee; ledger(null, null, null, { hero: ["Balance on Solana", [zec(S.balance), h("small", {}, "ZEC")]] }); }
  setStep(id, "active"); render();
}
const change = (id, render) => [h("span", { class: "chg" }, " · ", h("button", { class: "linkbtn", type: "button", onclick: () => reopen(id, render) }, "Change"))];
const IST = (iso) => new Date(iso).toLocaleString("en-GB", { timeZone: "Asia/Kolkata", day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", hour12: false }) + " IST";
function countdown(iso) {
  const ms = new Date(iso) - Date.now();
  if (ms <= 0) return "expired";
  const m = Math.floor(ms / 60000), d = Math.floor(m / 1440), hr = Math.floor((m % 1440) / 60);
  return d ? `valid ${d}d ${hr}h` : hr ? `valid ${hr}h ${m % 60}m` : `valid ${m}m`;
}
function countUp(el, zat, suffix) {
  const end = BigInt(zat);
  if (REDUCED) { put(el, zec(end), suffix); return; }
  const t0 = performance.now();
  const step = (t) => { const f = Math.min(1, (t - t0) / 450); const e = 1 - (1 - f) ** 3;
    put(el, zec((end * BigInt(Math.round(e * 1000))) / 1000n), suffix); if (f < 1) requestAnimationFrame(step); };
  requestAnimationFrame(step);
}

const used = () => { try { return JSON.parse(localStorage.getItem(USED_KEY)) || []; } catch { return []; } };
const markUsed = (a) => localStorage.setItem(USED_KEY, JSON.stringify([...new Set([...used(), a])]));

// ---------- address rules (server enforces the same, plus server-side reuse) ----------
export function checkRecipient(raw) {
  const a = (raw || "").trim();
  if (!a) return "Paste a shielded address, or let SHORE make a fresh one.";
  if (/^t[13m]/.test(a)) return "That is a transparent address. SHORE only delivers to shielded receivers.";
  if (/^zs1/.test(a)) return "That is a Sapling address. Paste a unified address (u1…) with an Orchard receiver.";
  if (!/^(u1|zu1)[02-9ac-hj-np-z]{100,}$/.test(a)) return "That does not look like a unified address (u1…).";
  if (used().includes(a)) return "This address was used for an earlier shield. 1Click status ties it to that deposit forever, so use a fresh one.";
  return null;
}

// ---------- wallet ----------
async function solBalance(owner) {
  const r = await fetch(RPC, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({
    jsonrpc: "2.0", id: 1, method: "getTokenAccountsByOwner", params: [owner, { mint: MINT }, { encoding: "jsonParsed" }] }) });
  const j = await r.json();
  if (j.error) throw new Error(j.error.message);
  return j.result.value.reduce((t, a) => t + BigInt(a.account.data.parsed.info.tokenAmount.amount), 0n);
}

async function connect() {
  const b = body("s-connect");
  try {
    if (LIVE) {
      const p = window.phantom?.solana || (window.solana?.isPhantom ? window.solana : null);
      if (!p) { put(b, h("p", {}, "Phantom is not installed in this browser. "), link("https://phantom.app/download", "Get Phantom"), h("p", { class: "help" }, "Or open the replay of the recorded run.")); return; }
      const r = await p.connect();
      S.wallet = { provider: p, address: r.publicKey.toString() };
      S.balance = await solBalance(S.wallet.address);
    } else {
      const rec = await api.loadReplay();
      S.wallet = { provider: null, address: rec.sender };
      S.balance = BigInt(rec.quote.amountIn);
    }
    renderConnected();
  } catch (e) { b.append(err(e)); }
}

function renderConnected() {
  receipt("s-connect", "connected",
    h("span", { class: "val", "data-testid": "balance" }, `${zec(S.balance)} ZEC`), " on Solana · ",
    link(EXPL.solAcct(S.wallet.address), short(S.wallet.address, 4)), MODE !== "live" ? " (recorded wallet)" : "");
  if (S.balance === 0n) body("s-connect").append(h("p", { class: "err" }, "No ZEC (SPL mint A7bd…XaS) in this wallet."));
  ledger(null, null, null, { hero: ["Balance on Solana", [zec(S.balance), h("small", {}, "ZEC")]] });
  setStep("s-address", "active");
  renderAddress();
}

// ---------- step 2 ----------
function renderAddress() {
  const b = body("s-address");
  const input = h("input", { type: "text", id: "own", placeholder: "Paste a fresh u1", autocomplete: "off", spellcheck: "false", "aria-describedby": "own-help" });
  const msg = h("p", { class: "help", id: "own-help" }, "Each shield gets its own address. Reuse would let anyone link this Phantom wallet to every shield sent there.");
  const fresh = h("button", { class: "btn", type: "button", "data-testid": "fresh", onclick: () => make({ mode: "fresh" }) }, "Make a fresh u1");
  const own = h("button", { class: "btn ghost", type: "button", onclick: () => {
    const why = checkRecipient(input.value);
    if (why) { put(msg, why); msg.className = "err"; return; }
    make({ mode: "own", address: input.value.trim() });
  } }, "Use this address");
  async function make(req) {
    fresh.disabled = own.disabled = true;
    try { S.session = await api.shieldAddress(req); renderAddressDone(); }
    catch (e) { put(msg, e.message); msg.className = "err"; fresh.disabled = own.disabled = false; }
  }
  // Fresh addresses come from the SHORE demo wallet (replay and mock only). On mainnet the ZEC must go to the user's own wallet.
  if (LIVE) { put(msg, "Paste a new receiving address from your own Zcash wallet (Zashi/Zodl, Ywallet). The ZEC lands there; SHORE never holds it."); put(b, h("label", { class: "f" }, h("span", { class: "label" }, "Your shielded address (u1…)"), input), msg, h("div", { class: "row" }, own)); return; }
  put(b, h("div", { class: "row" }, fresh), h("p", { class: "label" }, "Or paste your own"), h("label", { class: "f" }, input), msg, h("div", { class: "row" }, own));
}

function renderAddressDone() {
  const s = S.session;
  receipt("s-address", s.source === "fresh" ? "fresh" : "yours",
    h("span", { class: "val", "data-testid": "recipient", title: s.recipient }, short(s.recipient, 13)), " · ",
    (s.receivers || []).includes("orchard") ? "Orchard receiver" : "shielded receiver",
    s.source === "fresh" ? `, never used before (index ${s.uaIndex ?? "?"})` : ", your wallet", ...change("s-address", renderAddress));
  if (!s.readbackAvailable) body("s-address").append(h("p", { class: "help warn" }, "Read-back: after it lands, open your wallet or paste its viewing key (never a spending key)."));
  setStep("s-quote", "active");
  renderQuoteForm();
}

// ---------- step 3 ----------
function renderQuoteForm() {
  const fixed = MODE === "replay";
  const amt = h("input", { type: "text", inputmode: "decimal", id: "amt", value: fixed ? "0.0014" : INVITE?.amount || "", placeholder: "0.0014", readonly: fixed });
  const out = h("div");
  const go = h("button", { class: "btn", type: "button", onclick: run }, "Get quote");
  async function run() {
    const m = /^\s*(\d*)(?:\.(\d{0,8}))?\s*$/.exec(amt.value);
    if (!m || (!m[1] && !m[2])) { put(out, err(new Error("Enter an amount in ZEC, up to 8 decimals."))); return; }
    const base = BigInt((m[1] || "0") + (m[2] || "").padEnd(8, "0"));
    if (S.balance != null && base > S.balance) { put(out, err(new Error("More than this wallet holds."))); return; }
    go.disabled = true; put(out, h("p", { class: "help" }, "Asking 1Click…"));
    try { S.quote = await api.quote({ id: S.session.id, amount: String(base), sender: S.wallet.address, dry: false }); renderQuote(out); }
    catch (e) { put(out, err(e)); }
    go.disabled = false;
  }
  put(body("s-quote"), h("label", { class: "f" }, h("span", { class: "label" }, "Amount (ZEC on Solana)"), amt),
    h("p", { class: "help" }, "Minimum 0.00133334 ZEC (1Click floor), launch cap 0.02 ZEC"), h("div", { class: "row" }, go), out);
}

function renderQuote(out) {
  const q = S.quote;
  const bps = q.fees?.totalFeeBps ?? Number(((BigInt(q.amountIn) - BigInt(q.amountOut)) * 10000n) / BigInt(q.amountIn));
  const pct = (bps / 100).toFixed(1);
  if (q.recipient && q.recipient !== S.session.recipient) { put(out, err(new Error("Quote recipient differs from your address. Not continuing."))); return; }
  const valid = h("span", { "data-testid": "expiry", title: new Date(q.deadline).toISOString().replace(".000Z", "Z") }, `${countdown(q.deadline)} (until ${IST(q.deadline)})`);
  setInterval(() => put(valid, `${countdown(q.deadline)} (until ${IST(q.deadline)})`), 30000);
  const step = body("s-quote");
  put(step, h("div", { "data-testid": "quote", class: "qt" },
    h("p", { class: "qpair" }, h("span", {}, h("small", {}, "You send"), `${zec(q.amountIn)} ZEC`), h("i", { "aria-hidden": "true" }, "→"),
      h("span", {}, h("small", {}, "Lands, quoted"), `${zec(q.amountOut)} ZEC`)),
    h("p", { class: "qsec" }, h("span", { class: "qmini" }, `${zec(q.amountIn)} → ${zec(q.amountOut)} ZEC · `), h("span", { class: bps > 1000 ? "warn" : "" }, `${pct}% fee`), ` · about ${q.timeEstimateSecs}s · `, valid),
    bps > 1000 ? h("p", { class: "help", "data-testid": "fee-honest" }, `Most of it is the fixed ${zec(q.fees.withdrawFee)} ZEC Zcash withdraw fee, so the percentage falls fast as the amount grows.`) : null,
    h("details", { class: "fold" }, h("summary", {}, "Fee breakdown (6)"),
      kv([["Worst case after slippage", `${zec(q.minAmountOut)} ZEC`], ["Zcash withdraw fee", `${zec(q.fees.withdrawFee)} ZEC`], ["Refund fee if it fails", `${zec(q.fees.refundFee)} ZEC`], ["SHORE fee", q.fees.shoreFeeBps ? `${q.fees.shoreFeeBps / 100}% (1Click appFees)` : "none today (0%)"],
        ["Total cost", `${pct}% of what you send`], ["1Click minimum", `${zec(q.minAmount)} ZEC`], ["Quote expires", IST(q.deadline)]])),
    MODE === "replay" ? h("details", { class: "fold" }, h("summary", {}, "Source"), h("p", { class: "help" }, "Recorded 1Click quote, correlation 9136ee09, file shore-funded-quote-v2.json.")) : null,
    h("p", { class: "chg" }, h("button", { class: "linkbtn", type: "button", onclick: () => reopen("s-quote", renderQuoteForm) }, "Change amount"))));
  setStep("s-quote", "done", "quoted");
  ledger("send", "Send", `${zec(q.amountIn)} ZEC`);
  ledger("fee", "Fee", h("span", { class: bps > 1000 ? "warn" : "" }, `${pct}%`));
  ledger("lands", "Lands, quoted", `${zec(q.amountOut)} ZEC`, { hero: ["Lands in the shielded pool, quoted", [zec(q.amountOut), h("small", {}, "ZEC")]], strip: ["Lands, quoted", `${zec(q.amountOut)} ZEC`] });
  setStep("s-approve", "active");
  renderApprove();
}

// ---------- step 4 ----------
function renderApprove() {
  const q = S.quote;
  const ack = h("input", { type: "checkbox", id: "ack" });
  const btn = h("button", { class: "btn big", type: "button", "data-testid": "approve", disabled: LIVE }, LIVE ? `Approve ${zec(q.amountIn)} ZEC in Phantom` : `Approve in Phantom (${MODE})`);
  ack.addEventListener("change", () => { btn.disabled = !ack.checked; });
  const msg = h("div");
  btn.addEventListener("click", async () => {
    btn.disabled = true;
    try {
      if (LIVE) S.sig = await sendTransfer(q);
      else { put(msg, h("p", { class: "help" }, "Approved (simulated, nothing is sent).")); await sleep(600); S.sig = (await api.loadReplay()).solana_tx; }
      document.documentElement.dataset.signed = ""; // a signed transfer cannot be changed: hide every Change link
      if (S.session.source === "own") markUsed(S.session.recipient);
      api.notifyTx(S.session.id, S.sig);
      api.startClock();
      receipt("s-approve", "signed", "Signed · ", link(EXPL.sol(S.sig), short(S.sig, 6)));
      setStep("s-land", "active");
      land();
    } catch (e) { put(msg, err(e)); btn.disabled = !(ack.checked || !LIVE); }
  });
  put(body("s-approve"),
    h("p", {}, "One SPL transfer of ", h("b", {}, `${zec(q.amountIn)} ZEC`), " to the 1Click deposit address."),
    kv([["Deposit address", link(EXPL.solAcct(q.depositAddress), short(q.depositAddress, 8))], ["Token", short(q.mint, 6)], ["Refunds go to", short(S.wallet.address, 8)]]),
    LIVE ? h("label", { class: "check" }, ack, h("span", {}, "This is mainnet. Real ZEC leaves this wallet, and the hop is public (see the panel).")) : null,
    h("div", { class: "row" }, btn), msg);
}

async function sendTransfer(q) {
  const sol = await import("./vendor/sol.js");
  const conn = new sol.Connection(RPC, "confirmed");
  const owner = new sol.PublicKey(S.wallet.address), mint = new sol.PublicKey(q.mint), dest = new sol.PublicKey(q.depositAddress);
  const from = sol.getAssociatedTokenAddressSync(mint, owner), to = sol.getAssociatedTokenAddressSync(mint, dest, true);
  const tx = new sol.Transaction().add(
    sol.createAssociatedTokenAccountIdempotentInstruction(owner, to, dest, mint),
    sol.createTransferCheckedInstruction(from, mint, to, owner, BigInt(q.amountIn), q.decimals));
  tx.feePayer = owner;
  tx.recentBlockhash = (await conn.getLatestBlockhash()).blockhash;
  const { signature } = await S.wallet.provider.signAndSendTransaction(tx);
  return signature;
}

// ---------- step 5 ----------
// The crossing tracker (signature component): Solana -> bridge -> shielded pool, with elapsed time.
// idle shows the last recorded mainnet crossing; a new crossing resets it.
function setCrossing(phase) {
  const c = $("crossing"); c.dataset.phase = phase;
  const at = { idle: -1, send: 0, bridge: 1, landed: 3 }[phase];
  c.querySelectorAll(".node").forEach((n, i) => { n.className = "node" + (i < at ? " on" : i === at ? " now" : ""); });
  if (phase === "landed") tide(1); else if (phase === "idle") tide(0);
}
function tide(f) { $("tide-fill").style.clipPath = `inset(0 ${(100 - 100 * Math.max(0, Math.min(1, f))).toFixed(1)}% 0 0)`; }
const PHASE_OF = { PENDING_DEPOSIT: "send", KNOWN_DEPOSIT_TX: "send", INCOMPLETE_DEPOSIT: "send", PROCESSING: "bridge", SUCCESS: "landed" };
async function land() {
  const b = body("s-land");
  const clock = $("clock"), cap = $("clock-cap");
  const links = h("div", { class: "linksrow" });
  const note = h("div");
  const now = h("p", {}, "Crossing now. Nothing else to sign.");
  put(b, now, links, note);
  setCrossing("send");
  cap.textContent = MODE === "replay" ? "recorded time, replayed faster" : "since you approved";
  const start = performance.now();
  const eta = MODE === "replay" ? (await api.loadReplay()).seconds : S.quote.timeEstimateSecs || 135;
  const tick = setInterval(() => { const sec = Math.round(((performance.now() - start) / 1000) * api.SPEED);
    clock.textContent = `${sec}s`; tide(Math.min(0.94, sec / eta)); ledger(null, null, null, { hero: ["Crossing, elapsed", [String(sec), h("small", {}, "s")]], strip: ["Crossing, elapsed", `${sec}s`] }); }, 250);
  for (;;) {
    try {
      const s = (S.status = await api.status(S.quote.depositAddress));
      setCrossing(s.status === "SUCCESS" && !s.landed ? "bridge" : PHASE_OF[s.status] || "send");
      put(links, ...[
        s.solanaTx && [h("span", {}, "Solana "), link(EXPL.sol(s.solanaTx), short(s.solanaTx, 6))],
        s.nearTx && [h("span", {}, "NEAR "), link(EXPL.near(s.nearTx), short(s.nearTx, 6))],
        s.zcashTx && [h("span", {}, "Zcash "), link(EXPL.zec(s.zcashTx), short(s.zcashTx, 6))],
        [link(EXPL.status(S.quote.depositAddress), "1Click status (public)")],
      ].filter(Boolean).map((x) => h("span", {}, ...x.flat())));
      if (s.terminal) {
        clearInterval(tick);
        if (s.status !== "SUCCESS") { put(note, err(new Error(`1Click reports ${s.status}${s.refundReason ? `: ${s.refundReason}` : ""}. Refunds return to your Phantom wallet.`))); return; }
        if (MODE === "replay") { clock.textContent = `${(await api.loadReplay()).seconds}s`; cap.textContent = "send to land, recorded on mainnet"; }
        else cap.textContent = "send to land";
        ledger("elapsed", "Elapsed", clock.textContent);
        setCrossing("landed");
        put(now, "Landed on Zcash."); now.className = "receipt";
        setStep("s-land", "done", "landed");
        readbackLoop(); exposureLoad(); return;
      }
    } catch (e) { put(note, err(e)); }
    await sleep(LIVE ? 5000 : 1000);
  }
}

async function readbackLoop() {
  const card = $("readback");
  for (let n = 0; n < 120; n++) {
    try {
      const r = (S.rb = await api.readback(S.session.id));
      if (r.state === "UNAVAILABLE") { renderKeyReadback(card); return; }
      if (r.state === "LANDED") { renderReadback(r); return; }
    } catch (e) { /* keep polling */ }
    await sleep(LIVE ? 5000 : 800);
  }
}

// Your own address: open the note in your wallet, or paste that wallet's viewing key for a one-time read-back here.
function renderKeyReadback(card) {
  const input = h("input", { type: "text", id: "ufvk", autocomplete: "off", spellcheck: "false", placeholder: "uview1… or uvf1…" });
  const msg = h("p", { class: "help", "data-testid": "ufvk-msg" });
  const go = h("button", { class: "btn", type: "button", "data-testid": "ufvk-go", onclick: async () => {
    const k = input.value.trim();
    if (!/^(uview1|uvf1)[0-9a-z]{20,2000}$/.test(k)) { put(msg, "That is not a unified viewing key. Never paste a seed phrase or spending key."); return; }
    go.disabled = true; put(msg, "Scanning from the payout block with your viewing key…");
    try {
      for (let n = 0; n < 60; n++) {
        const r = await api.readbackKey(S.session.id, k);
        if (r.state === "LANDED") { input.value = ""; renderReadback(r); return; }
        put(msg, "Not mined yet, checking again…"); await sleep(LIVE ? 10000 : 800);
      }
    } catch (e) { put(msg, err(e)); }
    go.disabled = false;
  } }, "Read it back");
  put(card, h("h2", { class: "ch" }, "Read-back"),
    h("p", { class: "empty" }, "This went to your own wallet. Open it to see the note (landed at height ", String(S.status.landed?.height ?? "?"), "), or paste that wallet's viewing key to read it back here."),
    h("label", { class: "f" }, h("span", { class: "label" }, "Your unified viewing key"), input),
    h("p", { class: "help" }, "A viewing key can see this wallet's incoming notes but can never spend. SHORE uses it for this one scan over HTTPS and does not store or log it."),
    msg, h("div", { class: "row" }, go));
}

function renderReadback(r) {
  const card = $("readback");
  card.dataset.landed = "";
  const amt = h("p", { class: "amount", "data-testid": "rb-amount" });
  put(card,
    h("h2", { class: "ch" }, "Read back from Zcash"),
    amt,
    h("p", { class: "seal" }, "Decrypted from the note on Zcash with SHORE's viewing key. Orchard receiver, Ironwood pool (v6 transaction)."),
    kv([["Mined at height", link(EXPL.zecBlock(r.height), r.height.toLocaleString("en-US"))],
      ["Zcash txid", link(EXPL.zec(r.txid), short(r.txid, 8))],
      S.status?.solanaTx && ["Solana tx", link(EXPL.sol(S.status.solanaTx), short(S.status.solanaTx, 8))],
      S.status?.nearTx && ["NEAR tx", link(EXPL.near(S.status.nearTx), short(S.status.nearTx, 8))],
      ["Quoted", `${zec(r.quotedOutZat)} ZEC`], ["Difference", `${r.deltaZat >= 0 ? "+" : ""}${zec(r.deltaZat)} ZEC`]]),
    r.deltaZat ? h("p", { class: "note" }, `The note holds ${r.deltaZat} zat ${r.deltaZat > 0 ? "more" : "less"} than 1Click's status amountOut.`) : null,
    MODE === "replay" ? h("details", { class: "fold" }, h("summary", {}, "Source"), h("p", { class: "help" }, "Replay of the recorded mainnet run, 2026-10-10 09:22 IST, file shore-live-run.json.")) : null);
  countUp(amt, r.valueZat, h("small", {}, "ZEC"));
  const hv = h("span"); countUp(hv, r.valueZat, "");
  ledger(null, null, null, { hero: ["Shielded, read back from Zcash", [hv, h("small", {}, "ZEC")]], tone: "shielded", strip: ["Shielded", `${zec(r.valueZat)} ZEC`] });
  if (matchMedia("(max-width: 959px)").matches) card.scrollIntoView({ behavior: REDUCED ? "auto" : "smooth", block: "center" });
}

// ---------- what's still public ----------
const DEFAULT_EXPOSED = [
  ["The hop in is public.", "Your Phantom address, the deposit address, the amount and the time are on Solana for anyone to read."],
  ["1Click status links your Phantom address to your u1.", "Anyone holding the deposit address can call the unauthenticated /v0/status and read the recipient. That is why every shield gets a fresh u1."],
  ["NEAR can decrypt the payout.", "The NEAR Intents solver builds the Zcash transaction, so it holds the outgoing view key for that note. The amount entering the pool is also visible, because the delivery has transparent inputs."],
  ["What is private afterwards:", "spends you make later from the note. Nobody outside your wallet can see those."],
];
function renderExposureDefault() {
  put($("pub-n"), `${DEFAULT_EXPOSED.length} in general`);
  put($("exposed"), DEFAULT_EXPOSED.map(([b, t], i) => h("li", {}, i < 3 ? h("b", {}, b) : h("strong", {}, b), " ", t)));
}
async function exposureLoad() {
  try {
    const x = (S.exposure = await api.exposure(S.session.id, { sender: S.wallet.address, deposit: S.quote.depositAddress, recipient: S.session.recipient }));
    put($("pub-n"), `${x.rows.length} for this transfer`);
    put($("exposed"), x.rows.map((r) => h("li", { "data-exposed": String(r.exposed), "data-who": /anyone/i.test(r.whoCanSee) ? "anyone" : /nobody/i.test(r.whoCanSee) ? "nobody" : "some" },
      r.exposed ? h("b", {}, r.what) : h("strong", {}, r.what), r.value ? h("span", { class: "help" }, /^\d+$/.test(r.value) ? `${zec(r.value)} ZEC` : short(r.value, 10)) : null,
      h("span", { class: "help" + (/anyone/i.test(r.whoCanSee) ? " warn" : "") }, `${r.exposed ? "Readable by" : "Visible to"}: ${r.whoCanSee}`))),
      x.addressReused ? h("li", {}, h("b", {}, "This u1 was used before."), " Earlier shields are now linkable to this one.") : null);
  } catch { /* keep default text */ }
}

// ---------- boot ----------
function renderInvite() {
  if (!INVITE) return;
  const el = $("invite"); el.hidden = false;
  if (INVITE.error) { put(el, INVITE.error); return; }
  const judge = MODE === "replay";
  put(el, h("b", {}, INVITE.code ? `Invite ${INVITE.code}` : "Invite link"),
    INVITE.codeIgnored ? [h("br"), "The invite code was not recognized, so it is not shown."] : null,
    INVITE.note ? [h("br"), "The sender wrote: ", h("q", { "data-testid": "invite-note" }, INVITE.note)] : null,
    INVITE.amount ? [h("br"), "Suggested amount: ", h("b", { "data-testid": "invite-amount" }, `${INVITE.amount} ZEC`), judge ? ". The replay uses the recorded 0.0014 ZEC." : ". You can change it."] : null,
    INVITE.amountNote ? [h("br"), h("span", { "data-testid": "invite-amount-note" }, INVITE.amountNote)] : null,
    judge ? [h("br"), h("b", { "data-testid": "invite-judge" }, "Judge preview: this plays the recorded mainnet run. No wallet, nothing sent.")] : null,
    h("br"), h("span", { class: "help" }, "SHORE did not write this note and never holds funds. Your ZEC goes only to the Zcash address you paste."));
}
function boot() {
  renderInvite();
  const m = $("mode");
  m.dataset.mode = MODE;
  m.textContent = MODE === "replay" ? "Replay of a mainnet run" : MODE === "mock" ? "Mock data, not real" : "Live on mainnet";
  renderExposureDefault();
  setCrossing("idle");
  put(body("s-connect"),
    h("div", { class: "row" }, h("button", { class: "btn", type: "button", "data-testid": "connect", onclick: connect }, MODE === "live" ? "Connect Phantom" : "Connect recorded wallet"),
      MODE === "live" ? h("a", { class: "btn ghost", href: "?replay=1" }, "Watch the recorded run") : h("a", { class: "btn ghost", href: "?live=1" }, "Use it live (mainnet)")));
}
// ---------- judge tour ----------
// ?tour=1 (replay only): presses each step's primary button after a readable pause, then opens "What's still public".
export const TOUR_ORDER = ['[data-testid="connect"]', '[data-testid="fresh"]', '#s-quote .btn:not(.ghost)', '[data-testid="approve"]'];
function tour(pause = 2200) {
  if (MODE !== "replay" || !(new URLSearchParams(location.search).has("tour") || INVITE)) return;
  document.documentElement.dataset.tour = "";
  let i = 0;
  const tick = () => {
    if (i < TOUR_ORDER.length) {
      const b = document.querySelector(TOUR_ORDER[i]);
      if (b && !b.disabled && b.offsetParent) { b.scrollIntoView({ block: "center", behavior: "smooth" }); b.click(); i++; }
      return setTimeout(tick, i === TOUR_ORDER.length ? 600 : pause);
    }
    if ($("readback").hasAttribute("data-landed")) {
      setTimeout(() => { const d = $("public"); d.open = true; d.scrollIntoView({ block: "start", behavior: "smooth" }); document.documentElement.dataset.tour = "done"; }, pause * 1.5);
      return;
    }
    setTimeout(tick, 500);
  };
  setTimeout(tick, pause);
}
boot();
setTimeout(() => document.documentElement.classList.add("booted"), 900);
tour();
