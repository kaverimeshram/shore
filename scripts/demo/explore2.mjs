import { open } from "./rec.mjs";
const t = await open(process.env.HOME + "/.cache/qa-tab-airlock");
await t.goto("https://airlock-zec.vercel.app/#/split"); await t.sleep(1000);
await t.type("#amount","0.02"); await t.type("#cap","0.01"); await t.sleep(1500);
console.log(await t.ev(`JSON.stringify([...document.querySelectorAll('input,select,button')].map(e=>[e.tagName,e.name,e.id,e.placeholder,(e.innerText||'').slice(0,30)]))`));
console.log(await t.ev(`document.querySelector('select[name=asset]').innerHTML.slice(0,900)`));
console.log(await t.ev(`document.querySelector('select[name=spacing]').innerHTML`));
t.close();
