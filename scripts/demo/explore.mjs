import { open } from "./rec.mjs";
const t = await open(process.env.HOME + "/.cache/qa-tab-airlock");
for (const r of ["#/", "#/split", "#/inbound"]) {
  await t.goto("https://airlock-zec.vercel.app/" + r); await t.sleep(1500);
  console.log(r, await t.ev(`JSON.stringify([...document.querySelectorAll('input,select,button,textarea')].map(e=>[e.tagName,e.name,e.id,e.type,e.placeholder,(e.innerText||'').slice(0,30)]))`));
}
t.close();
