const API = String(window.AIRLOCK_API || "").replace(/\/+$/, "");
const TERMINAL = new Set(["SUCCESS", "REFUNDED", "FAILED"]);
const SWAPS_KEY = "airlock.swaps";
const INBOUND_KEY = "airlock.inbound";

/* browser-held swaps and every random choice: crypto only, never Math.random */
function readList(key) {
  try {
    const a = JSON.parse(localStorage.getItem(key));
    return Array.isArray(a) ? a : [];
  } catch {
    return [];
  }
}
const writeList = (key, list) => localStorage.setItem(key, JSON.stringify(list));
const removeItem = (key, id) => writeList(key, readList(key).filter((x) => x.id !== id));

function randInt(lo, hi) {
  const b = new Uint32Array(1);
  crypto.getRandomValues(b);
  return lo + Math.floor((b[0] / 2 ** 32) * (hi - lo + 1));
}

function randHex(bytes) {
  return [...crypto.getRandomValues(new Uint8Array(bytes))].map((x) => x.toString(16).padStart(2, "0")).join("");
}

function shuffle(a) {
  const r = [...a];
  for (let i = r.length - 1; i > 0; i--) {
    const j = randInt(0, i);
    [r[i], r[j]] = [r[j], r[i]];
  }
  return r;
}

/* ---------- helpers ---------- */

function h(tag, attrs = {}, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (v == null || v === false) continue;
    if (k === "class") el.className = v;
    else if (k.startsWith("on")) el.addEventListener(k.slice(2), v);
    else el.setAttribute(k, v === true ? "" : v);
  }
  for (const c of kids.flat(Infinity)) {
    if (c == null || c === false) continue;
    el.append(c.nodeType ? c : document.createTextNode(String(c)));
  }
  return el;
}

function put(el, ...kids) {
  el.replaceChildren(...kids.flat(Infinity).filter((c) => c != null && c !== false).map((c) => (c.nodeType ? c : document.createTextNode(String(c)))));
}

function formatZec(zat) {
  const whole = Math.floor(zat / 1e8);
  const frac = String(zat % 1e8).padStart(8, "0").replace(/0+$/, "");
  return frac ? `${whole}.${frac}` : String(whole);
}

function fmtTime(iso) {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return String(iso);
  return d.toISOString().replace("T", " ").slice(0, 16) + " UTC";
}

function clock(ms) {
  const d = new Date(ms);
  const hm = d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", hour12: false });
  return d.toDateString() === new Date().toDateString() ? hm : `${d.toLocaleDateString([], { month: "short", day: "numeric" })} ${hm}`;
}

function countdown(ms) {
  const s = Math.max(0, Math.ceil(ms / 1000));
  const hh = Math.floor(s / 3600);
  const mm = Math.floor((s % 3600) / 60);
  const ss = s % 60;
  return hh ? `${hh}h ${String(mm).padStart(2, "0")}m` : mm ? `${mm}m ${String(ss).padStart(2, "0")}s` : `${ss}s`;
}

class ApiError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

async function api(path, opts = {}) {
  let res;
  try {
    res = await fetch(API + path, opts);
  } catch (e) {
    if (e.name === "AbortError") throw e;
    throw new ApiError(0, `Could not reach the AIRLOCK service at ${API}.`);
  }
  let body = null;
  try {
    body = await res.json();
  } catch {
    body = null;
  }
  if (!res.ok) {
    throw new ApiError(res.status, (body && body.error) || `The service answered ${res.status}.`);
  }
  return body;
}

function errorNote(e) {
  const title =
    e.status === 404 ? "No swap found" :
    e.status === 400 ? "Not accepted" :
    e.status === 0 ? "Service unreachable" : "Request failed";
  return h("div", { class: "note", role: "alert" },
    h("p", { class: "label" }, title),
    h("p", { class: "data" }, e.message));
}

function waitNote(text, withTimer = true) {
  const tick = h("span", { class: "tick" }, "0s");
  const t0 = Date.now();
  const note = h("p", { class: "wait", role: "status" }, text, withTimer ? [" ", tick] : null);
  if (withTimer) {
    const id = setInterval(() => {
      if (!note.isConnected) return clearInterval(id);
      tick.textContent = `${Math.round((Date.now() - t0) / 1000)}s`;
    }, 1000);
  }
  return note;
}

/* receipt rows: key, dotted leader, value. Long values go under the key. */
function row(key, value, opt = {}) {
  const text = String(value);
  const val = opt.hl ? h("span", { class: "hl", style: `--i:${opt.i || 0}` }, text) : text;
  const long = text.length > 22;
  if (long) {
    return h("div", { class: "row long" },
      h("div", { class: "top" }, h("span", { class: "k" }, key), h("span", { class: "lead", "aria-hidden": "true" })),
      h("span", { class: "v" }, val));
  }
  return h("div", { class: "row" },
    h("span", { class: "k" }, key),
    h("span", { class: "lead", "aria-hidden": "true" }),
    h("span", { class: "v" }, val));
}

function hl(text, i) {
  return h("span", { class: "hl", style: `--i:${i}` }, text);
}

function copyButton(text) {
  const live = h("span", { class: "help", role: "status" });
  const btn = h("button", {
    type: "button", class: "btn-text", "aria-label": `Copy ${text}`,
    onclick: async () => {
      try {
        await navigator.clipboard.writeText(text);
        btn.textContent = "Copied";
      } catch {
        btn.textContent = "Select to copy";
      }
      setTimeout(() => { btn.textContent = "Copy"; }, 1800);
    },
  }, "Copy");
  return [btn, live];
}

/* ---------- page: exposure check ---------- */

function idleReceipt() {
  return h("article", { class: "receipt idle", "aria-label": "Receipt preview" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("h2", { class: "r-title" }, "Prints here"),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Header"),
      h("p", {}, "The swap status and the time 1Click last updated it.")),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Readable by anyone"),
      h("p", {}, "Every field the status endpoint returns with no key, marked with the highlighter.")),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Observers"),
      h("p", {}, "What the chain, the status endpoint and the solver each see.")));
}

/* The quote profile is public in /v0/status: this row says how many sampled swaps share it. */
function crowdRow(d) {
  if (!d.profileCrowd) return null;
  const { matches, sample } = d.profileCrowd;
  return row("Quote profile", `shared by ${matches} of ${sample} sampled swaps`);
}

function inspectReceipt(d) {
  let i = 0;
  const zec = d.amountZat == null ? null : `${formatZec(d.amountZat)} ZEC`;
  const fields = d.publicOnStatus.map((f) => row(f.field, f.value, { hl: true, i: i++ }));
  const metaRows = zec == null ? [] : [
    row("Amount", zec),
    row("Denomination", d.amountCanonical ? "ZIP 318 canonical" : "not a ZIP 318 canonical amount"),
  ];
  const crowd = crowdRow(d);
  if (crowd) metaRows.push(crowd);
  if (d.referral) metaRows.push(row("Referral", d.referral));
  const chainI = i;
  const nTags = d.publicOnStatus.length;
  return h("article", { class: "receipt", "aria-labelledby": "r-h" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("h2", { class: "r-title", id: "r-h", tabindex: "-1" }, "What a stranger reads"),
    h("div", { class: "r-sec" }, row("Status", d.status), row("Updated", fmtTime(d.updatedAt))),
    h("div", { class: "r-sec" }, h("p", { class: "label" }, "Readable by anyone"), fields),
    h("div", { class: "r-sec" }, metaRows),
    observerSecs(
      zec == null
        ? h("p", {}, "Sees the deposit address ", hl(d.depositAddress, chainI), " and what is sent to it. Nothing else.")
        : h("p", {}, "Sees the t-address ", hl(d.depositAddress, chainI), " and the ", hl(zec, chainI + 1), " sent to it. Nothing else."),
      h("p", {}, "Returns all ", String(nTags), " fields above to anyone who holds the deposit address. No key needed."),
      [row("From", d.originAsset), row("To", d.destinationAsset)]),
    h("div", { class: "r-foot" },
      h("p", {}, "Each part carries its own deposit address, refund address and destination, so no field repeats across the parts."),
      h("p", {}, h("a", { class: "link", href: "#/split" }, "Plan an exit →")),
      h("p", { class: "src" }, `GET /api/inspect/${d.depositAddress}`)));
}

function pageCheck(main) {
  const out = h("div", { class: "hero-out" }, idleReceipt());
  const input = h("input", { id: "addr", name: "addr", type: "text", autocomplete: "off", spellcheck: "false", autocapitalize: "off", "aria-describedby": "addr-help" });
  const btn = h("button", { class: "btn", type: "submit" }, "Print the receipt ", h("span", { "aria-hidden": "true" }, "→"));
  let ctl = null;

  async function run(raw) {
    const addr = raw.trim();
    if (ctl) ctl.abort();
    if (!addr) {
      put(out, errorNote(new ApiError(400, "Paste a deposit address first.")));
      input.focus();
      return;
    }
    ctl = new AbortController();
    btn.disabled = true;
    put(out, waitNote("Reading the status for this address.", false));
    try {
      const d = await api(`/api/inspect/${encodeURIComponent(addr)}`, { signal: ctl.signal });
      put(out, inspectReceipt(d));
      if (window.matchMedia("(max-width: 900px)").matches) out.scrollIntoView({ block: "start" });
    } catch (e) {
      if (e.name === "AbortError") return;
      put(out, errorNote(e));
    } finally {
      btn.disabled = false;
    }
  }

  put(main, 
    h("section", { class: "hero", "aria-labelledby": "home-h" },
      h("div", { class: "hero-copy" },
        h("h1", { class: "display", id: "home-h", tabindex: "-1" }, "What a stranger can read."),
        h("p", { class: "lede" }, "Paste a one-time deposit address. AIRLOCK shows every field 1Click exposes on it to anyone, no key needed."),
        h("form", { novalidate: true, onsubmit: (e) => { e.preventDefault(); run(input.value); } },
          h("label", { class: "field", for: "addr" }, h("span", { class: "label" }, "Paste a one-time deposit address"), input),
          h("p", { class: "help", id: "addr-help", style: "margin-top:8px" }, "Reads the same unauthenticated 1Click status call a stranger would make."),
          h("div", { class: "form-row" }, btn))),
      out),
    h("section", { class: "how", "aria-labelledby": "how-h" },
      h("h2", { class: "heading", id: "how-h" }, "How the receipt is made"),
      h("ol", { class: "how-list" },
        h("li", {}, h("span", { class: "n", "aria-hidden": "true" }, "a."), h("p", {}, "You paste the deposit address of any 1Click swap. It is public on the Zcash chain the moment funds move.")),
        h("li", {}, h("span", { class: "n", "aria-hidden": "true" }, "b."), h("p", {}, "AIRLOCK reads the status 1Click serves with no key and marks each field a stranger can read.")),
        h("li", {}, h("span", { class: "n", "aria-hidden": "true" }, "c."), h("p", {}, "AIRLOCK exits split into ZIP 318 denominations. Every part carries its own deposit address, refund address and destination.")))));
  return () => { if (ctl) ctl.abort(); };
}

/* ---------- page: split and create ---------- */

function groupTokens(tokens) {
  const by = new Map();
  for (const t of tokens) {
    if (!by.has(t.blockchain)) by.set(t.blockchain, []);
    by.get(t.blockchain).push(t);
  }
  return [...by.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([chain, list]) => [chain, list.sort((a, b) => a.symbol.localeCompare(b.symbol))]);
}

let tokensCache = null;
const getTokens = () => (tokensCache ||= api("/api/tokens").catch((e) => { tokensCache = null; throw e; }));

async function loadAssets(sel, prompt) {
  try {
    const tokens = await getTokens();
    put(sel,
      h("option", { value: "" }, prompt),
      groupTokens(tokens).map(([chain, list]) =>
        h("optgroup", { label: chain.toUpperCase() },
          list.map((t) => h("option", { value: t.assetId }, list.filter((u) => u.symbol === t.symbol).length > 1 ? `${t.symbol} on ${chain}, ${t.assetId.slice(-8)}` : `${t.symbol} on ${chain}`)))));
    sel.disabled = false;
  } catch (e) {
    put(sel, h("option", { value: "" }, "Assets unavailable"));
    throw e;
  }
}

function formatUnits(v, dec) {
  const s = String(v).padStart(dec + 1, "0");
  const whole = s.slice(0, s.length - dec);
  const frac = s.slice(s.length - dec).replace(/0+$/, "");
  return frac ? `${whole}.${frac}` : whole;
}

function minutes(blocks) {
  return Math.round((blocks * 75) / 60);
}

function chambers(plan) {
  const parts = plan.parts;
  const cells = parts.map((p, i) =>
    h("li", { class: "chamber", style: `flex-grow:${p.zat};--i:${i}`, title: `${formatZec(p.zat)} ZEC` },
      h("span", { class: "c-amt" }, formatZec(p.zat)),
      h("span", { class: "fill", "aria-hidden": "true" })));
  if (plan.residualZat > 0) {
    cells.push(h("li", { class: "chamber residual", style: `flex-grow:${plan.residualZat}`, title: `${formatZec(plan.residualZat)} ZEC stays shielded` },
      h("span", { class: "c-amt" }, formatZec(plan.residualZat))));
  }
  return h("ol", { class: "chambers", "aria-label": "Parts of the exit, widths proportional to amount" }, cells);
}

/* spacing between part quotes: [min, max] in ms, drawn per gap with crypto.getRandomValues */
const SPACINGS = [
  { key: "hours", label: "Spread over hours (recommended)", lo: 5 * 60e3, hi: 45 * 60e3 },
  { key: "hour", label: "Within the hour", lo: 2 * 60e3, hi: 10 * 60e3 },
  { key: "seconds", label: "Seconds (testing)", lo: 4e3, hi: 25e3 },
];

function pageSplit(main) {
  const state = { amount: "", cap: "", asset: "", plan: null, dest: [], refund: [] };
  let planCtl = null;
  let planSeq = 0;
  let debounce = null;
  let createCtl = null;

  const amountIn = h("input", { id: "amount", name: "amount", type: "text", inputmode: "decimal", autocomplete: "off", required: true });
  const capIn = h("input", { id: "cap", name: "cap", type: "text", inputmode: "decimal", autocomplete: "off", required: true });
  const assetSel = h("select", { id: "asset", name: "asset", required: true, disabled: true }, h("option", { value: "" }, "Loading assets"));
  const spacingSel = h("select", { id: "spacing", name: "spacing" },
    SPACINGS.map((o, i) => h("option", { value: o.key, selected: i === 0 }, o.label)));
  const preview = h("div", { "aria-live": "polite" });
  const partsBox = h("div", { class: "part-fields" });
  const status = h("div", { "aria-live": "polite" });
  const submit = h("button", { class: "btn", type: "submit" }, "Create swap ", h("span", { "aria-hidden": "true" }, "→"));
  let lastPlanKey = "";

  loadAssets(assetSel, "Choose a destination asset").catch((e) => put(status, errorNote(e)));

  function renderParts(plan) {
    state.dest.length = plan.parts.length;
    state.refund.length = plan.parts.length;
    put(partsBox, plan.parts.length === 0 ? "" : plan.parts.map((p, i) =>
      h("fieldset", {},
        h("legend", {}, `Part ${i + 1}`, h("small", {}, `${formatZec(p.zat)} ZEC`, Number.isFinite(p.sendAtBlocks) ? `, send at about +${minutes(p.sendAtBlocks)} min` : "")),
        h("label", { class: "field", for: `dest-${i}` }, h("span", { class: "label" }, "Destination address on the other chain"),
          h("input", { id: `dest-${i}`, type: "text", autocomplete: "off", spellcheck: "false", required: true, value: state.dest[i] || "",
            oninput: (e) => { state.dest[i] = e.target.value; } })),
        h("label", { class: "field", for: `refund-${i}` }, h("span", { class: "label" }, "Refund address, Orchard-only unified address from your wallet"),
          h("input", { id: `refund-${i}`, type: "text", autocomplete: "off", spellcheck: "false", required: true, value: state.refund[i] || "",
            oninput: (e) => { state.refund[i] = e.target.value; } })))));
  }

  function renderPlan(plan) {
    const parts = plan.parts;
    put(preview, 
      chambers(plan),
      h("ul", { class: "plan-list" },
        parts.map((p, i) => h("li", {}, h("span", {}, `Part ${i + 1}, ${formatZec(p.zat)} ZEC`), h("span", {}, p.canonical ? "ZIP 318 canonical" : "not canonical"))),
        plan.residualZat > 0 ? h("li", {}, h("span", {}, `${formatZec(plan.residualZat)} ZEC`), h("span", {}, "stays shielded")) : null));
  }

  async function replan() {
    state.amount = amountIn.value.trim();
    state.cap = capIn.value.trim();
    if (planCtl) planCtl.abort();
    if (!state.amount || !state.cap) {
      state.plan = null; lastPlanKey = "";
      put(preview, h("p", { class: "wait" }, "Enter an amount and a per-part cap to see the parts."));
      put(partsBox, );
      return;
    }
    planCtl = new AbortController();
    const seq = ++planSeq;
    try {
      const plan = await api("/api/plan", {
        method: "POST", headers: { "content-type": "application/json" },
        body: JSON.stringify({ amountZec: state.amount, capZec: state.cap }), signal: planCtl.signal,
      });
      if (seq !== planSeq) return;
      state.plan = plan;
      const key = JSON.stringify(plan);
      if (key !== lastPlanKey) { lastPlanKey = key; renderPlan(plan); renderParts(plan); }
    } catch (e) {
      if (e.name === "AbortError" || seq !== planSeq) return;
      state.plan = null; lastPlanKey = "";
      put(preview, errorNote(e));
      put(partsBox, );
    }
  }
  const schedule = () => { clearTimeout(debounce); debounce = setTimeout(replan, 300); };
  amountIn.addEventListener("input", schedule);
  capIn.addEventListener("input", schedule);
  put(preview, h("p", { class: "wait" }, "Enter an amount and a per-part cap to see the parts."));

  async function create(e) {
    e.preventDefault();
    const form = e.currentTarget;
    if (!form.reportValidity()) return;
    if (!state.plan || !state.plan.parts.length) {
      put(status, errorNote(new ApiError(400, "The plan has no parts to create.")));
      return;
    }
    const plan = state.plan;
    const asset = assetSel.value;
    const dest = state.dest.map((v) => (v || "").trim());
    const refund = state.refund.map((v) => (v || "").trim());
    const spacing = SPACINGS.find((o) => o.key === spacingSel.value) || SPACINGS[0];
    submit.disabled = true;
    createCtl = new AbortController();
    const signal = createCtl.signal;
    const order = shuffle(plan.parts.map((_, i) => i));
    const first = order[0];
    const line = h("p", { class: "wait", role: "status" });
    put(status, h("div", { class: "note" },
      h("p", { class: "label" }, "Creating the swap"),
      h("p", {}, "Only the first part is quoted now, over its own Tor circuit. Every other part is quoted later, at its own time."),
      line));
    const keep = (ev) => { ev.preventDefault(); ev.returnValue = ""; };
    window.addEventListener("beforeunload", keep);
    try {
      let made;
      for (;;) {
        put(line, `Part ${first + 1} of ${plan.parts.length}: opening its own circuit and quote. Usually 5 to 15 s.`);
        try {
          made = await api("/api/quote", {
            method: "POST", headers: { "content-type": "application/json" }, signal,
            body: JSON.stringify({ amountZat: plan.parts[first].zat, destinationAsset: asset, destination: dest[first], refund: refund[first] }),
          });
          break;
        } catch (err) {
          if (err.name === "AbortError") return;
          await new Promise((resolve) => {
            put(line, h("span", { role: "alert" }, `failed: ${err.message} `),
              h("button", { type: "button", class: "btn-text", onclick: resolve }, `Retry part ${first + 1}`));
            signal.addEventListener("abort", resolve);
          });
          if (signal.aborted) return;
        }
      }
      let at = Date.now();
      const parts = plan.parts.map((p, i) => ({ amountZat: p.zat, destination: dest[i], refund: refund[i], pending: true, quoteAt: 0 }));
      parts[first] = { amountZat: made.amountZat, depositAddress: made.depositAddress, deadline: made.deadline, amountOut: made.amountOutFormatted, minOut: made.minAmountOutFormatted, destination: made.destination, refund: made.refund };
      for (const i of order.slice(1)) {
        at += randInt(spacing.lo, spacing.hi);
        parts[i].quoteAt = at;
      }
      const swap = { id: randHex(6), createdAt: new Date().toISOString(), destinationAsset: asset, parts };
      try {
        writeList(SWAPS_KEY, [...readList(SWAPS_KEY), swap]);
      } catch {
        put(status, h("div", { class: "note", role: "alert" },
          h("p", { class: "label" }, "Not saved"),
          h("p", {}, "This browser would not store the swap, so the later parts cannot be quoted. Copy this deposit address now."),
          h("p", { class: "data" }, `Part ${first + 1}, ${formatZec(made.amountZat)} ZEC: ${made.depositAddress}`)));
        return;
      }
      location.hash = `#/swap/${swap.id}`;
    } finally {
      window.removeEventListener("beforeunload", keep);
      if (!signal.aborted && !location.hash.startsWith("#/swap/")) submit.disabled = false;
    }
  }

  assetSel.addEventListener("change", () => { state.asset = assetSel.value; });

  put(main, 
    h("section", { class: "page", "aria-labelledby": "split-h" },
      h("h1", { class: "display", id: "split-h", tabindex: "-1" }, "Split the exit."),
      h("p", { class: "lede", style: "margin-top:20px" }, "Choose an amount and a cap. Every part gets its own deposit address, refund address and destination."),
      h("form", { class: "page-grid single", novalidate: true, onsubmit: create },
        h("div", {},
          h("div", { class: "two" },
            h("label", { class: "field", for: "amount" }, h("span", { class: "label" }, "Amount in ZEC, at most 0.02"), amountIn),
            h("label", { class: "field", for: "cap" }, h("span", { class: "label" }, "Per-part cap in ZEC, 0.01 (the ZIP 318 floor)"), capIn)),
          h("label", { class: "field", for: "asset" }, h("span", { class: "label" }, "Destination asset"), h("div", { class: "select-wrap" }, assetSel)),
          h("label", { class: "field", for: "spacing" }, h("span", { class: "label" }, "Spacing between part quotes"), h("div", { class: "select-wrap" }, spacingSel)),
          h("div", { class: "field" }, h("span", { class: "label" }, "Parts"), preview),
          partsBox,
          h("div", { class: "form-row" }, submit),
          status))));
  return () => { clearTimeout(debounce); if (planCtl) planCtl.abort(); if (createCtl) createCtl.abort(); };
}

/* ---------- page: swap receipt ---------- */

/* An identifier is whatever a stranger can match on: the fields 1Click serves with no key, plus the
   deposit address, refund and destination this browser submitted for the part. 0x values compare
   without case because the status endpoint lowercases them. */
function identifiersOf(part, d) {
  const seen = new Map();
  const add = (field, value) => {
    if (!value) return;
    const norm = /^0x/i.test(value) ? value.toLowerCase() : value;
    seen.set(`${field}\u0000${norm}`, { field, value });
  };
  add("deposit_address", part.depositAddress);
  add("refund_to", part.refund);
  add("recipient", part.destination);
  if (d) for (const f of d.publicOnStatus) add(f.field, f.value);
  return [...seen.entries()].map(([key, v]) => ({ key, ...v }));
}

function sharedAcross(swap, reads) {
  const by = new Map();
  swap.parts.forEach((p, k) => {
    if (p.pending) return;
    for (const id of identifiersOf(p, reads[k].d)) {
      if (!by.has(id.key)) by.set(id.key, { field: id.field, value: id.value, parts: [] });
      by.get(id.key).parts.push(k);
    }
  });
  return [...by.values()].filter((s) => s.parts.length > 1);
}

function observerSecs(chain, statusNote, solverExtra) {
  return [
    h("div", { class: "r-sec" }, h("p", { class: "label" }, "Chain observer"), chain),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "1Click status endpoint"),
      statusNote,
      h("p", { class: "data" }, "GET /v0/status?depositAddress=")),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Solver"),
      h("p", {}, "Sees the full quote, including both assets, the amount, the recipient and the refund address."),
      solverExtra),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "AIRLOCK service"),
      h("p", {}, "Sees each request you send it: your network address and the addresses in that request, so it and its host can tell that the parts of one swap came from the same browser. Stores nothing. Run it yourself with ",
        h("code", {}, "airlock serve"), " and point ", h("code", {}, "web/config.js"), " at it to remove it from the picture.")),
  ];
}

function partsList(swap, reads, q) {
  return h("ol", { class: "parts" }, swap.parts.map((p, k) => {
    const r = reads[k];
    if (p.pending) {
      return h("li", {},
        h("span", { class: "n", "aria-hidden": "true" }, String(k + 1)),
        h("div", {},
          h("p", { class: "amt" }, `Part ${k + 1}, ${formatZec(p.amountZat)} ZEC`),
          h("p", { class: "meta" },
            h("span", { class: "status" }, q.busy === k ? "asking 1Click" : "quote pending"),
            h("span", {}, `Quote opens at ${clock(p.quoteAt)} local`),
            h("span", { class: "cd", "data-k": String(k) })),
          q.errs[k] ? h("p", { class: "help", role: "alert" }, `Last attempt failed: ${q.errs[k]}`) : null));
    }
    return h("li", {},
      h("span", { class: "n", "aria-hidden": "true" }, String(k + 1)),
      h("div", {},
        h("p", { class: "amt" }, `Part ${k + 1}, send ${formatZec(p.amountZat)} ZEC`),
        h("p", { class: "meta" },
          h("span", { class: "status" }, r.d ? r.d.status : "reading status"),
          h("span", {}, `deposit before ${fmtTime(p.deadline)}`),
          p.amountOut ? h("span", {}, `1Click expects to deliver about ${p.amountOut}${p.minOut ? `, at least ${p.minOut}` : ""}`) : null),
        h("div", { class: "addr" }, h("code", {}, p.depositAddress), copyButton(p.depositAddress)),
        r.err ? h("p", { class: "help", role: "alert" }, `Last read failed: ${r.err.message}`) : null));
  }));
}

function swapReceipt(swap, reads, shared, readAt) {
  let i = 0;
  const total = swap.parts.length;
  const quoted = swap.parts.filter((p) => !p.pending).length;
  const pending = total - quoted;
  const read = swap.parts.filter((p, k) => !p.pending && reads[k].d).length;
  const blocks = swap.parts.map((p, k) =>
    h("div", { class: "r-sec part-block" },
      h("p", { class: "label" }, `Part ${k + 1}${p.pending ? ", quote pending" : reads[k].d ? `, ${reads[k].d.status}` : ""}`),
      p.pending
        ? h("p", { class: "wait" }, `Quote opens at ${clock(p.quoteAt)} local. Nothing is public for this part yet.`)
        : reads[k].d
          ? [reads[k].d.publicOnStatus.map((f) => row(f.field, f.value, { hl: true, i: i++ })), crowdRow(reads[k].d)]
          : h("p", { class: "wait" }, "Reading the status for this part.")));
  const count = pending ? h("p", { class: "plan-note" }, `${quoted} of ${total} parts quoted`) : null;
  let linked;
  if (shared.length > 0) {
    linked = h("div", { class: "r-sec" },
      h("p", { class: "label" }, `Shared across parts: ${shared.length}`),
      shared.map((s) => h("div", {},
        row(s.field, s.value, { hl: true, i: i++ }),
        h("p", { class: "help" }, `in parts ${s.parts.map((x) => x + 1).join(", ")}`))),
      count);
  } else if (quoted >= 2 && read === quoted) {
    linked = h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Shared across parts"),
      h("div", { class: "seal-line" },
        h("div", { class: "seal", role: "img", "aria-label": "0 shared identifiers" }, h("b", {}, "0"), h("span", {}, "shared identifiers")),
        h("p", {}, `No identifier repeats across the ${quoted} parts. A stranger holding one deposit address finds no field that leads to the others; the quote profile rows above show how many other swaps look the same.`)),
      count);
  } else {
    linked = h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Shared across parts"),
      h("p", { class: "wait" }, quoted < 2 && !pending
        ? "One part only, so there is nothing to compare."
        : quoted < 2 ? "The count prints once at least 2 parts are quoted and read."
        : `Read ${read} of ${quoted} quoted parts. The count prints once each has been read.`),
      count);
  }
  return h("article", { class: "receipt", "aria-labelledby": "r-h" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("h2", { class: "r-title", id: "r-h", tabindex: "-1" }, "What a stranger reads"),
    blocks, linked,
    observerSecs(
      h("p", {}, "Sees each part's t-address, the ZEC sent to it, and each payout arriving at its destination."),
      h("p", {}, "Returns every highlighted field above, for each part, to anyone who holds that part's deposit address. No key needed.")),
    h("div", { class: "r-foot" },
      h("p", {}, "Not hidden: the parts' total, and the times they are sent, can still be matched by an observer who watches both ends."),
      h("p", { class: "src" }, srcLine(readAt))));
}

const srcLine = (at) => `Each part read with GET /api/inspect/{depositAddress}, one request per part${at ? `, last read ${fmtTime(at)}` : ""}`;

function missingPage(main, title, listKey, prefix, describe) {
  put(main,
    h("section", { class: "page", "aria-labelledby": "miss-h" },
      h("h1", { class: "display", id: "miss-h", tabindex: "-1" }, title),
      h("p", { class: "lede", style: "margin-top:20px" }, "Swaps live in the browser that created them. This browser holds nothing under that id."),
      savedList("Saved in this browser", readList(listKey), prefix, describe)));
}

function savedList(title, items, prefix, describe) {
  if (!items.length) return null;
  return h("div", { class: "saved" },
    h("p", { class: "label" }, title),
    h("ul", { class: "plan-list" }, items.map((s) =>
      h("li", {}, h("a", { class: "link", href: `#/${prefix}/${s.id}` }, s.id), h("span", {}, describe(s))))));
}

const describeSwap = (s) => `${fmtTime(s.createdAt)}, ${s.parts.length} part${s.parts.length === 1 ? "" : "s"}`;
const describeInbound = (s) => `${s.amountIn}, ${fmtTime(s.deadline)}`;

function deleteButton(key, id, back) {
  return h("button", { type: "button", class: "btn-text", onclick: () => {
    try { removeItem(key, id); } catch { /* storage unavailable, nothing was stored */ }
    location.hash = back;
  } }, "Delete from this browser");
}

/* One request per part, each on its own timer: staggered first reads, then every 60 s with jitter,
   until the part is terminal. */
function pollEach(order, read, timers, isDead) {
  let t = 0;
  for (const k of order) {
    const go = async () => {
      if (isDead()) return;
      const more = await read(k);
      if (more && !isDead()) timers[k] = setTimeout(go, randInt(45000, 75000));
    };
    timers[k] = setTimeout(go, t);
    t += randInt(1000, 6000);
  }
}

function pageSwap(main, id) {
  if (!id) {
    put(main,
      h("section", { class: "page", "aria-labelledby": "lookup-h" },
        h("h1", { class: "display", id: "lookup-h", tabindex: "-1" }, "Open a swap."),
        h("p", { class: "lede", style: "margin-top:20px" }, "Swaps live in the browser that created them. Pick one saved here."),
        savedList("Saved in this browser", readList(SWAPS_KEY), "swap", describeSwap) ||
          h("p", { class: "wait", style: "margin-top:24px" }, "No swap is saved in this browser. "),
        h("p", { style: "margin-top:12px" }, h("a", { class: "link", href: "#/split" }, "Plan an exit →"))));
    return null;
  }
  const swap = readList(SWAPS_KEY).find((s) => s.id === id);
  if (!swap) { missingPage(main, "Swap not found.", SWAPS_KEY, "swap", describeSwap); return null; }

  const reads = swap.parts.map(() => ({ d: null, err: null }));
  const left = h("div", {});
  const right = h("div", {});
  const timers = [];
  const ctls = [];
  /* one quote at a time: busy is the part index being quoted, nextOk spaces consecutive quotes in this session */
  const q = { busy: null, errs: {}, retryAt: {}, nextOk: 0, saveErr: false };
  let quoteCtl = null;
  let lastKey = "";
  let lastAt = null;
  let dead = false;

  function paint() {
    const key = JSON.stringify([swap.parts, reads.map((r) => [r.d, r.err && r.err.message]), q.busy, q.errs, q.saveErr]);
    if (key === lastKey) {
      const src = right.querySelector(".src");
      if (src) src.textContent = srcLine(lastAt);
      return;
    }
    lastKey = key;
    put(left, partsList(swap, reads, q), q.saveErr ? h("p", { class: "help", role: "alert" }, "This browser would not store the new quote. Copy its deposit address now.") : null);
    put(right, swapReceipt(swap, reads, sharedAcross(swap, reads), lastAt));
    tickCountdowns();
  }

  const effective = (k) => Math.max(swap.parts[k].quoteAt, q.retryAt[k] || 0, q.nextOk);

  function tickCountdowns() {
    for (const el of left.querySelectorAll(".cd")) {
      const k = Number(el.dataset.k);
      if (q.busy === k) { el.textContent = "quoting now"; continue; }
      const ms = effective(k) - Date.now();
      el.textContent = ms > 0 ? `in ${countdown(ms)}` : "due now";
    }
  }

  async function readPart(k) {
    ctls[k] = new AbortController();
    try {
      reads[k] = { d: await api(`/api/inspect/${encodeURIComponent(swap.parts[k].depositAddress)}`, { signal: ctls[k].signal }), err: null };
      lastAt = new Date();
    } catch (e) {
      if (e.name === "AbortError" || dead) return false;
      reads[k] = { d: reads[k].d, err: e };
    }
    paint();
    return !reads[k].d || !TERMINAL.has(reads[k].d.status);
  }

  async function quoteDue() {
    if (dead || q.busy !== null) return;
    const now = Date.now();
    const due = swap.parts.map((p, k) => k).filter((k) => swap.parts[k].pending && effective(k) <= now)
      .sort((a, b) => swap.parts[a].quoteAt - swap.parts[b].quoteAt);
    if (!due.length) return;
    const k = due[0];
    const p = swap.parts[k];
    q.busy = k;
    quoteCtl = new AbortController();
    paint();
    try {
      const made = await api("/api/quote", {
        method: "POST", headers: { "content-type": "application/json" }, signal: quoteCtl.signal,
        body: JSON.stringify({ amountZat: p.amountZat, destinationAsset: swap.destinationAsset, destination: p.destination, refund: p.refund }),
      });
      if (dead) return;
      swap.parts[k] = { amountZat: made.amountZat, depositAddress: made.depositAddress, deadline: made.deadline, amountOut: made.amountOutFormatted, minOut: made.minAmountOutFormatted, destination: made.destination, refund: made.refund };
      delete q.errs[k];
      q.nextOk = Date.now() + randInt(4000, 25000);
      try {
        const list = readList(SWAPS_KEY);
        if (!list.some((x) => x.id === id)) { dead = true; return; }
        writeList(SWAPS_KEY, list.map((x) => (x.id === id ? swap : x)));
      } catch {
        q.saveErr = true;
      }
      pollEach([k], readPart, timers, () => dead);
    } catch (e) {
      if (e.name === "AbortError" || dead) return;
      q.errs[k] = e.message;
      q.retryAt[k] = Date.now() + randInt(50000, 70000);
    } finally {
      q.busy = null;
      if (!dead) paint();
    }
  }

  paint();
  pollEach(shuffle(swap.parts.map((_, k) => k).filter((k) => !swap.parts[k].pending)), readPart, timers, () => dead);
  const ticker = setInterval(() => { tickCountdowns(); quoteDue(); }, 1000);
  quoteDue();

  put(main,
    h("section", { class: "page", "aria-labelledby": "swap-h" },
      h("h1", { class: "display", id: "swap-h", tabindex: "-1" }, "Swap receipt."),
      h("p", { class: "data", style: "margin-top:16px;overflow-wrap:anywhere" }, `swap ${id}, ${swap.parts.length} part${swap.parts.length === 1 ? "" : "s"}, created ${fmtTime(swap.createdAt)}`),
      h("div", { class: "page-grid" },
        h("div", {},
          h("div", { class: "steps" },
            h("p", {}, "Send each part from your own wallet to its deposit address once it is quoted."),
            h("p", {}, "Parts can be funded one at a time, in any order, before each deadline."),
            h("p", {}, "Keep this tab open. Each part is quoted at its own time so the quotes do not cluster."),
            h("p", {}, "This swap is saved only in this browser.")),
          left,
          h("div", { class: "form-row" }, deleteButton(SWAPS_KEY, id, "#/swap"))),
        right)));
  tickCountdowns();
  return () => { dead = true; clearInterval(ticker); timers.forEach(clearTimeout); ctls.forEach((c) => c && c.abort()); if (quoteCtl) quoteCtl.abort(); };
}

/* ---------- pages: inbound ---------- */

function pageInboundNew(main) {
  let ctl = null;
  const amountIn = h("input", { id: "in-amount", name: "amount", type: "text", inputmode: "decimal", autocomplete: "off", required: true });
  const assetSel = h("select", { id: "in-asset", name: "asset", required: true, disabled: true }, h("option", { value: "" }, "Loading assets"));
  const refundIn = h("input", { id: "in-refund", name: "refund", type: "text", autocomplete: "off", spellcheck: "false", required: true });
  const recipIn = h("input", { id: "in-recipient", name: "recipient", type: "text", autocomplete: "off", spellcheck: "false", required: true });
  const submit = h("button", { class: "btn", type: "submit" }, "Get a deposit address ", h("span", { "aria-hidden": "true" }, "→"));
  const status = h("div", { "aria-live": "polite" });
  loadAssets(assetSel, "Choose the asset you pay with").catch((e) => put(status, errorNote(e)));

  async function create(e) {
    e.preventDefault();
    if (!e.currentTarget.reportValidity()) return;
    submit.disabled = true;
    put(status, h("div", { class: "note", role: "status" },
      h("p", { class: "label" }, "Opening the quote"),
      h("p", {}, "The quote opens over its own Tor circuit, with an Orchard-only recipient."),
      waitNote("Elapsed")));
    ctl = new AbortController();
    try {
      const res = await api("/api/quote-in", {
        method: "POST", headers: { "content-type": "application/json" }, signal: ctl.signal,
        body: JSON.stringify({
          amountZec: amountIn.value.trim(), originAsset: assetSel.value,
          originRefund: refundIn.value.trim(), recipient: recipIn.value.trim(),
        }),
      });
      const rec = { id: randHex(6), ...res };
      try {
        writeList(INBOUND_KEY, [...readList(INBOUND_KEY), rec]);
      } catch {
        put(status, h("div", { class: "note", role: "alert" },
          h("p", { class: "label" }, "Not saved"),
          h("p", {}, "This browser would not store the swap. Copy the deposit address now."),
          h("p", { class: "data" }, res.depositAddress)));
        submit.disabled = false;
        return;
      }
      location.hash = `#/inbound/${rec.id}`;
    } catch (err) {
      if (err.name === "AbortError") return;
      put(status, errorNote(err));
      submit.disabled = false;
    }
  }

  main.replaceChildren(
    h("section", { class: "page", "aria-labelledby": "in-h" },
      h("h1", { class: "display", id: "in-h", tabindex: "-1" }, "Pay in, receive shielded."),
      h("p", { class: "lede", style: "margin-top:20px" }, "Pay from any chain. The ZEC lands in a fresh Orchard-only address of your wallet, with no unshield step."),
      h("form", { class: "page-grid single", novalidate: true, onsubmit: create },
        h("div", {},
          h("label", { class: "field", for: "in-amount" }, h("span", { class: "label" }, "ZEC to receive, at most 0.02"), amountIn),
          h("label", { class: "field", for: "in-asset" }, h("span", { class: "label" }, "Origin asset"), h("div", { class: "select-wrap" }, assetSel)),
          h("label", { class: "field", for: "in-refund" }, h("span", { class: "label" }, "Refund address on the origin chain"), refundIn),
          h("label", { class: "field", for: "in-recipient" }, h("span", { class: "label" }, "Recipient, Orchard-only unified address from your wallet"), recipIn),
          h("div", { class: "form-row" }, submit),
          status)),
      savedList("Saved in this browser", readList(INBOUND_KEY), "inbound", describeInbound)));
  return () => { if (ctl) ctl.abort(); };
}

function inboundPay(rec, d, tok) {
  const sym = tok ? tok.symbol : "";
  const min = tok ? `${formatUnits(rec.minAmountIn, tok.decimals)} ${sym}` : String(rec.minAmountIn);
  return h("div", {},
    h("ol", { class: "parts single" },
      h("li", {},
        h("div", {},
          h("p", { class: "amt" }, `Send ${rec.amountIn}${sym ? " " + sym : ""}`),
          h("p", { class: "meta" },
            h("span", { class: "status" }, d ? d.status : "reading status"),
            h("span", {}, `at least ${min}`),
            h("span", {}, `deposit before ${fmtTime(rec.deadline)}`)),
          h("div", { class: "addr" }, h("code", {}, rec.depositAddress), copyButton(rec.depositAddress))))),
    h("p", { class: "plan-note" }, `You receive ${formatZec(rec.amountOutZat)} ZEC, shielded.`));
}

function inboundReceipt(rec, d, err, readAt) {
  let i = 0;
  return h("article", { class: "receipt", "aria-labelledby": "r-h" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("h2", { class: "r-title", id: "r-h", tabindex: "-1" }, "What a stranger reads"),
    d ? h("div", { class: "r-sec" }, row("Status", d.status), row("Updated", fmtTime(d.updatedAt))) : null,
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Readable by anyone"),
      d ? d.publicOnStatus.map((f) => row(f.field, f.value, { hl: true, i: i++ })) : h("p", { class: "wait" }, "Reading the status for this address."),
      err ? h("p", { class: "help", role: "alert" }, `Last read failed: ${err.message}`) : null),
    h("div", { class: "r-sec" },
      row("Receive", `${formatZec(rec.amountOutZat)} ZEC`),
      row("Deadline", fmtTime(rec.deadline))),
    observerSecs(
      h("p", {}, "Sees the deposit address ", hl(rec.depositAddress, i), " on the origin chain and the amount sent to it. The ZEC lands in a shielded address, so the recipient does not appear on the Zcash chain."),
      h("p", {}, "Returns every highlighted field above to anyone who holds the deposit address. No key needed.")),
    h("div", { class: "r-foot" },
      h("p", {}, "Not hidden: the amount, and the time it is sent, can still be matched by an observer who watches both ends."),
      h("p", {}, "The recipient is a fresh Orchard-only address, so the destination of this swap is a shielded address."),
      h("p", { class: "src" }, `GET /api/inspect/${rec.depositAddress}${readAt ? `, read ${fmtTime(readAt)}` : ""}`)));
}

function pageInbound(main, id) {
  const rec = readList(INBOUND_KEY).find((s) => s.id === id);
  if (!rec) { missingPage(main, "Inbound swap not found.", INBOUND_KEY, "inbound", describeInbound); return null; }
  const left = h("div", {});
  const right = h("div", {});
  const state = { d: null, err: null, at: null, tok: null };
  const timers = [];
  let ctl = null;
  let lastKey = "";
  let dead = false;

  function paint() {
    const key = JSON.stringify([state.d, state.err && state.err.message, state.tok && state.tok.assetId]);
    if (key === lastKey) {
      const src = right.querySelector(".src");
      if (src) src.textContent = `GET /api/inspect/${rec.depositAddress}, read ${fmtTime(state.at)}`;
      return;
    }
    lastKey = key;
    put(left, inboundPay(rec, state.d, state.tok));
    put(right, inboundReceipt(rec, state.d, state.err, state.at));
  }

  async function readIn() {
    ctl = new AbortController();
    try {
      state.d = await api(`/api/inspect/${encodeURIComponent(rec.depositAddress)}`, { signal: ctl.signal });
      state.err = null;
      state.at = new Date();
    } catch (e) {
      if (e.name === "AbortError" || dead) return false;
      state.err = e;
    }
    paint();
    return !state.d || !TERMINAL.has(state.d.status);
  }

  paint();
  getTokens().then((t) => {
    if (dead) return;
    state.tok = t.find((x) => x.assetId === rec.originAsset) || null;
    paint();
  }).catch(() => {});
  pollEach([0], readIn, timers, () => dead);

  main.replaceChildren(
    h("section", { class: "page", "aria-labelledby": "ib-h" },
      h("h1", { class: "display", id: "ib-h", tabindex: "-1" }, "Inbound receipt."),
      h("p", { class: "data", style: "margin-top:16px;overflow-wrap:anywhere" }, `inbound ${id}, created in this browser`),
      h("div", { class: "page-grid" },
        h("div", {},
          h("div", { class: "steps" },
            h("p", {}, "Send the amount below from your own wallet to the deposit address."),
            h("p", {}, "Anything under the minimum is refunded to the address you gave.")),
          left,
          h("div", { class: "form-row" }, deleteButton(INBOUND_KEY, id, "#/inbound"))),
        right)));
  return () => { dead = true; timers.forEach(clearTimeout); if (ctl) ctl.abort(); };
}

/* ---------- router ---------- */

const main = document.getElementById("main");
let teardown = null;
let first = true;

function render() {
  if (teardown) { teardown(); teardown = null; }
  const parts = (location.hash || "#/").replace(/^#\/?/, "").split("/");
  const name = parts[0] || "check";
  let current = "check";
  if (name === "split") { teardown = pageSplit(main); current = "split"; }
  else if (name === "swap") { teardown = pageSwap(main, parts[1] ? decodeURIComponent(parts[1]) : ""); current = "swap"; }
  else if (name === "inbound") { teardown = parts[1] ? pageInbound(main, decodeURIComponent(parts[1])) : pageInboundNew(main); current = "inbound"; }
  else { teardown = pageCheck(main); }
  for (const a of document.querySelectorAll(".site-head nav a")) {
    if (a.dataset.route === current) a.setAttribute("aria-current", "page");
    else a.removeAttribute("aria-current");
  }
  if (!first) { window.scrollTo(0, 0); main.focus({ preventScroll: true }); }
  first = false;
}

window.addEventListener("hashchange", render);
render();

// Service health: the footer names the service the page talks to and whether it answers.
(async () => {
  const el = document.getElementById("svc");
  if (!el) return;
  let host = API;
  try { host = new URL(API).host; } catch {}
  try {
    const r = await fetch(API + "/api/health", { cache: "no-store" });
    if (!r.ok) throw new Error(String(r.status));
    el.textContent = `Service ${host} is reachable.`;
  } catch {
    el.textContent = `Service ${host} is not answering. Start it with airlock serve or set web/config.js.`;
  }
})();
