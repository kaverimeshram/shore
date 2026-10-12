import { chromium } from 'playwright';
import fs from 'fs';
const base='https://ui-v2.shore-zec.pages.dev/shore/';
const b = await chromium.launch();
const audit = () => {
  const cv=document.createElement('canvas'); cv.width=cv.height=1; const cx=cv.getContext('2d',{willReadFrequently:true});
  const parse=s=>{ if(!s||s==='transparent') return {r:0,g:0,b:0,a:0}; cx.clearRect(0,0,1,1); cx.fillStyle='#000'; cx.fillStyle=s; cx.fillRect(0,0,1,1); const d=cx.getImageData(0,0,1,1).data; return {r:d[0],g:d[1],b:d[2],a:d[3]/255}; };
  const lum=c=>{const f=x=>{x/=255;return x<=0.03928?x/12.92:((x+0.055)/1.055)**2.4}; return 0.2126*f(c.r)+0.7152*f(c.g)+0.0722*f(c.b)};
  const bgOf=e=>{ let layers=[]; for(let n=e;n;n=n.parentElement){const c=parse(getComputedStyle(n).backgroundColor); if(c&&c.a>0){layers.push(c); if(c.a>=1) break;}} let base={r:255,g:255,b:255}; for(const c of layers.reverse()){base={r:c.r*c.a+base.r*(1-c.a),g:c.g*c.a+base.g*(1-c.a),b:c.b*c.a+base.b*(1-c.a)}} return base;};
  const out=[];
  for (const e of document.querySelectorAll('body *')) {
    if(!e.offsetParent && getComputedStyle(e).position!=='fixed') continue;
    const txt=[...e.childNodes].filter(n=>n.nodeType==3).map(n=>n.textContent).join('').trim(); if(!txt) continue;
    const cs=getComputedStyle(e); let fg=parse(cs.color); const bg=bgOf(e); const op=parseFloat(cs.opacity);
    if(fg.a<1){fg={r:fg.r*fg.a+bg.r*(1-fg.a),g:fg.g*fg.a+bg.g*(1-fg.a),b:fg.b*fg.a+bg.b*(1-fg.a)}}
    const L1=lum(fg),L2=lum(bg); const cr=(Math.max(L1,L2)+.05)/(Math.min(L1,L2)+.05);
    const fs=parseFloat(cs.fontSize), fw=parseInt(cs.fontWeight); const large=fs>=24||(fs>=18.66&&fw>=700);
    out.push({t:txt.slice(0,40),fs,fw,cr:Math.round(cr*100)/100,need:large?3:4.5,font:cs.fontFamily.split(',')[0]});
  }
  return out;
};
const report={};
for (const [w,h,tag] of [[1440,900,'d'],[390,844,'m']]) {
  const p = await b.newPage({viewport:{width:w,height:h}});
  await p.goto(base,{waitUntil:'networkidle'});
  const shot=async n=>{await p.screenshot({path:`${tag}-${n}.png`}); report[`${tag}-${n}`]=await p.evaluate(audit);};
  await shot('0-idle');
  await p.screenshot({path:`${tag}-0-idle-full.png`,fullPage:true});
  const click=async re=>{const el=p.getByRole('button',{name:re}).first(); const n=await el.count(); if(n){await el.scrollIntoViewIfNeeded(); await el.click(); await p.waitForTimeout(1300);} console.log(tag,re,n); return n;};
  await click(/Connect recorded wallet/i); await shot('1-connected');
  await click(/fresh u1|fresh/i); await shot('2-address');
  await click(/quote/i); await shot('3-quote');
  console.log(tag,'buttons@quote', (await p.evaluate(()=>[...document.querySelectorAll('button')].filter(b=>b.offsetParent).map(b=>b.textContent.trim().slice(0,30)+'|'+getComputedStyle(b).backgroundColor))).join(' ; '));
  await click(/Approve/i); await p.waitForTimeout(500); await shot('4-signed');
  await p.waitForTimeout(4000); await shot('5-crossing');
  await p.waitForFunction(()=>/Landed on Zcash/i.test(document.body.innerText),null,{timeout:90000}).catch(()=>console.log('no landed text'));
  await p.waitForTimeout(1500); await shot('6-landed');
  await p.screenshot({path:`${tag}-7-full.png`,fullPage:true});
  console.log(tag,'buttons@end', (await p.evaluate(()=>[...document.querySelectorAll('button,a.btn,[role=button]')].filter(b=>b.offsetParent).map(b=>b.textContent.trim().slice(0,30)))).join(' ; '));
  console.log(tag,'docH',await p.evaluate(()=>document.documentElement.scrollHeight), 'scrollY', await p.evaluate(()=>scrollY));
  const sm=p.getByText(/What.s still public/i).first(); if(await sm.count()){await sm.click(); await p.waitForTimeout(600); await sm.evaluate(e=>e.scrollIntoView({block:'start'})); await p.waitForTimeout(300); await p.screenshot({path:`${tag}-8-public.png`}); report[`${tag}-8-public`]=await p.evaluate(audit);}
  // keyboard focus
  await p.goto(base,{waitUntil:'networkidle'}); for(let i=0;i<4;i++) await p.keyboard.press('Tab');
  console.log(tag,'focus',await p.evaluate(()=>{const e=document.activeElement,c=getComputedStyle(e);return e.textContent.trim().slice(0,30)+' outline:'+c.outlineStyle+' '+c.outlineWidth+' '+c.outlineColor+' shadow:'+c.boxShadow.slice(0,60)}));
  await p.screenshot({path:`${tag}-9-focus.png`});
  await p.close();
}
// tour mode slow
const p=await b.newPage({viewport:{width:1440,height:900}});
await p.goto(base+'?replay=1&tour=1&speed=3',{waitUntil:'networkidle'});
for(let i=0;i<6;i++){await p.waitForTimeout(7000); await p.screenshot({path:`d-tour-${i}.png`});}
fs.writeFileSync('contrast.json',JSON.stringify(report));
const fails=[]; for(const [k,v] of Object.entries(report)) for(const x of v) if(x.cr<x.need) fails.push(k+' '+JSON.stringify(x));
console.log('AA FAILS', fails.length); console.log([...new Set(fails.map(f=>f.replace(/^\S+ /,'')))].slice(0,40).join('\n'));
const sizes={}; for(const v of Object.values(report)) for(const x of v) sizes[x.fs]=(sizes[x.fs]||0)+1; console.log('sizes',JSON.stringify(sizes));
let minc=99; for(const v of Object.values(report)) for(const x of v) minc=Math.min(minc,x.cr); console.log('min contrast', minc);
await b.close();
