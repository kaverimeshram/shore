import { chromium } from 'playwright';
const b = await chromium.launch();
for (const [w,h,tag] of [[1440,900,'d'],[390,844,'m']]) {
const p = await b.newPage({viewport:{width:w,height:h}});
await p.goto('https://ui-v2.shore-zec.pages.dev/shore/',{waitUntil:'networkidle'});
const audit = async (label) => {
  // tab through up to 40 stops; record outline vs adjacent bg contrast
  await p.evaluate(()=>document.activeElement&&document.activeElement.blur());
  await p.keyboard.press('Tab');
  const res=[];
  for(let i=0;i<40;i++){
    const r = await p.evaluate(()=>{
      const cv=document.createElement('canvas');cv.width=cv.height=1;const cx=cv.getContext('2d');
      const P=s=>{cx.clearRect(0,0,1,1);cx.fillStyle='#000';cx.fillStyle=s;cx.fillRect(0,0,1,1);const d=cx.getImageData(0,0,1,1).data;return d;};
      const lum=d=>{const f=x=>{x/=255;return x<=0.03928?x/12.92:((x+0.055)/1.055)**2.4};return 0.2126*f(d[0])+0.7152*f(d[1])+0.0722*f(d[2])};
      const e=document.activeElement; if(!e||e===document.body) return null; const c=getComputedStyle(e);
      let n=e.parentElement,bg='rgb(255,255,255)'; while(n){const b=getComputedStyle(n).backgroundColor; if(b!=='rgba(0, 0, 0, 0)'){bg=b;break} n=n.parentElement}
      const oc = c.outlineStyle!=='none'&&parseFloat(c.outlineWidth)>0 ? c.outlineColor : (c.boxShadow!=='none'? 'shadow:'+c.boxShadow : 'NONE');
      let cr=null; if(!oc.startsWith('shadow')&&oc!=='NONE'){const a=lum(P(oc)),bb=lum(P(bg)); cr=(Math.max(a,bb)+.05)/(Math.min(a,bb)+.05)}
      return {t:(e.textContent||e.getAttribute('aria-label')||e.tagName).trim().slice(0,28), oc, w:c.outlineWidth, bg, cr:cr&&Math.round(cr*100)/100};
    });
    if(!r) break; res.push(r); await p.keyboard.press('Tab');
  }
  console.log(tag,label, res.map(x=>`${x.t} [${x.oc} ${x.w} on ${x.bg} => ${x.cr}]`).join('\n  '));
};
await audit('idle');
for (const re of [/Connect recorded wallet/i,/fresh u1|fresh/i,/quote/i]) { const el=p.getByRole('button',{name:re}).first(); if(await el.count()){await el.click(); await p.waitForTimeout(1200);} }
await audit('quote');
await p.getByRole('button',{name:/Approve/i}).first().focus(); await p.keyboard.press('Shift+Tab'); await p.keyboard.press('Tab');
const bb=await p.getByRole('button',{name:/Approve/i}).first().boundingBox(); await p.screenshot({path:`${tag}-focus-approve.png`,clip:{x:Math.max(0,bb.x-16),y:bb.y-16,width:Math.min(w,bb.width+32),height:bb.height+32}});
await p.goto('https://ui-v2.shore-zec.pages.dev/shore/',{waitUntil:'networkidle'});
await p.getByRole('button',{name:/Connect recorded wallet/i}).focus(); await p.keyboard.press('Shift+Tab'); await p.keyboard.press('Tab');
const b2=await p.getByRole('button',{name:/Connect recorded wallet/i}).boundingBox(); await p.screenshot({path:`${tag}-focus-connect.png`,clip:{x:Math.max(0,b2.x-16),y:b2.y-16,width:Math.min(w,b2.width+32),height:b2.height+32}});
await p.close();
}
await b.close();
