import type {PublicClientApplication} from '@azure/msal-browser';

const tenantId='35c6fe40-0ec0-46b6-98c6-213ad4de6650';
const tenantSubdomain='sociobotcustomers';
const clientId='25c704f4-465a-47af-80ab-2c489466b697';
const scopes=['openid','profile','email'];
const testTokenKey='mcb:test-access-token';

export const authority=`https://${tenantSubdomain}.ciamlogin.com/${tenantId}/`;

let identityPromise:Promise<PublicClientApplication>|null=null;
function identityClient():Promise<PublicClientApplication>{
  if(!identityPromise)identityPromise=(async()=>{
    const {BrowserCacheLocation,PublicClientApplication}=await import('@azure/msal-browser');
    const identity=new PublicClientApplication({
      auth:{clientId,authority,redirectUri:`${location.origin}/auth/callback`,postLogoutRedirectUri:location.origin,navigateToLoginRequestUrl:true},
      cache:{cacheLocation:BrowserCacheLocation.SessionStorage},
    });
    await identity.initialize();
    const result=await identity.handleRedirectPromise();
    if(result?.account)identity.setActiveAccount(result.account);
    return identity;
  })();
  return identityPromise;
}

function hasIdentityState(){return location.pathname==='/auth/callback'||Object.keys(sessionStorage).some(key=>key.includes(clientId))}

export async function identityToken():Promise<string|null>{
  const injected=sessionStorage.getItem(testTokenKey);
  if(injected)return injected;
  if(!hasIdentityState())return null;
  const identity=await identityClient();
  const account=identity.getActiveAccount()||identity.getAllAccounts()[0];
  if(!account)return null;
  identity.setActiveAccount(account);
  try{
    const result=await identity.acquireTokenSilent({account,scopes});
    return result.idToken;
  }catch{return null}
}

export async function signIn():Promise<void>{
  const identity=await identityClient();
  await identity.loginRedirect({scopes,prompt:'select_account'});
}

export async function signOut():Promise<void>{
  const injected=sessionStorage.getItem(testTokenKey);
  if(injected){sessionStorage.removeItem(testTokenKey);location.assign('/');return}
  const identity=await identityClient();
  await identity.logoutRedirect({account:identity.getActiveAccount()||identity.getAllAccounts()[0]});
}
