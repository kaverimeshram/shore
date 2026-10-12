import { chromium } from 'playwright';
const b = await chromium.launch();
const p = await b.newPage({viewport:{width:1440,height:900}});
await p.goto('https://ui-v2.shore-zec.pages.dev/shore/',{waitUntil:'networkidle'});
console.log(await p.evaluate(()=>[...document.querySelectorAll('button,a,summary')].filter(e=>e.offsetParent).map(e=>e.tagName+':'+e.textContent.trim().slice(0,40)).join(' | ')));
console.log(await p.evaluate(()=>[...new Set([...document.querySelectorAll('body *')].map(e=>getComputedStyle(e).fontFamily.split(',')[0]))].join(',')));
await b.close();
