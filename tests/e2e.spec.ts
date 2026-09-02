import {test,expect,type Page} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {spawn} from 'node:child_process';
import {existsSync,mkdtempSync,readFileSync,rmSync,statSync,symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';

test.describe.configure({mode:'serial'});
const ownerCode=process.env.MCB_TEST_OWNER_CODE||'adult-setup-code-0123456789';
const authToken=process.env.MCB_TEST_AUTH_TOKEN||'integration-test-entra-token';
const authenticate=async(page:Page)=>page.addInitScript(token=>sessionStorage.setItem('mcb:test-access-token',token),authToken);
const seriousAxe=async(page:Page)=>(await new AxeBuilder({page}).analyze()).violations.filter(item=>['serious','critical'].includes(item.impact||''));

test('plain public first screen and metadata are complete',async({page})=>{
  const errors:string[]=[];
  page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
  page.on('pageerror',error=>errors.push(error.message));
  const response=await page.goto('/');
  expect(response?.status()).toBe(200);
  await expect(page).toHaveTitle('Math Circle Board — Plan small math-circle sessions');
  await expect(page.getByRole('heading',{level:1})).toHaveText('Plan and record small math-circle sessions');
  await expect(page.getByText('For volunteer math circle facilitators')).toBeVisible();
  await expect(page.getByRole('link',{name:'Try it with sample data'})).toHaveAttribute('href','/demo');
  await expect(page.getByRole('button',{name:'Sign in with Microsoft'})).toBeVisible();
  await expect(page.getByRole('heading',{level:2,name:'Data kept on the board'})).toBeVisible();
  await expect(page.getByText('Original AI-assisted environmental art')).toHaveCount(0);
  await expect(page.locator('footer')).toContainText('v0.1.0');
  await expect(page.locator('link[rel=canonical]')).toHaveAttribute('href','https://math-circle-board.sociobot.in/');
  await expect(page.locator('meta[property="og:image"]')).toHaveAttribute('content',/social-card\.webp$/);
  expect(await seriousAxe(page)).toEqual([]);
  expect(errors).toEqual([]);
});

test('390 px landing headline renders without JavaScript or the status API',async({browser})=>{
  const context=await browser.newContext({viewport:{width:390,height:844},javaScriptEnabled:false});
  const page=await context.newPage();
  const response=await page.goto('/');
  expect(response?.status()).toBe(200);
  await expect(page.getByRole('heading',{level:1})).toHaveText('Plan and record small math-circle sessions');
  await expect(page.getByRole('link',{name:'Try it with sample data'})).toBeVisible();
  expect(await page.locator('h1').count()).toBe(1);
  await context.close();
});

test('390 px DevTools-throttled landing LCP stays below 2.5 seconds',async({browser})=>{
  const context=await browser.newContext({viewport:{width:390,height:844}});
  await context.addInitScript(()=>{
    (window as unknown as {mcbLcp?:number,mcbLcpText?:string}).mcbLcp=0;
    new PerformanceObserver(list=>{
      const entry=list.getEntries().at(-1) as PerformanceEntry&{element?:Element};
      const metrics=window as unknown as {mcbLcp?:number,mcbLcpText?:string};
      metrics.mcbLcp=entry.startTime;
      metrics.mcbLcpText=entry.element?.textContent?.trim();
    }).observe({type:'largest-contentful-paint',buffered:true});
  });
  const page=await context.newPage();
  const cdp=await context.newCDPSession(page);
  await cdp.send('Network.enable');
  await cdp.send('Network.emulateNetworkConditions',{offline:false,latency:562.5,downloadThroughput:1474.56*1024/8,uploadThroughput:675*1024/8,connectionType:'cellular3g'});
  await cdp.send('Emulation.setCPUThrottlingRate',{rate:4});
  await page.goto('/',{waitUntil:'load'});
  await page.waitForTimeout(1000);
  const metric=await page.evaluate(()=>({
    lcp:(window as unknown as {mcbLcp?:number}).mcbLcp||0,
    text:(window as unknown as {mcbLcpText?:string}).mcbLcpText||'',
    width:innerWidth,
  }));
  expect(metric.width).toBe(390);
  expect(metric.text).toContain('Plan and record small math-circle sessions');
  expect(metric.lcp).toBeGreaterThan(0);
  expect(metric.lcp).toBeLessThan(2500);
  await context.close();
});

test('@claim:demo-isolation sample mode is isolated and resettable',async({page})=>{
  const requests:string[]=[];
  page.on('request',request=>requests.push(request.url()));
  await page.goto('/?demo=1');
  await expect(page).toHaveURL(/\/board\?demo=1(?:#main)?$/);
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await page.getByRole('link',{name:'Learners'}).click();
  await page.getByLabel('Learner alias').fill('Ravi');
  await page.getByRole('button',{name:'Add learner'}).click();
  await expect(page.getByText('Ravi',{exact:true})).toBeVisible();
  const keys=await page.evaluate(()=>({local:Object.keys(localStorage),session:Object.keys(sessionStorage)}));
  expect(keys.local.filter(key=>key.includes('offline-board')||key.startsWith('demo:'))).toEqual([]);
  expect(keys.session.filter(key=>key.startsWith('demo:'))).toEqual(['demo:math-circle-board:board']);
  await page.getByRole('button',{name:'Reset demo'}).click();
  await expect(page.getByText('Ada',{exact:true})).toBeVisible();
  await expect(page.getByText('Ravi',{exact:true})).toHaveCount(0);
  expect(requests.some(url=>new URL(url).pathname.startsWith('/api/'))).toBeFalsy();
  await expect(page.getByRole('link',{name:'Start for real'})).toHaveAttribute('href','/');
});

test('@claim:sample-counts reset restores two sessions, three learners, four problems, and four attempts',async({page})=>{
  await page.goto('/learners?demo=1');
  await page.getByLabel('Learner alias').fill('Ravi');
  await page.getByRole('button',{name:'Add learner'}).click();
  await expect(page.locator('.learner-list > li')).toHaveCount(4);
  await page.getByRole('button',{name:'Reset demo'}).click();
  const counts=await page.evaluate(()=>{
    const sample=JSON.parse(sessionStorage.getItem('demo:math-circle-board:board')||'{}');
    return {sessions:sample.sessions?.length,learners:sample.learners?.length,problems:sample.problems?.length,attempts:sample.attempts?.length};
  });
  expect(counts).toEqual({sessions:2,learners:3,problems:4,attempts:4});
  await expect(page.locator('.learner-list > li')).toHaveCount(3);
});

test('@claim:learner-range the board supports a private group of up to 12 learners',async({page})=>{
  await page.goto('/');
  await expect(page.getByText(/recap for 6–12 learners/)).toBeVisible();
  await page.goto('/learners?demo=1');
  for(let learner=4;learner<=12;learner++){
    await page.getByLabel('Learner alias').fill(`Learner ${learner}`);
    await page.getByRole('button',{name:'Add learner'}).click();
  }
  await expect(page.locator('.learner-list > li')).toHaveCount(12);
  await page.getByLabel('Learner alias').fill('Learner 13');
  await page.getByRole('button',{name:'Add learner'}).click();
  await expect(page.locator('.add-learner .form-error')).toHaveText('A private circle can have up to 12 learner aliases.');
  await expect(page.locator('.learner-list > li')).toHaveCount(12);
});

test('demo rejects learner aliases case-insensitively like the backend',async({page})=>{
  await page.goto('/learners?demo=1');
  await page.getByLabel('Learner alias').fill('aDa');
  await page.getByRole('button',{name:'Add learner'}).click();
  await expect(page.locator('.add-learner .form-error')).toHaveText('That learner alias is already in the circle.');
  await expect(page.locator('.learner-list strong').filter({hasText:/^Ada$/i})).toHaveCount(1);
  await expect(page.locator('.learner-list > li')).toHaveCount(3);
});

test('demo rejects whitespace-only aliases with recovery like the backend',async({page})=>{
  await page.goto('/learners?demo=1');
  await page.getByLabel('Learner alias').fill('   ');
  await page.getByRole('button',{name:'Add learner'}).click();
  await expect(page.locator('.add-learner .form-error')).toHaveText('Enter a learner alias of 60 characters or fewer.');
  await expect(page.locator('.learner-list > li')).toHaveCount(3);
  await page.getByLabel('Learner alias').fill('Ravi');
  await page.getByRole('button',{name:'Add learner'}).click();
  await expect(page.getByText('Ravi',{exact:true})).toBeVisible();
  await expect(page.locator('.learner-list > li')).toHaveCount(4);
});

test('both demo entry points use the demo-specific document title',async({page})=>{
  for(const route of ['/demo','/?demo=1']){
    await page.goto(route);
    await expect(page).toHaveURL(/\/board\?demo=1(?:#main)?$/);
    await expect(page).toHaveTitle('Demo — Math Circle Board');
    await expect(page.locator('link[rel=canonical]')).toHaveAttribute('href','https://math-circle-board.sociobot.in/demo');
  }
});

test('demo rejects corrupt image bytes and recovers with a valid image like the backend',async({page})=>{
  await page.goto('/?demo=1');
  await page.getByLabel('Add photo').setInputFiles({name:'corrupt.png',mimeType:'image/png',buffer:Buffer.from([137,80,78,71,13,10,26,10])});
  await expect(page.locator('.toast')).toHaveText('Use a valid JPEG, PNG, or WebP image under 5 MB.');
  await expect(page.getByRole('img',{name:/Uploaded attempt/})).toHaveCount(0);
  expect(await page.evaluate(()=>JSON.parse(sessionStorage.getItem('demo:math-circle-board:board')||'{}').attachments.length)).toBe(0);
  await page.getByLabel('Add photo').setInputFiles('frontend/public/art/lantern-room-768.webp');
  await expect(page.getByRole('img',{name:/Uploaded attempt/})).toBeVisible();
});

test('@claim:attempt-record demo records partial attempts and private notes',async({page})=>{
  await page.goto('/?demo=1');
  await page.getByLabel('What they tried').fill('Grouped the moves into pairs and checked the parity.');
  await page.getByText('◐ Exploring',{exact:true}).click();
  await page.getByLabel('Private facilitator note').fill('Ask for a diagram before the next hint.');
  await page.getByRole('button',{name:'Save attempt'}).click();
  await expect(page.getByText('Attempt saved.')).toBeVisible();
  await page.getByLabel('Add photo').setInputFiles('frontend/public/art/lantern-room-768.webp');
  await expect(page.getByRole('img',{name:/Uploaded attempt/})).toBeVisible();
  await page.reload();
  await expect(page.getByLabel('What they tried')).toHaveValue('Grouped the moves into pairs and checked the parity.');
  await expect(page.getByLabel('Private facilitator note')).toHaveValue('Ask for a diagram before the next hint.');
  await expect(page.getByRole('img',{name:/Uploaded attempt/})).toBeVisible();
});

test('@claim:recap-privacy printable recap omits private notes',async({page})=>{
  await page.goto('/?demo=1');
  await page.getByLabel('What they tried').fill('Matched each move with a second move.');
  await page.getByLabel('Private facilitator note').fill('PRIVATE NOTE MUST NOT PRINT');
  await page.getByRole('button',{name:'Save attempt'}).click();
  await page.getByRole('button',{name:'Print recap'}).click();
  await expect(page).toHaveURL(/\/recap\?demo=1$/);
  await expect(page.getByText('Matched each move with a second move.')).toBeVisible();
  await expect(page.getByText('PRIVATE NOTE MUST NOT PRINT')).toHaveCount(0);
  await expect(page.getByRole('button',{name:'Print session recap'})).toBeVisible();
});

test('@claim:json-export exports the complete demo record as JSON',async({page})=>{
  await page.goto('/settings?demo=1');
  await page.evaluate(()=>{
    const original=URL.createObjectURL.bind(URL);
    URL.createObjectURL=(blob:Blob)=>{
      (window as unknown as {exportedText?:Promise<string>}).exportedText=blob.text();
      return original(blob);
    };
    HTMLAnchorElement.prototype.click=function(){
      (window as unknown as {exportedName?:string}).exportedName=this.download;
    };
  });
  await page.getByRole('button',{name:'Export data'}).click();
  const exported=await page.evaluate(async()=>({name:(window as unknown as {exportedName:string}).exportedName,text:await (window as unknown as {exportedText:Promise<string>}).exportedText}));
  expect(exported.name).toBe('math-circle-board-export.json');
  const value=JSON.parse(exported.text);
  expect(value.group_name).toContain('Saturday Problem Circle');
  expect(value.learners).toHaveLength(3);
  expect(value.sessions).toHaveLength(2);
  expect(value.attempts[0].private_note).toContain('Ask Ada');
  expect(value.attachment_files).toEqual([]);
});

test('@claim:offline-reload demo reloads offline after its first visit',async({browser})=>{
  const context=await browser.newContext({viewport:{width:390,height:844}});
  const page=await context.newPage();
  await page.goto('/?demo=1');
  await page.evaluate(()=>navigator.serviceWorker.ready.then(()=>true));
  await page.reload();
  await expect.poll(()=>page.evaluate(()=>navigator.serviceWorker.controller!==null)).toBe(true);
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByRole('heading',{name:'The coin trail'})).toBeVisible();
  expect(await page.evaluate(()=>document.body.scrollWidth)).toBe(390);
  await context.close();
});

test('@claim:no-tracking landing and demo make no third-party requests',async({page})=>{
  const origins=new Set<string>();
  page.on('request',request=>origins.add(new URL(request.url()).origin));
  await page.goto('/');
  await page.getByRole('link',{name:'Try it with sample data'}).click();
  await page.getByRole('link',{name:'Learners'}).click();
  expect([...origins]).toEqual([new URL(page.url()).origin]);
});

test('@claim:owner-access private board API requires the signed-in Microsoft owner',async({page})=>{
  const response=await page.request.get('/api/board',{headers:{'x-forwarded-for':'198.51.100.81'}});
  expect(response.status()).toBe(401);
  expect(response.headers()['www-authenticate']).toContain('Bearer');
  await page.goto('/');
  await expect(page.getByRole('button',{name:'Sign in with Microsoft'})).toBeVisible();
  await expect(page.locator('input[type=password]')).toHaveCount(0);
  await page.getByRole('link',{name:'Try it with sample data'}).click();
  await page.getByRole('link',{name:'Learners'}).click();
  await expect(page.getByLabel('Learner alias')).toBeVisible();
  await expect(page.locator('input[type=email]')).toHaveCount(0);
});

test('@claim:first-boot-runtime starts with only PORT and creates the documented local storage',async()=>{
  const workDir=mkdtempSync(join(tmpdir(),'mcb-runtime-'));
  const port='18084';
  const binary=process.env.MCB_TEST_BACKEND_BIN||resolve('target/debug/math-circle-board');
  symlinkSync(resolve('dist'),join(workDir,'dist'),'dir');
  let logs='';
  const server=spawn(binary,[],{cwd:workDir,env:{PATH:process.env.PATH||'',PORT:port},stdio:['ignore','pipe','pipe']});
  server.stdout.on('data',chunk=>{logs+=String(chunk)});
  server.stderr.on('data',chunk=>{logs+=String(chunk)});
  try{
    await expect.poll(async()=>{try{return (await fetch(`http://127.0.0.1:${port}/health`)).ok}catch{return false}},{timeout:20_000}).toBe(true);
    const health=await fetch(`http://127.0.0.1:${port}/health`).then(response=>response.json()) as {ok:boolean,build:string};
    expect(health.ok).toBe(true);
    expect(health.build.length).toBeGreaterThan(0);
    expect(statSync(join(workDir,'data','board.db')).size).toBeGreaterThan(0);
    expect(statSync(join(workDir,'data','uploads')).isDirectory()).toBe(true);
    const ownerCode=readFileSync(join(workDir,'data','owner-invite.txt'),'utf8').trim();
    expect(ownerCode).toMatch(/^[a-f0-9]{48}$/);
    expect(statSync(join(workDir,'data','owner-invite.txt')).mode&0o777).toBe(0o600);
    expect(logs).toContain('sociobotcustomers.ciamlogin.com');
  }finally{
    server.kill('SIGTERM');
    await new Promise(resolveExit=>server.once('exit',resolveExit));
    rmSync(workDir,{recursive:true,force:true});
  }
  const overrideDir=mkdtempSync(join(tmpdir(),'mcb-runtime-overrides-'));
  const overrideData=join(overrideDir,'kept-data');
  const overridePort='18085';
  let overrideLogs='';
  const overrideServer=spawn(binary,[],{cwd:overrideDir,env:{
    PATH:process.env.PATH||'',PORT:overridePort,DATA_DIR:overrideData,DIST_DIR:resolve('dist'),
    MCB_OWNER_INVITE:'adult-override-code-0123456789',ENTRA_TENANT_ID:'test-tenant',
    ENTRA_TENANT_SUBDOMAIN:'test-authority',ENTRA_CLIENT_ID:'test-client',
  },stdio:['ignore','pipe','pipe']});
  overrideServer.stdout.on('data',chunk=>{overrideLogs+=String(chunk)});
  overrideServer.stderr.on('data',chunk=>{overrideLogs+=String(chunk)});
  try{
    await expect.poll(async()=>{try{return (await fetch(`http://127.0.0.1:${overridePort}/health`)).ok}catch{return false}},{timeout:20_000}).toBe(true);
    expect(statSync(join(overrideData,'board.db')).size).toBeGreaterThan(0);
    expect(statSync(join(overrideData,'uploads')).isDirectory()).toBe(true);
    expect(existsSync(join(overrideData,'owner-invite.txt'))).toBe(false);
    expect(overrideLogs).toContain('test-authority.ciamlogin.com');
    expect(overrideLogs).toContain('test-client');
    expect(overrideLogs).toContain('owner_invite_supplied');
  }finally{
    overrideServer.kill('SIGTERM');
    await new Promise(resolveExit=>overrideServer.once('exit',resolveExit));
    rmSync(overrideDir,{recursive:true,force:true});
  }
});

test('@claim:container-runtime container recipe preserves runtime identity and storage contracts',async({request})=>{
  const dockerfile=readFileSync('Dockerfile','utf8');
  expect(dockerfile).toMatch(/^FROM node:22-alpine AS web/m);
  expect(dockerfile).toMatch(/^FROM rust:1-alpine AS server/m);
  expect(dockerfile).toContain('ARG BUILD_SHA=unknown');
  expect(dockerfile).toContain('ENV DATA_DIR=/data DIST_DIR=/app/dist');
  expect(dockerfile).toContain('USER app');
  expect(dockerfile).toContain('EXPOSE 8080');
  expect(dockerfile).toContain('VOLUME ["/data"]');
  expect(dockerfile).not.toContain('COPY .git');
  const health=await request.get('/health');
  expect(health.status()).toBe(200);
  expect(await health.json()).toMatchObject({ok:true,build:expect.any(String)});
});

test('@claim:rate-limits read and write bursts return 429 with Retry-After',async({request})=>{
  const reads=await Promise.all(Array.from({length:100},()=>request.get('/api/status',{headers:{'x-forwarded-for':'203.0.113.81'}})));
  const readLimited=reads.filter(response=>response.status()===429);
  const readAllowed=reads.filter(response=>response.status()===200);
  expect(readAllowed.length).toBeGreaterThanOrEqual(40);
  expect(readAllowed.length).toBeLessThanOrEqual(45);
  expect(readLimited.length).toBeGreaterThan(0);
  expect(readLimited[0].headers()['retry-after']).toMatch(/^\d+$/);
  const writes=await Promise.all(Array.from({length:30},()=>request.post('/api/setup',{headers:{'x-forwarded-for':'203.0.113.82'},data:{}})));
  const writeLimited=writes.filter(response=>response.status()===429);
  const writeAllowed=writes.filter(response=>response.status()!==429);
  expect(writeAllowed.length).toBeGreaterThanOrEqual(8);
  expect(writeAllowed.length).toBeLessThanOrEqual(9);
  expect(writeLimited.length).toBeGreaterThan(0);
  expect(writeLimited[0].headers()['retry-after']).toMatch(/^\d+$/);
});

test('@claim:release-scope the public product has no untestable paid or organization tier',async({page})=>{
  await page.goto('/');
  await expect(page.getByText('All current board tools are free.')).toBeVisible();
  await expect(page.getByRole('heading',{name:'This release is for one private circle'})).toBeVisible();
  await expect(page.getByText('It has no paid plan, checkout, organization controls, or extra storage tier.')).toBeVisible();
  await expect(page.locator(`a[href="https://api.sociobot.in/api/v1/products/math-circle-board/checkout"]`)).toHaveCount(0);
  await page.goto('/terms');
  await expect(page.getByText('This release has no paid plan, checkout, organization controls, or extra storage tier.')).toBeVisible();
});

test('@claim:strategy-palette free strategy prompts add and save a reusable prompt',async({page})=>{
  await page.goto('/plus?demo=1');
  await expect(page.getByRole('heading',{name:'Four strategy prompts are included'})).toBeVisible();
  await page.getByRole('link',{name:'Board',exact:true}).click();
  const palette=page.getByLabel('Strategy prompt palette');
  await expect(palette.getByRole('button')).toHaveCount(4);
  await palette.getByRole('button',{name:'+ Draw a diagram',exact:true}).click();
  await expect(page.getByRole('button',{name:'Remove strategy Draw a diagram'})).toBeVisible();
  await page.getByRole('button',{name:'Save attempt'}).click();
  await expect(page.getByText('Attempt saved.')).toBeVisible();
  await page.reload();
  await expect(page.getByRole('button',{name:'Remove strategy Draw a diagram'})).toBeVisible();
});

test('legal, 404, mobile, keyboard, routes, and focus pass regression checks',async({browser})=>{
  const context=await browser.newContext({viewport:{width:390,height:844}});
  const page=await context.newPage();
  const errors:string[]=[];
  page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
  page.on('pageerror',error=>errors.push(error.message));
  for(const route of ['/privacy','/terms']){
    const response=await page.goto(route);
    expect(response?.status()).toBe(200);
    expect(await seriousAxe(page)).toEqual([]);
  }
  expect(errors).toEqual([]);
  const missing=await page.goto('/not-a-real-page');
  expect(missing?.status()).toBe(404);
  await expect(page.getByRole('heading',{level:1})).toHaveText('This page does not exist');
  expect(await seriousAxe(page)).toEqual([]);
  errors.length=0;
  await page.goto('/?demo=1');
  await page.keyboard.press('Tab');
  await expect(page.locator(':focus')).toHaveText('Skip to main content');
  await page.keyboard.press('Enter');
  await expect(page.locator('#main')).toBeFocused();
  await page.getByRole('link',{name:'Learners'}).focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/learners\?demo=1$/);
  await expect(page.locator('main h1')).toBeFocused();
  await page.goBack();
  await expect(page).toHaveURL(/\/board\?demo=1(?:#main)?$/);
  await expect(page.locator('main h1')).toBeFocused();
  await page.reload();
  await expect(page.getByRole('heading',{level:2,name:'Invariants in motion'})).toBeVisible();
  expect(await page.evaluate(()=>document.body.scrollWidth)).toBe(390);
  expect(await seriousAxe(page)).toEqual([]);
  expect(errors).toEqual([]);
  await context.close();
});

test('390 px public navigation shows every destination without horizontal clipping',async({browser})=>{
  const context=await browser.newContext({viewport:{width:390,height:844}});
  const page=await context.newPage();
  await page.goto('/');
  const nav=page.getByRole('navigation',{name:'Public navigation'});
  const navWidth=await nav.evaluate(element=>({clientWidth:element.clientWidth,scrollWidth:element.scrollWidth}));
  expect(navWidth.scrollWidth).toBeLessThanOrEqual(navWidth.clientWidth);
  const measurements=await nav.locator('a').evaluateAll(links=>links.map(link=>{const rect=link.getBoundingClientRect();return {label:link.textContent?.trim(),left:rect.left,right:rect.right,top:rect.top,bottom:rect.bottom,width:rect.width,height:rect.height}}));
  for(const link of measurements){
    expect(link.left,`${link.label} starts outside the viewport`).toBeGreaterThanOrEqual(0);
    expect(link.right,`${link.label} clips outside the viewport`).toBeLessThanOrEqual(390);
    expect(link.width,`${link.label} is too narrow`).toBeGreaterThanOrEqual(44);
    expect(link.height,`${link.label} is too short`).toBeGreaterThanOrEqual(44);
  }
  await context.close();
});

test('390 px app navigation shows every destination inside its 374 px content width',async({browser})=>{
  const context=await browser.newContext({viewport:{width:390,height:844}});
  const page=await context.newPage();
  await page.goto('/?demo=1');
  const nav=page.getByRole('navigation',{name:'Main navigation'});
  const navBox=await nav.evaluate(element=>{const rect=element.getBoundingClientRect();return {clientWidth:element.clientWidth,scrollWidth:element.scrollWidth,left:rect.left,right:rect.right}});
  expect(navBox.clientWidth).toBe(374);
  expect(navBox.scrollWidth).toBeLessThanOrEqual(navBox.clientWidth);
  const measurements=await nav.locator('a').evaluateAll(links=>links.map(link=>{const rect=link.getBoundingClientRect();return {label:link.textContent?.trim(),left:rect.left,right:rect.right,width:rect.width,height:rect.height}}));
  expect(measurements.map(link=>link.label)).toEqual(['Board','Learners','Strategies','Settings']);
  for(const link of measurements){
    expect(link.left,`${link.label} starts outside the navigation`).toBeGreaterThanOrEqual(navBox.left);
    expect(link.right,`${link.label} clips outside the navigation`).toBeLessThanOrEqual(navBox.right);
    expect(link.width,`${link.label} is too narrow`).toBeGreaterThanOrEqual(44);
    expect(link.height,`${link.label} is too short`).toBeGreaterThanOrEqual(44);
  }
  await context.close();
});

test('all visible mobile controls have at least a 44 by 44 CSS pixel target',async({browser})=>{
  const context=await browser.newContext({viewport:{width:390,height:844}});
  const page=await context.newPage();
  for(const route of ['/','/?demo=1','/learners?demo=1','/plus?demo=1','/settings?demo=1','/privacy','/terms','/not-a-real-page']){
    await page.goto(route);
    const smallTargets=await page.locator('a,button,input,select,textarea,label.upload-button').evaluateAll(nodes=>nodes.flatMap(node=>{
      const element=node as HTMLElement;
      const rect=element.getBoundingClientRect();
      const style=getComputedStyle(element);
      if(style.display==='none'||style.visibility==='hidden'||rect.width===0||rect.height===0)return [];
      if(element instanceof HTMLInputElement&&['hidden','radio','file'].includes(element.type))return [];
      if(rect.width>=44&&rect.height>=44)return [];
      return [{name:element.getAttribute('aria-label')||element.textContent?.trim()||element.getAttribute('placeholder')||element.tagName,width:rect.width,height:rect.height}];
    }));
    expect(smallTargets,`${route} has undersized controls`).toEqual([]);
  }
  await context.close();
});

test('@claim:full-delete the owner can delete the complete private board',async({page})=>{
  await authenticate(page);
  await page.goto('/');
  await page.getByLabel('Facilitator name').fill('Morgan');
  await page.getByLabel('Circle name').fill('Saturday Circle');
  await page.getByLabel('Adult setup code').fill(ownerCode);
  await page.getByLabel(/I am an adult responsible/).check();
  await page.getByRole('button',{name:'Create private board'}).click();
  await expect(page).toHaveURL(/\/board$/);
  await page.getByRole('button',{name:'New session'}).click();
  await page.getByLabel('Session title').fill('Parity paths');
  await page.getByLabel('Guiding focus').fill('What never changes?');
  await page.getByRole('button',{name:'Add session'}).click();
  await page.getByRole('button',{name:'Add problem'}).click();
  await page.getByLabel('Short title').fill('The coin trail');
  await page.getByLabel('Open prompt').fill('Move one coin at a time. Which arrangements can you reach?');
  await page.getByRole('button',{name:'Add to sequence'}).click();
  await page.getByRole('link',{name:'Learners'}).click();
  await page.getByLabel('Learner alias').fill('Ada');
  await page.getByRole('button',{name:'Add learner'}).click();
  await page.getByRole('link',{name:'Board',exact:true}).click();
  await page.getByLabel('What they tried').fill('Marked odd gaps and tested the smallest row first.');
  await page.getByLabel('Private facilitator note').fill('Ask for the invariant next time.');
  await page.getByText('◐ Exploring',{exact:true}).click();
  await page.getByRole('button',{name:'Save attempt'}).click();
  await page.getByLabel('Add photo').setInputFiles('frontend/public/art/lantern-room-768.webp');
  await expect(page.getByRole('img',{name:/Uploaded attempt/})).toBeVisible();
  const exported=await page.evaluate(async token=>{const response=await fetch('/api/export',{headers:{Authorization:`Bearer ${token}`}});return response.json()},authToken);
  expect(exported.attachment_files).toHaveLength(1);
  await page.getByRole('button',{name:'Print recap'}).click();
  await expect(page.getByText('Marked odd gaps and tested the smallest row first.')).toBeVisible();
  await expect(page.getByText('Ask for the invariant next time.')).toHaveCount(0);
  await page.getByRole('link',{name:'Settings'}).click();
  page.once('dialog',dialog=>dialog.accept());
  await page.getByRole('button',{name:'Delete the entire board'}).click();
  await expect(page.getByLabel('Facilitator name')).toBeVisible();
  const status=await page.evaluate(async token=>fetch('/api/status',{headers:{Authorization:`Bearer ${token}`}}).then(response=>response.json()),authToken);
  expect(status.configured).toBe(false);
});
