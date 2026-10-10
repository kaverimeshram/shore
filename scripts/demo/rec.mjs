// CDP screencast recorder for one pinned tab in the deepsurge profile.
// usage (as a module): import { open } from "./rec.mjs"
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

export async function open(pinFile, { w = 1280, h = 720 } = {}) {
  const port = execFileSync("bhn", ["deepsurge", "status"], { encoding: "utf8" }).match(/\s(\d{4,5})\s+RUNNING/)[1];
  const base = `http://127.0.0.1:${port}`;
  let tab;
  const pages = (await (await fetch(`${base}/json/list`)).json()).filter((t) => t.type === "page");
  const pinned = fs.existsSync(pinFile) && fs.readFileSync(pinFile, "utf8").trim();
  tab = pages.find((t) => t.id === pinned);
  if (!tab) {
    tab = await (await fetch(`${base}/json/new?about:blank`, { method: "PUT" })).json();
    fs.writeFileSync(pinFile, tab.id);
  }
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let id = 0;
  const pend = {};
  let frames = [];
  let recording = false;
  ws.onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.id && pend[d.id]) { pend[d.id](d); delete pend[d.id]; }
    else if (d.method === "Page.screencastFrame") {
      if (recording) frames.push({ t: d.params.metadata.timestamp, data: d.params.data });
      send("Page.screencastFrameAck", { sessionId: d.params.sessionId });
    }
  };
  const send = (method, params = {}) => new Promise((res, rej) => {
    const i = ++id; pend[i] = (d) => (d.error ? rej(new Error(method + ": " + d.error.message)) : res(d.result));
    ws.send(JSON.stringify({ id: i, method, params }));
  });
  await send("Page.enable");
  await send("Page.bringToFront");
  await send("Emulation.setDeviceMetricsOverride", { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  const ev = async (expr) => (await send("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true })).result?.value;

  const api = {
    send, ev, sleep,
    async goto(url) { await send("Page.navigate", { url }); await sleep(1500); await api.cursorInit(); },
    async cursorInit() {
      await ev(`(()=>{if(document.getElementById('__cur'))return;const c=document.createElement('div');c.id='__cur';
      c.style.cssText='position:fixed;left:0;top:0;width:22px;height:22px;border-radius:50%;background:#E4501B;border:3px solid #14120F;z-index:99999;pointer-events:none;transition:transform .55s cubic-bezier(.3,.7,.2,1);transform:translate(640px,360px)';
      document.body.appendChild(c)})()`);
    },
    async moveTo(sel) {
      const r = JSON.parse(await ev(`(()=>{const e=document.querySelector(${JSON.stringify(sel)});if(!e)return 'null';e.scrollIntoView({block:'center'});const b=e.getBoundingClientRect();return JSON.stringify([b.x+Math.min(b.width/2,60),b.y+b.height/2])})()`));
      if (!r) throw new Error("no element " + sel);
      await api.cursorInit();
      await ev(`document.getElementById('__cur').style.transform='translate(${r[0]}px,${r[1]}px)'`);
      await sleep(700);
    },
    async click(sel) { await api.moveTo(sel); await ev(`document.querySelector(${JSON.stringify(sel)}).click()`); await sleep(300); },
    async clickText(tag, text) {
      const sel = await ev(`(()=>{const e=[...document.querySelectorAll(${JSON.stringify(tag)})].find(x=>x.innerText.trim().startsWith(${JSON.stringify(text)}));if(!e)return null;e.setAttribute('data-rec','1');return '[data-rec=\"1\"]'})()`);
      if (!sel) throw new Error("no " + tag + " with text " + text);
      await api.click(sel);
      await ev(`document.querySelectorAll('[data-rec]').forEach(e=>e.removeAttribute('data-rec'))`);
    },
    async type(sel, text, ms = 28) {
      await api.moveTo(sel);
      await ev(`(()=>{const e=document.querySelector(${JSON.stringify(sel)});e.focus();e.select&&e.select()})()`);
      await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Backspace", code: "Backspace", windowsVirtualKeyCode: 8 });
      await send("Input.dispatchKeyEvent", { type: "keyUp", key: "Backspace", code: "Backspace", windowsVirtualKeyCode: 8 });
      for (const ch of text) { await send("Input.insertText", { text: ch }); await sleep(ms); }
    },
    async waitText(text, timeout = 60000) {
      const t0 = Date.now();
      while (Date.now() - t0 < timeout) {
        if (await ev(`document.body.innerText.includes(${JSON.stringify(text)})`)) return true;
        await sleep(400);
      }
      throw new Error("timeout waiting for " + text);
    },
    async start() {
      frames = []; recording = true;
      await send("Page.startScreencast", { format: "jpeg", quality: 92, everyNthFrame: 1 });
    },
    async stop(outMp4, tmp) {
      outMp4 = path.resolve(outMp4); tmp = path.resolve(tmp);
      recording = false;
      await send("Page.stopScreencast");
      fs.rmSync(tmp, { recursive: true, force: true }); fs.mkdirSync(tmp, { recursive: true });
      let list = "";
      frames.forEach((f, i) => {
        const p = `${tmp}/f${String(i).padStart(5, "0")}.jpg`;
        fs.writeFileSync(p, Buffer.from(f.data, "base64"));
        const dur = i + 1 < frames.length ? frames[i + 1].t - f.t : 0.5;
        list += `file '${p}'\nduration ${Math.max(dur, 0.01).toFixed(4)}\n`;
      });
      list += `file '${tmp}/f${String(frames.length - 1).padStart(5, "0")}.jpg'\n`;
      fs.writeFileSync(`${tmp}/list.txt`, list);
      execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", `${tmp}/list.txt`, "-vf", `fps=30,scale=${w}:${h},setsar=1`, "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18", outMp4]);
      return frames.length;
    },
    close() { ws.close(); },
  };
  return api;
}
