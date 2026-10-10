// SHORE store: the API's only state (sessions, the address index, the used-address set) in D1.
// Called by the Rust API with a bearer token (secret TOKEN). The two privacy guards are atomic in SQL:
// the index is one UPDATE ... RETURNING (D1 serialises writes), and a used address is a primary key.
const json = (v, status = 200) => new Response(JSON.stringify(v), { status, headers: { "content-type": "application/json" } });

function same(a, b) {
  if (typeof a !== "string" || a.length !== b.length) return false;
  let d = 0;
  for (let i = 0; i < a.length; i++) d |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return d === 0;
}

export default {
  async fetch(req, env) {
    if (!env.TOKEN || !same((req.headers.get("authorization") || "").replace(/^Bearer /, ""), env.TOKEN)) return json({ error: "unauthorized" }, 401);
    const url = new URL(req.url);
    const [, a, b] = url.pathname.replace(/^\/store(?=\/)/, "").split("/");
    const key = b ? decodeURIComponent(b) : "";
    if (key && !/^[A-Za-z0-9]{1,200}$/.test(key)) return json({ error: "bad key" }, 400);
    const db = env.DB;
    if (req.method === "POST" && a === "index") {
      const { start = 0 } = await req.json();
      await db.prepare("INSERT OR IGNORE INTO counter (k, v) VALUES ('ua', ?)").bind(Number(start) | 0).run();
      const r = await db.prepare("UPDATE counter SET v = v + 1 WHERE k = 'ua' RETURNING v - 1 AS index_").first();
      return json({ index: r.index_ });
    }
    if (req.method === "POST" && a === "claim") {
      const { u1 } = await req.json();
      if (typeof u1 !== "string" || !/^u1[a-z0-9]{50,400}$/.test(u1)) return json({ error: "bad u1" }, 400);
      const r = await db.prepare("INSERT OR IGNORE INTO used (u1, at) VALUES (?, ?)").bind(u1, Date.now()).run();
      return json({ claimed: r.meta.changes === 1 });
    }
    if (a === "session" && key && req.method === "PUT") {
      const s = await req.json();
      if (s.id !== key) return json({ error: "id mismatch" }, 400);
      await db.prepare("INSERT INTO session (id, deposit, recipient, body) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET deposit = ?2, recipient = ?3, body = ?4")
        .bind(key, s.deposit_address || null, s.recipient, JSON.stringify(s)).run();
      return json({ ok: true });
    }
    if (req.method === "GET" && key) {
      const q = { session: "SELECT body FROM session WHERE id = ?", deposit: "SELECT body FROM session WHERE deposit = ? LIMIT 1" }[a];
      if (q) { const r = await db.prepare(q).bind(key).first(); return json({ session: r ? JSON.parse(r.body) : null }); }
      if (a === "recipient") { const r = await db.prepare("SELECT COUNT(*) AS n FROM session WHERE recipient = ?").bind(key).first(); return json({ count: r.n }); }
    }
    return json({ error: "not found" }, 404);
  },
};
