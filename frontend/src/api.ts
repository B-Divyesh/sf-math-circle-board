export class ApiFailure extends Error{constructor(message:string,public status:number){super(message)}}
export async function api<T>(path:string,options:RequestInit={}):Promise<T>{
  const response=await fetch(`/api${path}`,{...options,headers:{...(options.body instanceof FormData?{}:{'Content-Type':'application/json'}),...options.headers}});
  if(!response.ok){let message='The board could not complete that request.';try{message=(await response.json()).error||message}catch{}throw new ApiFailure(message,response.status)}
  return response.json() as Promise<T>;
}
export const send=<T>(path:string,data:unknown,method='POST')=>api<T>(path,{method,body:JSON.stringify(data)});
