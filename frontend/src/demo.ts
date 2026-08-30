import type {Board} from './types';

const demoKey='demo:math-circle-board:board';
const demoFilePrefix='demo:math-circle-board:file:';

export function isDemoMode():boolean{
  return location.pathname==='/demo'||new URLSearchParams(location.search).get('demo')==='1';
}

function seedBoard():Board{
  return {
    group_name:'Saturday Problem Circle — sample',
    learners:[
      {id:1,alias:'Ada',created_at:1_788_019_200},
      {id:2,alias:'Noor',created_at:1_788_019_260},
      {id:3,alias:'Milo',created_at:1_788_019_320},
    ],
    sessions:[
      {id:1,title:'Invariants in motion',session_date:'2026-08-29',focus:'What stays unchanged after every move?',created_at:1_788_019_400},
      {id:2,title:'Patterns in growing squares',session_date:'2026-08-22',focus:'How can one picture explain every case?',created_at:1_787_414_400},
    ],
    problems:[
      {id:1,session_id:1,position:1,title:'The coin trail',prompt:'Move one coin at a time. Which arrangements can you reach without changing the number of odd gaps?'},
      {id:2,session_id:1,position:2,title:'Corner cuts',prompt:'Cut one corner from a paper square. What remains unchanged after more cuts?'},
      {id:3,session_id:1,position:3,title:'Switching lamps',prompt:'Each move switches two lamps. Can exactly one lamp remain on?'},
      {id:4,session_id:2,position:1,title:'Growing borders',prompt:'Build square borders with tiles. How many new tiles does each border need?'},
    ],
    attempts:[
      {id:1,learner_id:1,problem_id:1,status:'exploring',thinking:'Marked the odd gaps, then tested rows of three and five coins.',strategies:'["Try a smaller case","Track what changes"]',private_note:'Ask Ada to explain why two gaps change together.',updated_at:1_788_019_700},
      {id:2,learner_id:2,problem_id:1,status:'shared',thinking:'Used red and blue marks to show that the number of odd gaps keeps the same parity.',strategies:'["Color-code cases","Look for an invariant"]',private_note:'Invite Noor to open the recap next week.',updated_at:1_788_019_760},
      {id:3,learner_id:3,problem_id:2,status:'exploring',thinking:'Drew four cuts and counted sides instead of corners.',strategies:'["Draw a diagram"]',private_note:'Offer tracing paper if Milo returns to this problem.',updated_at:1_788_019_820},
      {id:4,learner_id:1,problem_id:4,status:'shared',thinking:'Rearranged each border into four strips and corrected for the corners.',strategies:'["Rearrange the pieces"]',private_note:'',updated_at:1_787_414_700},
    ],
    attachments:[],
  };
}

export function resetDemo():Board{
  const value=seedBoard();
  sessionStorage.setItem(demoKey,JSON.stringify(value));
  return value;
}

export function clearDemo():void{
  for(let index=sessionStorage.length-1;index>=0;index--){
    const key=sessionStorage.key(index);
    if(key?.startsWith('demo:math-circle-board'))sessionStorage.removeItem(key);
  }
}

function readBoard():Board{
  const stored=sessionStorage.getItem(demoKey);
  if(!stored)return resetDemo();
  try{return JSON.parse(stored) as Board}catch{return resetDemo()}
}

function writeBoard(value:Board):void{
  sessionStorage.setItem(demoKey,JSON.stringify(value));
}

function bodyOf(options:RequestInit):Record<string,unknown>{
  if(typeof options.body!=='string')return {};
  try{return JSON.parse(options.body) as Record<string,unknown>}catch{return {}}
}

function nextId(items:Array<{id:number}>):number{
  return Math.max(0,...items.map(item=>item.id))+1;
}

export async function demoApi<T>(path:string,options:RequestInit={}):Promise<T>{
  const method=(options.method||'GET').toUpperCase();
  if(path==='/status')return {configured:true,signed_in:true,authenticated:true,facilitator:'Morgan'} as T;
  if(path==='/board'&&method==='GET')return readBoard() as T;
  const current=readBoard();
  const input=bodyOf(options);
  if(path==='/learners'&&method==='POST'){
    const id=nextId(current.learners);
    current.learners.push({id,alias:String(input.alias||'').trim(),created_at:Date.now()/1000});
    writeBoard(current);return {id} as T;
  }
  const learnerMatch=path.match(/^\/learners\/(\d+)$/);
  if(learnerMatch&&method==='DELETE'){
    const id=Number(learnerMatch[1]);
    const attemptIds=current.attempts.filter(item=>item.learner_id===id).map(item=>item.id);
    current.learners=current.learners.filter(item=>item.id!==id);
    current.attempts=current.attempts.filter(item=>item.learner_id!==id);
    current.attachments=current.attachments.filter(item=>!attemptIds.includes(item.attempt_id));
    writeBoard(current);return {ok:true} as T;
  }
  if(path==='/sessions'&&method==='POST'){
    const id=nextId(current.sessions);
    current.sessions.unshift({id,title:String(input.title||'').trim(),session_date:String(input.session_date||''),focus:String(input.focus||'').trim(),created_at:Date.now()/1000});
    writeBoard(current);return {id} as T;
  }
  const sessionMatch=path.match(/^\/sessions\/(\d+)$/);
  if(sessionMatch&&method==='DELETE'){
    const id=Number(sessionMatch[1]);
    const problemIds=current.problems.filter(item=>item.session_id===id).map(item=>item.id);
    const attemptIds=current.attempts.filter(item=>problemIds.includes(item.problem_id)).map(item=>item.id);
    current.sessions=current.sessions.filter(item=>item.id!==id);
    current.problems=current.problems.filter(item=>item.session_id!==id);
    current.attempts=current.attempts.filter(item=>!problemIds.includes(item.problem_id));
    current.attachments=current.attachments.filter(item=>!attemptIds.includes(item.attempt_id));
    writeBoard(current);return {ok:true} as T;
  }
  if(path==='/problems'&&method==='POST'){
    const id=nextId(current.problems);
    const sessionId=Number(input.session_id);
    const position=current.problems.filter(item=>item.session_id===sessionId).length+1;
    current.problems.push({id,session_id:sessionId,position,title:String(input.title||'').trim(),prompt:String(input.prompt||'').trim()});
    writeBoard(current);return {id} as T;
  }
  if(path==='/attempts'&&method==='POST'){
    const learnerId=Number(input.learner_id),problemId=Number(input.problem_id);
    let item=current.attempts.find(attempt=>attempt.learner_id===learnerId&&attempt.problem_id===problemId);
    if(!item){item={id:nextId(current.attempts),learner_id:learnerId,problem_id:problemId,status:'not_started',thinking:'',strategies:'[]',private_note:'',updated_at:0};current.attempts.push(item)}
    item.status=String(input.status||'not_started') as typeof item.status;
    item.thinking=String(input.thinking||'').trim();
    item.strategies=JSON.stringify(Array.isArray(input.strategies)?input.strategies:[]);
    item.private_note=String(input.private_note||'').trim();
    item.updated_at=Math.floor(Date.now()/1000);
    writeBoard(current);return {id:item.id,updated_at:item.updated_at} as T;
  }
  const uploadMatch=path.match(/^\/attempts\/(\d+)\/upload$/);
  if(uploadMatch&&method==='POST'&&options.body instanceof FormData){
    const file=options.body.get('image');
    if(!(file instanceof File)||!['image/jpeg','image/png','image/webp'].includes(file.type)||file.size>5*1024*1024)throw new Error('Use a JPEG, PNG, or WebP image under 5 MB.');
    const id=nextId(current.attachments);
    const bytes=new Uint8Array(await file.arrayBuffer());
    let binary='';
    for(const byte of bytes)binary+=String.fromCharCode(byte);
    sessionStorage.setItem(`${demoFilePrefix}${id}`,JSON.stringify({mime:file.type,data:btoa(binary)}));
    current.attachments.push({id,attempt_id:Number(uploadMatch[1]),original_name:file.name, mime:file.type,created_at:Math.floor(Date.now()/1000)});
    writeBoard(current);return {id} as T;
  }
  const fileMatch=path.match(/^\/files\/(\d+)$/);
  if(fileMatch&&method==='DELETE'){
    const id=Number(fileMatch[1]);
    current.attachments=current.attachments.filter(item=>item.id!==id);
    sessionStorage.removeItem(`${demoFilePrefix}${id}`);
    writeBoard(current);return {ok:true} as T;
  }
  if(path==='/board'&&method==='DELETE'){
    clearDemo();return {ok:true} as T;
  }
  throw new Error('That action is not available in the sample demo.');
}

export function demoExport():Blob{
  const current=readBoard();
  const attachment_files=current.attachments.map(item=>{
    const stored=JSON.parse(sessionStorage.getItem(`${demoFilePrefix}${item.id}`)||'{"data":""}') as {data:string};
    return {id:item.id,original_name:item.original_name,mime:item.mime,data_base64:stored.data};
  });
  return new Blob([JSON.stringify({...current,attachment_files},null,2)],{type:'application/json'});
}

export function demoFile(path:string):Blob{
  const id=Number(path.match(/^\/files\/(\d+)$/)?.[1]);
  const stored=JSON.parse(sessionStorage.getItem(`${demoFilePrefix}${id}`)||'null') as {mime:string;data:string}|null;
  if(!stored)throw new Error('Sample image not found.');
  const binary=atob(stored.data);
  return new Blob([Uint8Array.from(binary,character=>character.charCodeAt(0))],{type:stored.mime});
}
