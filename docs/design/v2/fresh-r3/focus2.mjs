import { chromium } from 'playwright';
const b = await chromium.launch();
const p = await b.newPage({viewport:{width:1440,height:900}});
await p.goto('https://ui-v2.shore-zec.pages.dev/shore/',{waitUntil:'networkidle'});
const btn=p.getByRole('button',{name:/Connect recorded wallet/i});
await btn.focus(); await p.keyboard.press('Shift+Tab'); await p.keyboard.press('Tab');
for (const t of [0,50,100,200,500]) { if(t) await p.waitForTimeout(t===50?50:t-[0,50,100,200,500][[0,50,100,200,500].indexOf(t)-1]); console.log(t, await btn.evaluate(e=>getComputedStyle(e).outlineColor+' '+getComputedStyle(e).transitionProperty.slice(0,60)+' '+getComputedStyle(e).transitionDuration)); }
const bb=await btn.boundingBox(); await p.screenshot({path:'d-focus-connect-500ms.png',clip:{x:bb.x-16,y:bb.y-16,width:bb.width+32,height:bb.height+32}});
await b.close();
