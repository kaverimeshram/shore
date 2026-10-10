import { open } from "./rec.mjs";
const t = await open(process.env.HOME + "/.cache/qa-tab-airlock");
await t.goto("https://airlock-zec.vercel.app/#/"); await t.sleep(1200);
await t.start(); await t.sleep(1500);
await t.clickText("button", "TRY A SETTLED SWAP");
await t.waitText("READABLE BY ANYONE"); await t.sleep(2500);
// scroll the receipt down in steps so each exposed field is on screen
for (const y of [380, 720, 1060]) { await t.ev(`window.scrollTo({top:${y},behavior:'smooth'})`); await t.sleep(2600); }
const n = await t.stop("../../work/video/clips/s1-check.mp4", "../../work/video/tmp-s1");
console.log("frames", n);
t.close();
