import {test,expect,type Page} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

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
  await expect(page.getByRole('link',{name:'Try it with sample data'})).toHaveAttribute('href','/?demo=1');
  await expect(page.getByRole('button',{name:'Sign in with Microsoft'})).toBeVisible();
  await expect(page.locator('link[rel=canonical]')).toHaveAttribute('href','https://math-circle-board.sociobot.in/');
  await expect(page.locator('meta[property="og:image"]')).toHaveAttribute('content',/social-card\.webp$/);
  expect(await seriousAxe(page)).toEqual([]);
  expect(errors).toEqual([]);
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

test('demo rejects learner aliases case-insensitively like the backend',async({page})=>{
  await page.goto('/learners?demo=1');
  await page.getByLabel('Learner alias').fill('aDa');
  await page.getByRole('button',{name:'Add learner'}).click();
  await expect(page.locator('.add-learner .form-error')).toHaveText('That learner alias is already in the circle.');
  await expect(page.locator('.learner-list strong').filter({hasText:/^Ada$/i})).toHaveCount(1);
  await expect(page.locator('.learner-list > li')).toHaveCount(3);
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

test('@claim:owner-access private board API requires the signed-in owner',async({page})=>{
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

test('@claim:plus-price core board is free and Plus is a $39 one-time option',async({page})=>{
  await page.goto('/');
  await expect(page.getByText('Core board: free. Circle Plus: $39 once.')).toBeVisible();
  const buy=page.getByRole('link',{name:'Buy Circle Plus through Sociobot'});
  await expect(buy).toHaveAttribute('href','https://api.sociobot.in/api/v1/products/math-circle-board/checkout');
});

test('@claim:plus-strategy-palette Plus adds and saves a reusable strategy prompt',async({page})=>{
  await page.goto('/plus?demo=1');
  await expect(page.getByRole('heading',{name:'Four strategy prompts are available'})).toBeVisible();
  await page.getByRole('link',{name:'Board',exact:true}).click();
  const palette=page.getByLabel('Circle Plus strategy palette');
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
