const API = String(window.AIRLOCK_API || "").replace(/\/+$/, "");
const TERMINAL = new Set(["SUCCESS", "REFUNDED", "FAILED"]);
const POLL_MS = 30000;

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

function inspectReceipt(d) {
  let i = 0;
  const zec = `${formatZec(d.amountZat)} ZEC`;
  const fields = d.publicOnStatus.map((f) => row(f.field, f.value, { hl: true, i: i++ }));
  const metaRows = [
    row("Amount", zec),
    row("Denomination", d.amountCanonical ? "ZIP 318 canonical" : "not a ZIP 318 canonical amount"),
  ];
  if (d.referral) metaRows.push(row("Referral", d.referral));
  const chainI = i;
  const nTags = d.publicOnStatus.length;
  return h("article", { class: "receipt", "aria-labelledby": "r-h" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("h2", { class: "r-title", id: "r-h", tabindex: "-1" }, "What a stranger reads"),
    h("div", { class: "r-sec" }, row("Status", d.status), row("Updated", fmtTime(d.updatedAt))),
    h("div", { class: "r-sec" }, h("p", { class: "label" }, "Readable by anyone"), fields),
    h("div", { class: "r-sec" }, metaRows),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Chain observer"),
      h("p", {}, "Sees the t-address ", hl(d.depositAddress, chainI), " and the ", hl(zec, chainI + 1),
        " sent to it. Nothing else.")),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "1Click status endpoint"),
      h("p", {}, "Returns all ", String(nTags), " fields above to anyone who holds the deposit address. No key needed."),
      h("p", { class: "data" }, "GET /v0/status?depositAddress=")),
    h("div", { class: "r-sec" },
      h("p", { class: "label" }, "Solver"),
      h("p", {}, "Sees the full quote, including both assets, the amount, the recipient and the refund address."),
      row("From", d.originAsset), row("To", d.destinationAsset)),
    h("div", { class: "r-foot" },
      h("p", {}, "An AIRLOCK exit gives every part its own deposit address, refund address and destination, so there is nothing to link."),
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

function pageSplit(main) {
  const state = { amount: "", cap: "", asset: "", plan: null, dest: [], refund: [] };
  let planCtl = null;
  let planSeq = 0;
  let debounce = null;
  let createCtl = null;

  const amountIn = h("input", { id: "amount", name: "amount", type: "text", inputmode: "decimal", autocomplete: "off", required: true });
  const capIn = h("input", { id: "cap", name: "cap", type: "text", inputmode: "decimal", autocomplete: "off", required: true });
  const assetSel = h("select", { id: "asset", name: "asset", required: true, disabled: true }, h("option", { value: "" }, "Loading assets"));
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
    submit.disabled = true;
    const progress = h("div", { class: "note", role: "status" },
      h("p", { class: "label" }, "Creating the swap"),
      h("p", {}, "Each part opens its own Tor circuit and asks 1Click for its own quote. The elapsed time counts below."),
      h("ul", { class: "plan-list" },
        plan.parts.map((p, i) => h("li", {}, h("span", {}, `Part ${i + 1}, ${formatZec(p.zat)} ZEC`), h("span", {}, "own circuit, own quote")))),
      waitNote("Elapsed"));
    put(status, progress);
    createCtl = new AbortController();
    try {
      const res = await api("/api/swaps", {
        method: "POST", headers: { "content-type": "application/json" }, signal: createCtl.signal,
        body: JSON.stringify({
          amountZec: state.amount, capZec: state.cap, destinationAsset: assetSel.value,
          destinations: state.dest.map((s) => s.trim()), refunds: state.refund.map((s) => s.trim()),
        }),
      });
      try {
        sessionStorage.setItem(`airlock.swap.${res.swapId}`, JSON.stringify(res));
        sessionStorage.setItem("airlock.lastSwap", res.swapId);
      } catch { /* storage unavailable, the receipt page reads the service */ }
      location.hash = `#/swap/${res.swapId}`;
    } catch (err) {
      if (err.name === "AbortError") return;
      put(status, errorNote(err));
      submit.disabled = false;
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
            h("label", { class: "field", for: "amount" }, h("span", { class: "label" }, "Amount in ZEC"), amountIn),
            h("label", { class: "field", for: "cap" }, h("span", { class: "label" }, "Per-part cap in ZEC"), capIn)),
          h("label", { class: "field", for: "asset" }, h("span", { class: "label" }, "Destination asset"), h("div", { class: "select-wrap" }, assetSel)),
          h("div", { class: "field" }, h("span", { class: "label" }, "Parts"), preview),
          partsBox,
          h("div", { class: "form-row" }, submit),
          status))));
  return () => { clearTimeout(debounce); if (planCtl) planCtl.abort(); if (createCtl) createCtl.abort(); };
}

/* ---------- page: swap receipt ---------- */

function partsList(parts, residual) {
  return h("div", {},
    h("ol", { class: "parts" }, parts.map((p, k) => {
      const n = (p.index ?? k) + 1;
      return h("li", {},
        h("span", { class: "n", "aria-hidden": "true" }, String(n)),
        h("div", {},
          h("p", { class: "amt" }, `Part ${n}, send ${formatZec(p.amountZat)} ZEC`),
          h("p", { class: "meta" },
            h("span", { class: "status" }, p.status || "reading status"),
            h("span", {}, `deposit before ${fmtTime(p.deadline)}`)),
          h("div", { class: "addr" }, h("code", {}, p.depositAddress), copyButton(p.depositAddress))));
    })),
    residual > 0 ? h("p", { class: "plan-note" }, `${formatZec(residual)} ZEC stays shielded in your wallet.`) : null);
}

function swapReceipt(id, d, readAt) {
  let i = 0;
  const shared = d.sharedIdentifiers;
  const blocks = d.parts.map((p, k) =>
    h("div", { class: "r-sec part-block" },
      h("p", { class: "label" }, `Part ${(p.index ?? k) + 1}, ${p.status}`),
      p.publicOnStatus.map((f) => row(f.field, f.value, { hl: true, i: i++ }))));
  const linked = shared.length === 0
    ? h("div", { class: "r-sec" },
        h("p", { class: "label" }, "Shared across parts"),
        h("div", { class: "seal-line" },
          h("div", { class: "seal", role: "img", "aria-label": "0 shared identifiers" }, h("b", {}, String(shared.length)), h("span", {}, "shared identifiers")),
          h("p", {}, "No identifier repeats across the parts. A stranger holding one deposit address cannot reach the others.")))
    : h("div", { class: "r-sec" },
        h("p", { class: "label" }, `Shared across parts: ${shared.length}`),
        shared.map((s) => h("div", {},
          row(s.field, s.value, { hl: true, i: i++ }),
          h("p", { class: "help" }, `in parts ${s.parts.map((x) => x + 1).join(", ")}`))));
  const constants = d.sharedConstants.length === 0 ? null :
    h("details", { class: "constants" },
      h("summary", {}, `Route constants identical across parts: asset ids, fees, amount (${d.sharedConstants.length})`),
      d.sharedConstants.map((c) => h("div", { class: "row long" },
        h("div", { class: "top" }, h("span", { class: "help" }, c.path), h("span", { class: "lead", "aria-hidden": "true" })),
        h("span", { class: "v" }, String(c.value)),
        h("span", { class: "help" }, `parts ${c.parts.map((x) => x + 1).join(", ")}`))));
  return h("article", { class: "receipt", "aria-labelledby": "r-h" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("h2", { class: "r-title", id: "r-h", tabindex: "-1" }, "What a stranger reads"),
    blocks, linked,
    h("div", { class: "r-foot" },
      constants,
      h("p", { class: "src" }, `GET /api/swaps/${id}, read ${fmtTime(readAt)}`)));
}

function pageSwap(main, id) {
  if (!id) return pageSwapLookup(main);
  let cached = null;
  try { cached = JSON.parse(sessionStorage.getItem(`airlock.swap.${id}`)); } catch { cached = null; }
  const left = h("div", {});
  const right = h("div", {});
  let timer = null;
  let ctl = null;
  let lastKey = "";
  let dead = false;

  put(left, cached ? partsList(cached.parts, cached.residualZat) : "");
  put(right, h("div", { class: "receipt idle", "aria-live": "polite" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("p", { class: "r-title" }, "Reading status"),
    h("div", { class: "r-sec" }, waitNote(`Reading ${cached ? `${cached.parts.length} part${cached.parts.length === 1 ? "" : "s"}` : "the parts"} over Tor. The elapsed time counts here.`))));

  async function load(first) {
    ctl = new AbortController();
    try {
      const d = await api(`/api/swaps/${encodeURIComponent(id)}`, { signal: ctl.signal });
      if (dead) return;
      const key = JSON.stringify(d);
      if (key !== lastKey) {
        lastKey = key;
        put(left, partsList(d.parts, d.residualZat));
        put(right, swapReceipt(id, d, new Date()));
      } else {
        const src = right.querySelector(".src");
        if (src) src.textContent = `GET /api/swaps/${id}, read ${fmtTime(new Date())}`;
      }
      if (d.parts.some((p) => !TERMINAL.has(p.status))) timer = setTimeout(() => load(false), POLL_MS);
    } catch (e) {
      if (e.name === "AbortError" || dead) return;
      if (first && !lastKey) {
        put(right, errorNote(e));
      } else {
        right.prepend(errorNote(e));
      }
      timer = setTimeout(() => load(false), POLL_MS);
    }
  }
  load(true);

  put(main, 
    h("section", { class: "page", "aria-labelledby": "swap-h" },
      h("h1", { class: "display", id: "swap-h", tabindex: "-1" }, "Swap receipt."),
      h("p", { class: "data", style: "margin-top:16px;overflow-wrap:anywhere" }, `swap ${id}`),
      h("div", { class: "page-grid" },
        h("div", {},
          h("div", { class: "steps" },
            h("p", {}, "Send each part from your own wallet to its deposit address."),
            h("p", {}, "Parts can be funded one at a time, in any order, before each deadline.")),
          left),
        right)));
  return () => { dead = true; clearTimeout(timer); if (ctl) ctl.abort(); };
}

function pageSwapLookup(main) {
  let last = null;
  try { last = sessionStorage.getItem("airlock.lastSwap"); } catch { last = null; }
  const input = h("input", { id: "swap-id", name: "swap-id", type: "text", autocomplete: "off", spellcheck: "false", required: true, value: last || "" });
  put(main, 
    h("section", { class: "page", "aria-labelledby": "lookup-h" },
      h("h1", { class: "display", id: "lookup-h", tabindex: "-1" }, "Open a swap."),
      h("p", { class: "lede", style: "margin-top:20px" }, "Enter the swap id AIRLOCK returned when the swap was created."),
      h("form", { style: "max-width:640px", novalidate: true, onsubmit: (e) => {
        e.preventDefault();
        const v = input.value.trim();
        if (!v) { input.focus(); return; }
        location.hash = `#/swap/${encodeURIComponent(v)}`;
      } },
        h("label", { class: "field", for: "swap-id" }, h("span", { class: "label" }, "Swap id"), input),
        h("div", { class: "form-row" }, h("button", { class: "btn", type: "submit" }, "Open receipt ", h("span", { "aria-hidden": "true" }, "→"))))));
  return null;
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
      const res = await api("/api/inbound", {
        method: "POST", headers: { "content-type": "application/json" }, signal: ctl.signal,
        body: JSON.stringify({
          amountZec: amountIn.value.trim(), originAsset: assetSel.value,
          originRefund: refundIn.value.trim(), recipient: recipIn.value.trim(),
        }),
      });
      try {
        sessionStorage.setItem(`airlock.inbound.${res.inboundId}`, JSON.stringify(res));
        sessionStorage.setItem("airlock.lastInbound", res.inboundId);
      } catch { /* storage unavailable, the live view reads the service */ }
      location.hash = `#/inbound/${res.inboundId}`;
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
          h("label", { class: "field", for: "in-amount" }, h("span", { class: "label" }, "ZEC to receive"), amountIn),
          h("label", { class: "field", for: "in-asset" }, h("span", { class: "label" }, "Origin asset"), h("div", { class: "select-wrap" }, assetSel)),
          h("label", { class: "field", for: "in-refund" }, h("span", { class: "label" }, "Refund address on the origin chain"), refundIn),
          h("label", { class: "field", for: "in-recipient" }, h("span", { class: "label" }, "Recipient, Orchard-only unified address from your wallet"), recipIn),
          h("div", { class: "form-row" }, submit),
          status))));
  return () => { if (ctl) ctl.abort(); };
}

function inboundPay(d, tok) {
  const sym = tok ? tok.symbol : "";
  const fmt = (v) => (tok ? `${formatUnits(v, tok.decimals)} ${sym}` : String(v));
  return h("div", {},
    h("ol", { class: "parts single" },
      h("li", {},
        h("div", {},
          h("p", { class: "amt" }, `Send ${d.amountIn}${sym ? " " + sym : ""}`),
          h("p", { class: "meta" },
            h("span", { class: "status" }, d.status || "reading status"),
            h("span", {}, `at least ${fmt(d.minAmountIn)}`),
            h("span", {}, `deposit before ${fmtTime(d.deadline)}`)),
          h("div", { class: "addr" }, h("code", {}, d.depositAddress), copyButton(d.depositAddress))))),
    h("p", { class: "plan-note" }, `You receive ${formatZec(d.amountOutZat)} ZEC, shielded.`));
}

function inboundReceipt(id, d, readAt) {
  let i = 0;
  const fields = d.publicOnStatus.map((f) => row(f.field, f.value, { hl: true, i: i++ }));
  return h("article", { class: "receipt", "aria-labelledby": "r-h" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("h2", { class: "r-title", id: "r-h", tabindex: "-1" }, "What a stranger reads"),
    h("div", { class: "r-sec" }, row("Status", d.status), row("Updated", fmtTime(d.updatedAt))),
    h("div", { class: "r-sec" }, h("p", { class: "label" }, "Readable by anyone"), fields),
    h("div", { class: "r-sec" },
      row("Receive", `${formatZec(d.amountOutZat)} ZEC`),
      row("Deadline", fmtTime(d.deadline))),
    h("div", { class: "r-foot" },
      h("p", {}, "The recipient is a fresh Orchard-only address, so the destination of this swap is a shielded address."),
      h("p", { class: "src" }, `GET /api/inbound/${id}, read ${fmtTime(readAt)}`)));
}

function pageInbound(main, id) {
  let cached = null;
  try { cached = JSON.parse(sessionStorage.getItem(`airlock.inbound.${id}`)); } catch { cached = null; }
  const left = h("div", {});
  const right = h("div", {});
  let tok = null;
  let timer = null;
  let ctl = null;
  let lastKey = "";
  let dead = false;
  let lastData = cached;

  const paint = (d) => put(left, inboundPay(d, tok));
  if (cached) paint(cached);
  put(right, h("div", { class: "receipt idle", "aria-live": "polite" },
    h("p", { class: "label" }, "Exposure receipt"),
    h("p", { class: "r-title" }, "Reading status"),
    h("div", { class: "r-sec" }, waitNote("Reading the status over Tor. The elapsed time counts here."))));

  async function load(first) {
    ctl = new AbortController();
    try {
      const d = await api(`/api/inbound/${encodeURIComponent(id)}`, { signal: ctl.signal });
      if (dead) return;
      if (!tok) {
        try { tok = (await getTokens()).find((t) => t.assetId === d.originAsset) || null; } catch { tok = null; }
      }
      lastData = d;
      const key = JSON.stringify(d);
      if (key !== lastKey) {
        lastKey = key;
        paint(d);
        put(right, inboundReceipt(id, d, new Date()));
      } else {
        const src = right.querySelector(".src");
        if (src) src.textContent = `GET /api/inbound/${id}, read ${fmtTime(new Date())}`;
      }
      if (!TERMINAL.has(d.status)) timer = setTimeout(() => load(false), POLL_MS);
    } catch (e) {
      if (e.name === "AbortError" || dead) return;
      if (first && !lastKey) put(right, errorNote(e));
      else right.prepend(errorNote(e));
      timer = setTimeout(() => load(false), POLL_MS);
    }
  }
  load(true);
  getTokens().then((t) => {
    if (cached && !tok) { tok = t.find((x) => x.assetId === cached.originAsset) || null; if (!dead && lastData) paint(lastData); }
  }).catch(() => {});

  main.replaceChildren(
    h("section", { class: "page", "aria-labelledby": "ib-h" },
      h("h1", { class: "display", id: "ib-h", tabindex: "-1" }, "Inbound receipt."),
      h("p", { class: "data", style: "margin-top:16px;overflow-wrap:anywhere" }, `inbound ${id}`),
      h("div", { class: "page-grid" },
        h("div", {},
          h("div", { class: "steps" },
            h("p", {}, "Send the amount below from your own wallet to the deposit address."),
            h("p", {}, "Anything under the minimum is refunded to the address you gave.")),
          left),
        right)));
  return () => { dead = true; clearTimeout(timer); if (ctl) ctl.abort(); };
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
