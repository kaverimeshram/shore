import { chromium } from 'playwright';
const b = await chromium.launch();
const p = await b.newPage({viewport:{width:1440,height:900}});
await p.goto('https://design-preview.shore-zec.pages.dev/shore/',{waitUntil:'networkidle'});
for (const re of [/Connect recorded wallet/i,/fresh u1/i,/quote/i,/Approve/i]) { const el=p.getByRole('button',{name:re}).first(); if(await el.count()){await el.click(); await p.waitForTimeout(1000);} }
const t0=Date.now();
await p.waitForFunction(()=>/Landed on Zcash/i.test(document.body.innerText),null,{timeout:60000});
console.log('landed after ms',Date.now()-t0);
const el = p.getByText(/What.s still public/i).first();
await el.scrollIntoViewIfNeeded(); await el.click(); await p.waitForTimeout(700);
await el.evaluate(e=>e.scrollIntoView({block:'start'})); await p.waitForTimeout(300);
await p.screenshot({path:'d-9-public.png'});
console.log(await el.evaluate(e=>{const d=e.closest('section,details,div'); return d.innerText.slice(0,1200)}));
console.log(await p.evaluate(()=>[...document.querySelectorAll('body *')].filter(e=>e.children.length==0 && /anyone|nobody/i.test(e.textContent)).map(e=>e.textContent.trim().slice(0,30)+'|'+getComputedStyle(e).color+'|'+getComputedStyle(e,'::before').backgroundColor).join('\n')));
await b.close();
