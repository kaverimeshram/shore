import { chromium } from 'playwright';
const url='https://design-preview.shore-zec.pages.dev/shore/';
const b = await chromium.launch();
for (const [w,h,tag] of [[1440,900,'d'],[390,844,'m']]) {
  const p = await b.newPage({viewport:{width:w,height:h}});
  await p.goto(url,{waitUntil:'networkidle'});
  await p.screenshot({path:`${tag}-0-idle.png`});
  const click = async (re) => { const el=p.getByRole('button',{name:re}).first(); const n=await el.count(); if(n){await el.scrollIntoViewIfNeeded(); await el.click(); await p.waitForTimeout(1200);} console.log(tag,re,n); };
  await click(/Connect recorded wallet/i);
  await p.screenshot({path:`${tag}-1-connected.png`});
  await click(/fresh u1/i);
  await p.screenshot({path:`${tag}-2-address.png`});
  await click(/quote/i);
  await p.screenshot({path:`${tag}-3-quote.png`});
  console.log('buttons@quote', (await p.evaluate(()=>[...document.querySelectorAll('button')].filter(b=>b.offsetParent).map(b=>b.textContent.trim()+(b.disabled?'[dis]':'')))).join(' ; '));
  await click(/Approve/i);
  await p.waitForTimeout(800);
  await p.screenshot({path:`${tag}-4-signed.png`});
  await p.waitForTimeout(5000);
  await p.screenshot({path:`${tag}-5-crossing.png`});
  let t0=Date.now();
  await p.waitForFunction(()=>/Landed in/i.test(document.body.innerText),null,{timeout:120000}).catch(()=>console.log('no landed'));
  console.log('crossing took ms', Date.now()-t0);
  await p.waitForTimeout(1500);
  await p.screenshot({path:`${tag}-6-landed.png`});
  const occ = await p.evaluate(()=>{
    const vis=[...document.querySelectorAll('body *')].filter(e=>{const r=e.getBoundingClientRect(); return e.offsetParent && r.bottom>0 && r.top<innerHeight && [...e.childNodes].some(n=>n.nodeType==3&&/0\.00115|84\s?s/.test(n.textContent))});
    const strip=[...document.querySelectorAll('*')].find(e=>getComputedStyle(e).position==='sticky'||getComputedStyle(e).position==='fixed');
    const sb = strip? strip.getBoundingClientRect().bottom : 0;
    const bigs=[...document.querySelectorAll('body *')].filter(e=>e.offsetParent && parseFloat(getComputedStyle(e).fontSize)>=40 && [...e.childNodes].some(n=>n.nodeType==3&&n.textContent.trim())).map(e=>({t:e.textContent.trim().slice(0,20),top:Math.round(e.getBoundingClientRect().top),fs:getComputedStyle(e).fontSize}));
    return {visible:vis.map(e=>e.textContent.trim().slice(0,40)+'@'+getComputedStyle(e).fontSize), stripBottom:Math.round(sb), bigs, fonts:[...new Set([...document.querySelectorAll('body *')].map(e=>getComputedStyle(e).fontFamily.split(',')[0]))], bg:getComputedStyle(document.body).backgroundColor};
  });
  console.log(tag,'LANDED', JSON.stringify(occ));

  await p.screenshot({path:`${tag}-7-full.png`, fullPage:true});
  console.log('buttons@end', (await p.evaluate(()=>[...document.querySelectorAll('button')].filter(b=>b.offsetParent).map(b=>b.textContent.trim()+(b.disabled?'[dis]':'')))).join(' ; '));
  const info = await p.evaluate(()=>{
    const sticky=[...document.querySelectorAll('*')].filter(e=>getComputedStyle(e).position==='sticky').map(e=>({c:e.className, bg:getComputedStyle(e).backgroundColor, h:e.getBoundingClientRect().height}));
    const big=[...document.querySelectorAll('body *')].filter(e=>e.offsetParent && [...e.childNodes].some(n=>n.nodeType==3&&n.textContent.trim())).map(e=>({t:e.textContent.trim().slice(0,30), fs:parseFloat(getComputedStyle(e).fontSize), y:Math.round(e.getBoundingClientRect().top+scrollY), col:getComputedStyle(e).color})).filter(x=>x.fs>=30);
    const small=[...document.querySelectorAll('body *')].filter(e=>e.offsetParent && [...e.childNodes].some(n=>n.nodeType==3&&n.textContent.trim())).map(e=>parseFloat(getComputedStyle(e).fontSize));
    const amber=[...document.querySelectorAll('body *')].filter(e=>e.offsetParent && /22\.9/.test(e.textContent) && e.children.length==0).map(e=>getComputedStyle(e).color);
    const nums = getComputedStyle(document.querySelector('h1')||document.body).fontVariantNumeric;
    return {sticky,big,minFs:Math.min(...small),amber,scrollMt:[...document.querySelectorAll('[id]')].map(e=>e.id+':'+getComputedStyle(e).scrollMarginTop).filter(s=>!s.endsWith(':0px')), docH:document.documentElement.scrollHeight, landedY:null};
  });
  console.log(tag, JSON.stringify(info));
  // focus visibility on disclosure
  const s = p.locator('summary').first(); if (await s.count()) { await s.focus(); await p.keyboard.press('Enter'); await p.waitForTimeout(400); await s.scrollIntoViewIfNeeded(); await p.screenshot({path:`${tag}-8-disclosure.png`}); console.log('summary', await s.evaluate(e=>getComputedStyle(e).outline)); }
  await p.close();
}
// slow crossing at 1440 to see motion
const p = await b.newPage({viewport:{width:1440,height:900}});
await p.goto(url+'?speed=3',{waitUntil:'networkidle'});
for (const re of [/Connect recorded wallet/i,/fresh u1/i,/quote/i,/Approve/i]) { const el=p.getByRole('button',{name:re}).first(); if(await el.count()){await el.click(); await p.waitForTimeout(1000);} }
for (let i=0;i<4;i++){ await p.waitForTimeout(6000); await p.screenshot({path:`d-slow-${i}.png`, clip:{x:100,y:0,width:560,height:500}}); }
await b.close();
