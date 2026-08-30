import {identityToken} from './auth';
import {demoApi,demoExport,demoFile,isDemoMode} from './demo';

export class ApiFailure extends Error{constructor(message:string,public status:number){super(message)}}

async function response(path:string,options:RequestInit={}):Promise<Response>{
  const token=await identityToken();
  const headers=new Headers(options.headers);
  if(!(options.body instanceof FormData))headers.set('Content-Type','application/json');
  if(token)headers.set('Authorization',`Bearer ${token}`);
  const result=await fetch(`/api${path}`,{...options,headers});
  if(!result.ok){let message='The board could not complete that request.';try{message=(await result.clone().json()).error||message}catch{}throw new ApiFailure(message,result.status)}
  return result;
}
export async function api<T>(path:string,options:RequestInit={}):Promise<T>{
  if(isDemoMode())return demoApi<T>(path,options);
  return (await response(path,options)).json() as Promise<T>;
}
export async function apiBlob(path:string):Promise<Blob>{if(isDemoMode())return path==='/export'?demoExport():demoFile(path);return (await response(path)).blob()}
export const send=<T>(path:string,data:unknown,method='POST')=>api<T>(path,{method,body:JSON.stringify(data)});
