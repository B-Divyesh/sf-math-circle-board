export type Learner={id:number;alias:string;created_at:number};
export type CircleSession={id:number;title:string;session_date:string;focus:string;created_at:number};
export type Problem={id:number;session_id:number;position:number;title:string;prompt:string};
export type Attempt={id:number;learner_id:number;problem_id:number;status:'not_started'|'exploring'|'shared';thinking:string;strategies:string;private_note:string;updated_at:number};
export type Attachment={id:number;attempt_id:number;original_name:string;mime:string;created_at:number};
export type Board={group_name:string;learners:Learner[];sessions:CircleSession[];problems:Problem[];attempts:Attempt[];attachments:Attachment[]};
export type Status={configured:boolean;authenticated:boolean;facilitator?:string};

export const parseStrategies=(raw:string):string[]=>{try{const value=JSON.parse(raw);return Array.isArray(value)?value.filter(v=>typeof v==='string'):[]}catch{return[]}};
export const statusLabel=(status:Attempt['status'])=>status==='shared'?'✓ Shared':status==='exploring'?'◐ Exploring':'○ Not started';
export const shortDate=(iso:string)=>new Intl.DateTimeFormat(undefined,{month:'short',day:'numeric',year:'numeric'}).format(new Date(`${iso}T12:00:00`));
export const escapeHtml=(value:string)=>value.replace(/[&<>'"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]!));
