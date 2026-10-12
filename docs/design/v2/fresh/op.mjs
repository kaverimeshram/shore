import { chromium } from 'playwright';
const b = await chromium.launch();
const p = await b.newPage({viewport:{width:1440,height:900}});
await p.goto('https://ui-v2.shore-zec.pages.dev/shore/',{waitUntil:'networkidle'});
console.log(await p.evaluate(()=>[...document.querySelectorAll('body *')].filter(e=>e.offsetParent&&parseFloat(getComputedStyle(e).opacity)<1).map(e=>e.className.toString().slice(0,50)+' op='+getComputedStyle(e).opacity+' :: '+e.textContent.trim().slice(0,40)).slice(0,15).join('\n')));
console.log(await p.evaluate(()=>{const r=[];for(const e of document.querySelectorAll('h1,h2,h3,.metric *,[class*=label],[class*=chip]')){const c=getComputedStyle(e); if(e.offsetParent) r.push(e.tagName+'.'+String(e.className).slice(0,30)+' '+c.fontFamily.split(',')[0]+' '+c.fontSize+'/'+c.fontWeight+' '+c.color+' '+c.letterSpacing+' :: '+e.textContent.trim().slice(0,25))} return [...new Set(r)].slice(0,40).join('\n')}));
await b.close();
