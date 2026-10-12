// Overlap probe: run inside the page. Returns problems at the current scroll position.
// 1) every sticky/fixed element must have an opaque background (on itself), so content scrolled under it is hidden;
// 2) no two visible text runs (outside sticky/fixed layers) may intersect on screen;
// 3) a text run under a sticky/fixed layer is fine only when that layer is opaque (checked by 1).
export function probeOverlap() {
  const out = [];
  const vis = (el) => { const s = getComputedStyle(el); return s.visibility !== "hidden" && s.display !== "none" && +s.opacity > 0.05; };
  const opaque = (el) => { const c = getComputedStyle(el).backgroundColor; const m = c.match(/rgba?\(([^)]+)\)/); if (!m) return false; const p = m[1].split(",").map(Number); return p.length < 4 || p[3] >= 0.98; };
  const layers = [...document.querySelectorAll("body *")].filter((el) => { const p = getComputedStyle(el).position; return (p === "sticky" || p === "fixed") && vis(el) && el.getBoundingClientRect().height > 0; });
  for (const l of layers) if (!opaque(l)) out.push(`sticky/fixed without opaque background: ${l.tagName}.${l.className || l.id}`);
  const inLayer = (n) => layers.some((l) => l.contains(n));
  const runs = [];
  const w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, { acceptNode: (t) => (t.textContent.trim() ? 1 : 2) });
  const vh = innerHeight, vw = innerWidth;
  for (let t; (t = w.nextNode()); ) {
    const el = t.parentElement; if (!el || !vis(el) || el.closest("noscript,script,style,[hidden],details:not([open])>*:not(summary)")) continue;
    let anc = el; let ok = true; while (anc) { if (!vis(anc)) { ok = false; break; } anc = anc.parentElement; } if (!ok) continue;
    const r = document.createRange(); r.selectNodeContents(t);
    for (const b of r.getClientRects()) {
      if (b.width < 1 || b.height < 1 || b.bottom <= 0 || b.top >= vh || b.right <= 0 || b.left >= vw) continue;
      runs.push({ b, el, layer: inLayer(el), txt: t.textContent.trim().slice(0, 30) });
    }
  }
  const hit = (a, b) => Math.min(a.right, b.right) - Math.max(a.left, b.left) > 2 && Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top) > 2;
  for (let i = 0; i < runs.length; i++) for (let j = i + 1; j < runs.length; j++) {
    const a = runs[i], b = runs[j];
    if (a.el === b.el || a.el.contains(b.el) || b.el.contains(a.el)) continue;
    if (a.layer !== b.layer) continue; // one is in an opaque sticky layer covering the other: allowed by rule 1
    if (hit(a.b, b.b)) out.push(`text overlap: "${a.txt}" x "${b.txt}"`);
  }
  return [...new Set(out)].slice(0, 12);
}
