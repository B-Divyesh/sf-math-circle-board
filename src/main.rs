use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    FromRow, SqlitePool,
};
use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{net::TcpListener, sync::RwLock};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    upload_dir: PathBuf,
    owner_invite_hash: Arc<RwLock<Option<String>>>,
    owner_invite_path: PathBuf,
    auth: AuthState,
}
type Shared = Arc<AppState>;

#[derive(Debug)]
struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (self.0, Json(json!({"error": self.1}))).into_response();
        if response.status() == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"math-circle-board\""),
            );
        }
        response
    }
}
type ApiResult<T> = Result<T, ApiError>;

const DEFAULT_TENANT_ID: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const DEFAULT_TENANT_SUBDOMAIN: &str = "sociobotcustomers";
const DEFAULT_CLIENT_ID: &str = "25c704f4-465a-47af-80ab-2c489466b697";

#[derive(Clone)]
struct AuthState {
    tenant_id: String,
    client_id: String,
    discovery_url: String,
    metadata: Arc<RwLock<Option<OidcMetadata>>>,
    keys: Arc<RwLock<HashMap<String, DecodingKey>>>,
    refreshed_at: Arc<RwLock<i64>>,
    client: reqwest::Client,
    #[cfg(any(test, feature = "test-auth"))]
    test_token: Option<String>,
}

#[derive(Clone, Deserialize)]
struct OidcMetadata {
    issuer: String,
    jwks_uri: String,
}

#[derive(Debug, Deserialize)]
struct EntraClaims {
    aud: String,
    exp: usize,
    iss: String,
    nbf: Option<usize>,
    oid: String,
    tid: String,
}

impl AuthState {
    fn from_env() -> Self {
        let tenant_id = env::var("ENTRA_TENANT_ID").unwrap_or_else(|_| DEFAULT_TENANT_ID.into());
        let subdomain =
            env::var("ENTRA_TENANT_SUBDOMAIN").unwrap_or_else(|_| DEFAULT_TENANT_SUBDOMAIN.into());
        let client_id = env::var("ENTRA_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.into());
        Self {
            discovery_url: format!(
                "https://{subdomain}.ciamlogin.com/{tenant_id}/v2.0/.well-known/openid-configuration"
            ),
            tenant_id,
            client_id,
            metadata: Arc::new(RwLock::new(None)),
            keys: Arc::new(RwLock::new(HashMap::new())),
            refreshed_at: Arc::new(RwLock::new(0)),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .expect("build identity client"),
            #[cfg(any(test, feature = "test-auth"))]
            test_token: env::var("MCB_TEST_AUTH_TOKEN").ok(),
        }
    }

    async fn refresh(&self) -> Result<(), String> {
        let metadata = self
            .client
            .get(&self.discovery_url)
            .send()
            .await
            .map_err(|e| format!("OIDC discovery failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("OIDC discovery failed: {e}"))?
            .json::<OidcMetadata>()
            .await
            .map_err(|e| format!("OIDC discovery was invalid: {e}"))?;
        let jwks = self
            .client
            .get(&metadata.jwks_uri)
            .send()
            .await
            .map_err(|e| format!("OIDC keys failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("OIDC keys failed: {e}"))?
            .json::<JwkSet>()
            .await
            .map_err(|e| format!("OIDC keys were invalid: {e}"))?;
        let mut keys = HashMap::new();
        for jwk in &jwks.keys {
            if let (Some(kid), Ok(key)) = (jwk.common.key_id.as_ref(), DecodingKey::from_jwk(jwk)) {
                keys.insert(kid.clone(), key);
            }
        }
        if keys.is_empty() {
            return Err("OIDC returned no usable signing keys".into());
        }
        *self.metadata.write().await = Some(metadata);
        *self.keys.write().await = keys;
        *self.refreshed_at.write().await = now();
        Ok(())
    }

    async fn validate(&self, token: &str) -> Result<String, ApiError> {
        #[cfg(any(test, feature = "test-auth"))]
        if self.test_token.as_deref() == Some(token) {
            return Ok("00000000-0000-4000-8000-000000000001".into());
        }
        let header = decode_header(token).map_err(|_| unauthorized())?;
        if header.alg != Algorithm::RS256 {
            return Err(unauthorized());
        }
        let kid = header.kid.ok_or_else(unauthorized)?;
        let stale = now() - *self.refreshed_at.read().await >= 3600;
        if stale || !self.keys.read().await.contains_key(&kid) {
            self.refresh().await.map_err(|error| {
                tracing::error!(%error, "could not refresh Entra signing keys");
                ApiError(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Sign-in verification is temporarily unavailable. Try again.".into(),
                )
            })?;
        }
        let metadata = self
            .metadata
            .read()
            .await
            .clone()
            .ok_or_else(unauthorized)?;
        let key = self
            .keys
            .read()
            .await
            .get(&kid)
            .cloned()
            .ok_or_else(unauthorized)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.client_id]);
        validation.set_issuer(&[&metadata.issuer]);
        validation.set_required_spec_claims(&["aud", "exp", "iss", "nbf", "oid", "tid"]);
        validation.validate_nbf = true;
        validation.leeway = 60;
        let claims = decode::<EntraClaims>(token, &key, &validation)
            .map_err(|_| unauthorized())?
            .claims;
        if claims.tid != self.tenant_id
            || claims.aud != self.client_id
            || claims.iss != metadata.issuer
            || claims.oid.trim().is_empty()
            || claims.exp == 0
            || claims.nbf.is_none()
        {
            return Err(unauthorized());
        }
        Ok(claims.oid)
    }
}

fn unauthorized() -> ApiError {
    ApiError(
        StatusCode::UNAUTHORIZED,
        "Sign in with the circle owner’s Microsoft account to continue.".into(),
    )
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn random_hex(bytes: usize) -> String {
    let mut data = vec![0u8; bytes];
    rand::rng().fill_bytes(&mut data);
    hex::encode(data)
}
fn password_hash(pass: &str, salt: &str) -> String {
    let mut block = format!("{salt}:{pass}").into_bytes();
    for _ in 0..120_000 {
        block = Sha256::digest(&block).to_vec();
    }
    hex::encode(block)
}
fn valid_label(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.trim().chars().count() <= max
}
fn valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, byte)| matches!(i, 4 | 7) || byte.is_ascii_digit())
    {
        return false;
    }
    let year = value[0..4].parse::<u32>().ok();
    let month = value[5..7].parse::<u32>().ok();
    let day = value[8..10].parse::<u32>().ok();
    let (Some(year), Some(month), Some(day)) = (year, month, day) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    year > 0 && (1..=days).contains(&day)
}
fn detected_image_mime(bytes: &[u8]) -> Option<&'static str> {
    let format = image::guess_format(bytes).ok()?;
    let mime = match format {
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::WebP => "image/webp",
        _ => return None,
    };
    image::load_from_memory_with_format(bytes, format).ok()?;
    Some(mime)
}
fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}
async fn identity(state: &Shared, headers: &HeaderMap) -> ApiResult<String> {
    state
        .auth
        .validate(bearer(headers).ok_or_else(unauthorized)?)
        .await
}
async fn require_auth(state: &Shared, headers: &HeaderMap) -> ApiResult<String> {
    let oid = identity(state, headers).await?;
    let owner_oid: Option<String> =
        sqlx::query_scalar("SELECT owner_oid FROM settings WHERE id=1 AND owner_oid<>''")
            .fetch_optional(&state.db)
            .await
            .map_err(db_err)?;
    if owner_oid.as_deref() != Some(&oid) {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "This Microsoft account does not own this private circle.".into(),
        ));
    }
    Ok(oid)
}
fn db_err(e: sqlx::Error) -> ApiError {
    tracing::error!(error=%e,"database error");
    ApiError(
        StatusCode::INTERNAL_SERVER_ERROR,
        "The board could not save that change. Try again.".into(),
    )
}

#[derive(Deserialize)]
struct SetupInput {
    facilitator: String,
    group_name: String,
    owner_code: String,
    adult_confirmed: bool,
}

async fn status(State(s): State<Shared>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    let configured: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings WHERE owner_oid<>''")
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    let facilitator: Option<String> =
        sqlx::query_scalar("SELECT facilitator FROM settings LIMIT 1")
            .fetch_optional(&s.db)
            .await
            .map_err(db_err)?;
    let oid = match bearer(&headers) {
        Some(_) => Some(identity(&s, &headers).await?),
        None => None,
    };
    let signed_in = oid.is_some();
    let owner_oid: Option<String> =
        sqlx::query_scalar("SELECT owner_oid FROM settings WHERE id=1 AND owner_oid<>''")
            .fetch_optional(&s.db)
            .await
            .map_err(db_err)?;
    let authenticated = oid
        .as_deref()
        .is_some_and(|id| owner_oid.as_deref() == Some(id));
    Ok(Json(
        json!({"configured":configured>0,"signed_in":signed_in,"authenticated":authenticated,"facilitator":facilitator}),
    ))
}
async fn setup(
    State(s): State<Shared>,
    headers: HeaderMap,
    Json(input): Json<SetupInput>,
) -> ApiResult<impl IntoResponse> {
    let owner_oid = identity(&s, &headers).await?;
    if !valid_label(&input.facilitator, 80)
        || !valid_label(&input.group_name, 100)
        || !input.adult_confirmed
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Use a facilitator name, group name, and the adult responsibility confirmation.".into(),
        ));
    }
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings WHERE owner_oid<>''")
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    if exists > 0 {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "This board is already owned.".into(),
        ));
    }
    let Some(owner_invite_hash) = s.owner_invite_hash.read().await.clone() else {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "This deployment needs an installer-issued adult setup code before it can be claimed."
                .into(),
        ));
    };
    if password_hash(&input.owner_code, "mcb-owner-invite") != owner_invite_hash {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "That adult setup code did not match this deployment. Ask the deployment operator for the code.".into(),
        ));
    }
    let mut tx = s.db.begin().await.map_err(db_err)?;
    sqlx::query("INSERT INTO settings(id,facilitator,group_name,owner_oid,created_at) VALUES(1,?,?,?,?) ON CONFLICT(id) DO UPDATE SET facilitator=excluded.facilitator,group_name=excluded.group_name,owner_oid=excluded.owner_oid")
        .bind(input.facilitator.trim()).bind(input.group_name.trim()).bind(owner_oid).bind(now()).execute(&mut *tx).await.map_err(db_err)?;
    tx.commit().await.map_err(db_err)?;
    *s.owner_invite_hash.write().await = None;
    let _ = tokio::fs::remove_file(&s.owner_invite_path).await;
    Ok(Json(json!({"ok":true})))
}

#[derive(Serialize, FromRow)]
struct Learner {
    id: i64,
    alias: String,
    created_at: i64,
}
#[derive(Serialize, FromRow)]
struct CircleSession {
    id: i64,
    title: String,
    session_date: String,
    focus: String,
    created_at: i64,
}
#[derive(Serialize, FromRow)]
struct Problem {
    id: i64,
    session_id: i64,
    position: i64,
    title: String,
    prompt: String,
}
#[derive(Serialize, FromRow)]
struct Attempt {
    id: i64,
    learner_id: i64,
    problem_id: i64,
    status: String,
    thinking: String,
    strategies: String,
    private_note: String,
    updated_at: i64,
}
#[derive(Serialize, FromRow)]
struct Attachment {
    id: i64,
    attempt_id: i64,
    original_name: String,
    mime: String,
    created_at: i64,
}
async fn board(State(s): State<Shared>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    let group_name: String = sqlx::query_scalar("SELECT group_name FROM settings WHERE id=1")
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    let learners = sqlx::query_as::<_, Learner>(
        "SELECT id,alias,created_at FROM learners ORDER BY alias COLLATE NOCASE",
    )
    .fetch_all(&s.db)
    .await
    .map_err(db_err)?;
    let sessions=sqlx::query_as::<_,CircleSession>("SELECT id,title,session_date,focus,created_at FROM circle_sessions ORDER BY session_date DESC,id DESC").fetch_all(&s.db).await.map_err(db_err)?;
    let problems = sqlx::query_as::<_, Problem>(
        "SELECT id,session_id,position,title,prompt FROM problems ORDER BY session_id,position,id",
    )
    .fetch_all(&s.db)
    .await
    .map_err(db_err)?;
    let attempts=sqlx::query_as::<_,Attempt>("SELECT id,learner_id,problem_id,status,thinking,strategies,private_note,updated_at FROM attempts").fetch_all(&s.db).await.map_err(db_err)?;
    let attachments = sqlx::query_as::<_, Attachment>(
        "SELECT id,attempt_id,original_name,mime,created_at FROM attachments",
    )
    .fetch_all(&s.db)
    .await
    .map_err(db_err)?;
    Ok(Json(
        json!({"group_name":group_name,"learners":learners,"sessions":sessions,"problems":problems,"attempts":attempts,"attachments":attachments}),
    ))
}

#[derive(Deserialize)]
struct LearnerInput {
    alias: String,
}
async fn add_learner(
    State(s): State<Shared>,
    headers: HeaderMap,
    Json(v): Json<LearnerInput>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    if !valid_label(&v.alias, 60) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Enter a learner alias of 60 characters or fewer.".into(),
        ));
    }
    let learner_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM learners")
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    if learner_count >= 12 {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "A private circle can have up to 12 learner aliases.".into(),
        ));
    }
    let res = sqlx::query("INSERT INTO learners(alias,created_at) VALUES(?,?)")
        .bind(v.alias.trim())
        .bind(now())
        .execute(&s.db)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                ApiError(
                    StatusCode::CONFLICT,
                    "That learner alias is already in the circle.".into(),
                )
            } else if e.to_string().contains("learner_limit") {
                ApiError(
                    StatusCode::CONFLICT,
                    "A private circle can have up to 12 learner aliases.".into(),
                )
            } else {
                db_err(e)
            }
        })?;
    Ok(Json(json!({"id":res.last_insert_rowid()})))
}
async fn delete_learner(
    State(s): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    let files: Vec<String> = sqlx::query_scalar("SELECT a.stored_name FROM attachments a JOIN attempts t ON t.id=a.attempt_id WHERE t.learner_id=?")
        .bind(id)
        .fetch_all(&s.db)
        .await
        .map_err(db_err)?;
    sqlx::query("DELETE FROM learners WHERE id=?")
        .bind(id)
        .execute(&s.db)
        .await
        .map_err(db_err)?;
    remove_files(&s.upload_dir, files).await;
    Ok(Json(json!({"ok":true})))
}

async fn delete_board(State(s): State<Shared>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    let files: Vec<String> = sqlx::query_scalar("SELECT stored_name FROM attachments")
        .fetch_all(&s.db)
        .await
        .map_err(db_err)?;
    let mut tx = s.db.begin().await.map_err(db_err)?;
    for statement in [
        "DELETE FROM attachments",
        "DELETE FROM attempts",
        "DELETE FROM problems",
        "DELETE FROM circle_sessions",
        "DELETE FROM learners",
        "DELETE FROM settings",
    ] {
        sqlx::query(statement)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
    }
    tx.commit().await.map_err(db_err)?;
    remove_files(&s.upload_dir, files).await;
    let code = random_hex(24);
    tokio::fs::write(&s.owner_invite_path, &code)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The board was deleted, but a new owner code could not be created. Restart the service before setup.".into(),
            )
        })?;
    #[cfg(unix)]
    tokio::fs::set_permissions(
        &s.owner_invite_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .await
    .map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The board was deleted, but the new owner code could not be secured. Restart the service before setup.".into(),
        )
    })?;
    *s.owner_invite_hash.write().await = Some(password_hash(&code, "mcb-owner-invite"));
    tracing::info!("private board deleted; a new adult setup code was generated");
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
struct SessionInput {
    title: String,
    session_date: String,
    focus: String,
}
async fn add_session(
    State(s): State<Shared>,
    headers: HeaderMap,
    Json(v): Json<SessionInput>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    if !valid_label(&v.title, 100)
        || !valid_iso_date(&v.session_date)
        || v.focus.chars().count() > 300
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Enter a title, a real calendar date, and a focus under 300 characters.".into(),
        ));
    }
    let res = sqlx::query(
        "INSERT INTO circle_sessions(title,session_date,focus,created_at) VALUES(?,?,?,?)",
    )
    .bind(v.title.trim())
    .bind(v.session_date)
    .bind(v.focus.trim())
    .bind(now())
    .execute(&s.db)
    .await
    .map_err(db_err)?;
    Ok(Json(json!({"id":res.last_insert_rowid()})))
}
async fn delete_session(
    State(s): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    let files: Vec<String> = sqlx::query_scalar("SELECT a.stored_name FROM attachments a JOIN attempts t ON t.id=a.attempt_id JOIN problems p ON p.id=t.problem_id WHERE p.session_id=?")
        .bind(id)
        .fetch_all(&s.db)
        .await
        .map_err(db_err)?;
    sqlx::query("DELETE FROM circle_sessions WHERE id=?")
        .bind(id)
        .execute(&s.db)
        .await
        .map_err(db_err)?;
    remove_files(&s.upload_dir, files).await;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
struct ProblemInput {
    session_id: i64,
    title: String,
    prompt: String,
}
async fn add_problem(
    State(s): State<Shared>,
    headers: HeaderMap,
    Json(v): Json<ProblemInput>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    if !valid_label(&v.title, 120) || !valid_label(&v.prompt, 2000) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Add a problem title and prompt.".into(),
        ));
    }
    let position: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(position),0)+1 FROM problems WHERE session_id=?")
            .bind(v.session_id)
            .fetch_one(&s.db)
            .await
            .map_err(db_err)?;
    let res = sqlx::query("INSERT INTO problems(session_id,position,title,prompt) VALUES(?,?,?,?)")
        .bind(v.session_id)
        .bind(position)
        .bind(v.title.trim())
        .bind(v.prompt.trim())
        .execute(&s.db)
        .await
        .map_err(db_err)?;
    Ok(Json(json!({"id":res.last_insert_rowid()})))
}
async fn delete_problem(
    State(s): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    let files: Vec<String> = sqlx::query_scalar("SELECT a.stored_name FROM attachments a JOIN attempts t ON t.id=a.attempt_id WHERE t.problem_id=?")
        .bind(id)
        .fetch_all(&s.db)
        .await
        .map_err(db_err)?;
    sqlx::query("DELETE FROM problems WHERE id=?")
        .bind(id)
        .execute(&s.db)
        .await
        .map_err(db_err)?;
    remove_files(&s.upload_dir, files).await;
    Ok(Json(json!({"ok":true})))
}

async fn remove_files(upload_dir: &std::path::Path, files: Vec<String>) {
    for stored in files {
        let _ = tokio::fs::remove_file(upload_dir.join(stored)).await;
    }
}

#[derive(Deserialize)]
struct AttemptInput {
    learner_id: i64,
    problem_id: i64,
    status: String,
    thinking: String,
    strategies: Vec<String>,
    private_note: String,
}
async fn save_attempt(
    State(s): State<Shared>,
    headers: HeaderMap,
    Json(v): Json<AttemptInput>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    if !["not_started", "exploring", "shared"].contains(&v.status.as_str())
        || v.thinking.chars().count() > 5000
        || v.private_note.chars().count() > 3000
        || v.strategies.len() > 12
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "The attempt is too long or has an unknown status.".into(),
        ));
    }
    let strategies = serde_json::to_string(&v.strategies).unwrap_or("[]".into());
    sqlx::query("INSERT INTO attempts(learner_id,problem_id,status,thinking,strategies,private_note,updated_at) VALUES(?,?,?,?,?,?,?) ON CONFLICT(learner_id,problem_id) DO UPDATE SET status=excluded.status,thinking=excluded.thinking,strategies=excluded.strategies,private_note=excluded.private_note,updated_at=excluded.updated_at")
        .bind(v.learner_id).bind(v.problem_id).bind(v.status).bind(v.thinking.trim()).bind(strategies).bind(v.private_note.trim()).bind(now()).execute(&s.db).await.map_err(db_err)?;
    let id: i64 = sqlx::query_scalar("SELECT id FROM attempts WHERE learner_id=? AND problem_id=?")
        .bind(v.learner_id)
        .bind(v.problem_id)
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    Ok(Json(json!({"id":id,"updated_at":now()})))
}
async fn upload(
    State(s): State<Shared>,
    headers: HeaderMap,
    Path(attempt_id): Path<i64>,
    mut multipart: Multipart,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attachments WHERE attempt_id=?")
        .bind(attempt_id)
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    if count >= 4 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Each attempt can hold up to four images.".into(),
        ));
    }
    let field = multipart
        .next_field()
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_REQUEST,
                "The image could not be read.".into(),
            )
        })?
        .ok_or(ApiError(
            StatusCode::BAD_REQUEST,
            "Choose an image to upload.".into(),
        ))?;
    let name = field
        .file_name()
        .unwrap_or("attempt-image")
        .chars()
        .take(120)
        .collect::<String>();
    let bytes = field.bytes().await.map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "The image could not be read.".into(),
        )
    })?;
    if bytes.len() > 5 * 1024 * 1024 {
        return Err(ApiError(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Keep each image under 5 MB.".into(),
        ));
    }
    let mime = detected_image_mime(&bytes).ok_or(ApiError(
        StatusCode::BAD_REQUEST,
        "Use a valid JPEG, PNG, or WebP image file.".into(),
    ))?;
    let stored = random_hex(24);
    tokio::fs::write(s.upload_dir.join(&stored), &bytes)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The image could not be stored.".into(),
            )
        })?;
    let res=sqlx::query("INSERT INTO attachments(attempt_id,stored_name,original_name,mime,created_at) VALUES(?,?,?,?,?)").bind(attempt_id).bind(stored).bind(name).bind(mime).bind(now()).execute(&s.db).await.map_err(db_err)?;
    Ok(Json(json!({"id":res.last_insert_rowid()})))
}
async fn file(
    State(s): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Response> {
    require_auth(&s, &headers).await?;
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT stored_name,mime FROM attachments WHERE id=?")
            .bind(id)
            .fetch_optional(&s.db)
            .await
            .map_err(db_err)?;
    let Some((stored, mime)) = row else {
        return Err(ApiError(StatusCode::NOT_FOUND, "Image not found.".into()));
    };
    let bytes = tokio::fs::read(s.upload_dir.join(stored))
        .await
        .map_err(|_| ApiError(StatusCode::NOT_FOUND, "Image not found.".into()))?;
    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "private, max-age=3600".into()),
        ],
        bytes,
    )
        .into_response())
}
async fn delete_file(
    State(s): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    require_auth(&s, &headers).await?;
    if let Some(stored) =
        sqlx::query_scalar::<_, String>("SELECT stored_name FROM attachments WHERE id=?")
            .bind(id)
            .fetch_optional(&s.db)
            .await
            .map_err(db_err)?
    {
        let _ = tokio::fs::remove_file(s.upload_dir.join(stored)).await;
    }
    sqlx::query("DELETE FROM attachments WHERE id=?")
        .bind(id)
        .execute(&s.db)
        .await
        .map_err(db_err)?;
    Ok(Json(json!({"ok":true})))
}
async fn export_data(State(s): State<Shared>, headers: HeaderMap) -> ApiResult<Response> {
    let mut data = board(State(s.clone()), headers).await?.0;
    let rows: Vec<(i64, String, String, String)> =
        sqlx::query_as("SELECT id,stored_name,original_name,mime FROM attachments ORDER BY id")
            .fetch_all(&s.db)
            .await
            .map_err(db_err)?;
    let mut files = Vec::with_capacity(rows.len());
    for (id, stored, original_name, mime) in rows {
        let bytes = tokio::fs::read(s.upload_dir.join(stored))
            .await
            .map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An attachment could not be included in the export.".into(),
                )
            })?;
        files.push(json!({"id":id,"original_name":original_name,"mime":mime,"data_base64":BASE64.encode(bytes)}));
    }
    data["attachment_files"] = json!(files);
    let body = serde_json::to_vec_pretty(&data).unwrap();
    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=math-circle-board-export.json",
            ),
        ],
        body,
    )
        .into_response())
}

async fn health() -> Json<Value> {
    Json(json!({"ok":true,"build":option_env!("BUILD_SHA").unwrap_or("development")}))
}
async fn migrate(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA foreign_keys=ON").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS settings(id INTEGER PRIMARY KEY CHECK(id=1),facilitator TEXT NOT NULL,group_name TEXT NOT NULL,owner_oid TEXT NOT NULL,created_at INTEGER NOT NULL)").execute(db).await?;
    let columns: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(settings)")
            .fetch_all(db)
            .await?;
    if !columns.iter().any(|column| column.1 == "owner_oid") {
        sqlx::query("ALTER TABLE settings ADD COLUMN owner_oid TEXT NOT NULL DEFAULT ''")
            .execute(db)
            .await?;
    }
    sqlx::query("DROP TABLE IF EXISTS auth_sessions")
        .execute(db)
        .await?;
    for statement in [
        "CREATE TABLE IF NOT EXISTS learners(id INTEGER PRIMARY KEY AUTOINCREMENT,alias TEXT NOT NULL COLLATE NOCASE UNIQUE,created_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS circle_sessions(id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT NOT NULL,session_date TEXT NOT NULL,focus TEXT NOT NULL DEFAULT '',created_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS problems(id INTEGER PRIMARY KEY AUTOINCREMENT,session_id INTEGER NOT NULL REFERENCES circle_sessions(id) ON DELETE CASCADE,position INTEGER NOT NULL,title TEXT NOT NULL,prompt TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS attempts(id INTEGER PRIMARY KEY AUTOINCREMENT,learner_id INTEGER NOT NULL REFERENCES learners(id) ON DELETE CASCADE,problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE CASCADE,status TEXT NOT NULL DEFAULT 'not_started',thinking TEXT NOT NULL DEFAULT '',strategies TEXT NOT NULL DEFAULT '[]',private_note TEXT NOT NULL DEFAULT '',updated_at INTEGER NOT NULL,UNIQUE(learner_id,problem_id))",
        "CREATE TABLE IF NOT EXISTS attachments(id INTEGER PRIMARY KEY AUTOINCREMENT,attempt_id INTEGER NOT NULL REFERENCES attempts(id) ON DELETE CASCADE,stored_name TEXT NOT NULL,original_name TEXT NOT NULL,mime TEXT NOT NULL,created_at INTEGER NOT NULL)",
        "CREATE TRIGGER IF NOT EXISTS learners_limit BEFORE INSERT ON learners WHEN (SELECT COUNT(*) FROM learners) >= 12 BEGIN SELECT RAISE(ABORT, 'learner_limit'); END"
    ] {sqlx::query(statement).execute(db).await?;}
    Ok(())
}
async fn schema_is_current(db: &SqlitePool) -> Result<bool, sqlx::Error> {
    let tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('settings','learners','circle_sessions','problems','attempts','attachments')",
    )
    .fetch_one(db)
    .await?;
    if tables != 6 {
        return Ok(false);
    }
    let columns: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(settings)")
            .fetch_all(db)
            .await?;
    let learner_limit: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name='learners_limit'",
    )
    .fetch_one(db)
    .await?;
    Ok(columns.iter().any(|column| column.1 == "owner_oid") && learner_limit == 1)
}
fn sqlite_is_locked(error: &sqlx::Error) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    message.contains("database is locked") || message.contains("database is busy")
}
async fn open_database_with_retry(
    path: PathBuf,
    busy_timeout: Duration,
    retry_delay: Duration,
    attempts: usize,
) -> Result<SqlitePool, sqlx::Error> {
    for attempt in 1..=attempts {
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            // Azure Files is SMB-backed. The dot-file VFS preserves
            // cross-process exclusion without POSIX byte-range locks.
            .vfs("unix-dotfile")
            .busy_timeout(busy_timeout);
        let result = async {
            let db = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await?;
            if !schema_is_current(&db).await? {
                migrate(&db).await?;
            }
            Ok::<_, sqlx::Error>(db)
        }
        .await;
        match result {
            Ok(db) => return Ok(db),
            Err(error) if sqlite_is_locked(&error) && attempt < attempts => {
                tracing::warn!(attempt, "SQLite is busy during rollout; reopening database");
                tokio::time::sleep(retry_delay).await;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!()
}
async fn remove_empty_database_artifacts(path: &std::path::Path) -> std::io::Result<bool> {
    let Ok(metadata) = tokio::fs::metadata(path).await else {
        return Ok(false);
    };
    if metadata.len() != 0 {
        return Ok(false);
    }
    let journal = PathBuf::from(format!("{}-journal", path.display()));
    if tokio::fs::metadata(&journal).await.is_ok() {
        tokio::fs::remove_file(&journal).await?;
    }
    tokio::fs::remove_file(path).await?;
    Ok(true)
}
fn app(state: Shared, dist: PathBuf) -> Router {
    let mut general_builder = GovernorConfigBuilder::default();
    let general_config = Arc::new(
        general_builder
            .per_millisecond(50)
            .burst_size(40)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("valid general API rate limit"),
    );
    let mut write_builder = GovernorConfigBuilder::default();
    let write_config = Arc::new(
        write_builder
            .per_millisecond(250)
            .burst_size(8)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("valid write API rate limit"),
    );
    let read_api = Router::new()
        .route("/status", get(status))
        .route("/board", get(board))
        .route("/files/{id}", get(file))
        .route("/export", get(export_data))
        .route_layer(GovernorLayer::new(general_config).error_handler(rate_limit_response));
    let write_api = Router::new()
        .route("/setup", post(setup))
        .route("/board", delete(delete_board))
        .route("/learners", post(add_learner))
        .route("/learners/{id}", delete(delete_learner))
        .route("/sessions", post(add_session))
        .route("/sessions/{id}", delete(delete_session))
        .route("/problems", post(add_problem))
        .route("/problems/{id}", delete(delete_problem))
        .route("/attempts", post(save_attempt))
        .route("/attempts/{id}/upload", post(upload))
        .route("/files/{id}", delete(delete_file))
        .route_layer(GovernorLayer::new(write_config).error_handler(rate_limit_response));
    let api = read_api.merge(write_api).with_state(state);
    let app_shell = dist.join("app.html");
    Router::new()
        .route("/health", get(health))
        .nest("/api", api)
        .route_service("/privacy", ServeFile::new(app_shell.clone()))
        .route_service("/terms", ServeFile::new(app_shell.clone()))
        .route_service("/demo", ServeFile::new(app_shell.clone()))
        .route_service("/board", ServeFile::new(app_shell.clone()))
        .route_service("/learners", ServeFile::new(app_shell.clone()))
        .route_service("/recap", ServeFile::new(app_shell.clone()))
        .route_service("/plus", ServeFile::new(app_shell.clone()))
        .route_service("/settings", ServeFile::new(app_shell.clone()))
        .route_service("/auth/callback", ServeFile::new(app_shell.clone()))
        .fallback_service(
            ServeDir::new(&dist).not_found_service(ServeFile::new(app_shell)),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(6 * 1024 * 1024))
        .layer(middleware::from_fn(immutable_hashed_assets))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; img-src 'self' data: blob:; connect-src 'self' https://sociobotcustomers.ciamlogin.com; frame-src https://sociobotcustomers.ciamlogin.com; style-src 'self'; script-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(TraceLayer::new_for_http())
}
fn rate_limit_response(error: tower_governor::GovernorError) -> Response {
    let wait = match error {
        tower_governor::GovernorError::TooManyRequests { wait_time, .. } => wait_time.max(1),
        _ => 1,
    };
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({"error":"Too many requests. Wait before trying again."})),
    )
        .into_response();
    response.headers_mut().insert(
        header::RETRY_AFTER,
        HeaderValue::from_str(&wait.to_string()).expect("valid Retry-After"),
    );
    response
}
async fn immutable_hashed_assets(
    request: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let immutable = request.uri().path().starts_with("/assets/");
    let mut response = next.run(request).await;
    if immutable && response.status().is_success() {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }
    response
}
fn owner_invite(data_dir: &std::path::Path) -> std::io::Result<(String, bool)> {
    if let Ok(code) = env::var("MCB_OWNER_INVITE") {
        return Ok((code, true));
    }
    let path = data_dir.join("owner-invite.txt");
    if path.exists() {
        return std::fs::read_to_string(path).map(|code| (code.trim().to_string(), false));
    }
    let code = random_hex(24);
    std::fs::write(&path, &code)?;
    #[cfg(unix)]
    std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    Ok((code, false))
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "math_circle_board=info,tower_http=info".into()),
        )
        .init();
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);
    let data_dir = PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()));
    let dist = PathBuf::from(env::var("DIST_DIR").unwrap_or_else(|_| "./dist".into()));
    tokio::fs::create_dir_all(data_dir.join("uploads"))
        .await
        .expect("create data directory");
    let database_path = data_dir.join("board.db");
    if remove_empty_database_artifacts(&database_path)
        .await
        .expect("clean incomplete empty database")
    {
        tracing::warn!(path=%database_path.display(), "removed incomplete zero-byte SQLite database");
    }
    tracing::info!(path=%database_path.display(), "opening SQLite database");
    let db = open_database_with_retry(
        database_path,
        Duration::from_secs(2),
        Duration::from_secs(1),
        60,
    )
    .await
    .expect("open database");
    let configured: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings WHERE owner_oid<>''")
        .fetch_one(&db)
        .await
        .expect("read configuration");
    let owner_invite_path = data_dir.join("owner-invite.txt");
    let (owner_invite_hash, owner_invite_supplied) = if configured == 0 {
        let (code, supplied) = owner_invite(&data_dir).expect("create owner invite");
        (Some(password_hash(&code, "mcb-owner-invite")), supplied)
    } else {
        (None, false)
    };
    let auth = AuthState::from_env();
    match auth.refresh().await {
        Ok(()) => {
            tracing::info!(authority=%auth.discovery_url,"Microsoft Entra External ID discovery and signing keys loaded")
        }
        Err(error) => {
            tracing::warn!(%error,authority=%auth.discovery_url,"Microsoft Entra External ID metadata will retry on first authenticated request")
        }
    }
    tracing::info!(port,data_dir=%data_dir.display(),owner_invite_supplied,entra_tenant=%auth.tenant_id,entra_client=%auth.client_id,"configuration ready; database path supplied or defaulted, owner invite generated or supplied, Entra defaults supplied or overridden");
    let state = Arc::new(AppState {
        db,
        upload_dir: data_dir.join("uploads"),
        owner_invite_hash: Arc::new(RwLock::new(owner_invite_hash)),
        owner_invite_path,
        auth,
    });
    let listener = TcpListener::bind(("0.0.0.0", port)).await.expect("bind");
    axum::serve(
        listener,
        app(state, dist).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await
    .expect("serve");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn test_app(invite: &str) -> (Router, Shared, PathBuf) {
        let dir = std::env::temp_dir().join(format!("mcb-test-{}", random_hex(6)));
        tokio::fs::create_dir_all(dir.join("uploads"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(dir.join("assets")).await.unwrap();
        tokio::fs::write(dir.join("index.html"), "<!doctype html><title>test</title>")
            .await
            .unwrap();
        tokio::fs::write(
            dir.join("app.html"),
            "<!doctype html><title>test app</title>",
        )
        .await
        .unwrap();
        tokio::fs::write(dir.join("assets/app-abc123.js"), "console.log('cached')")
            .await
            .unwrap();
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let state = Arc::new(AppState {
            db,
            upload_dir: dir.join("uploads"),
            owner_invite_hash: Arc::new(RwLock::new(Some(password_hash(
                invite,
                "mcb-owner-invite",
            )))),
            owner_invite_path: dir.join("owner-invite.txt"),
            auth: AuthState {
                tenant_id: DEFAULT_TENANT_ID.into(),
                client_id: DEFAULT_CLIENT_ID.into(),
                discovery_url: "https://sociobotcustomers.ciamlogin.com/test".into(),
                metadata: Arc::new(RwLock::new(None)),
                keys: Arc::new(RwLock::new(HashMap::new())),
                refreshed_at: Arc::new(RwLock::new(0)),
                client: reqwest::Client::new(),
                test_token: Some("integration-test-entra-token".into()),
            },
        });
        (app(state.clone(), dir.clone()), state, dir)
    }

    fn api_request(method: &str, uri: &str, body: Body) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("x-forwarded-for", "198.51.100.7, 10.0.0.4")
            .header(header::AUTHORIZATION, "Bearer integration-test-entra-token")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap()
    }

    #[tokio::test]
    async fn health_works() {
        let (router, _, _) = test_app("adult-setup-code-0123456789").await;
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::STRICT_TRANSPORT_SECURITY)
                .unwrap(),
            "max-age=63072000; includeSubDomains"
        );
    }
    #[test]
    fn labels_and_hashes_are_stable() {
        assert!(valid_label("Gauss", 60));
        assert!(!valid_label("", 60));
        assert_eq!(
            password_hash("secret", "salt"),
            password_hash("secret", "salt")
        );
    }

    #[tokio::test]
    async fn reads_are_limited_by_first_forwarded_ip_with_retry_after() {
        let (router, _, _) = test_app("adult-setup-code-0123456789").await;
        let mut requests = tokio::task::JoinSet::new();
        for _ in 0..120 {
            let service = router.clone();
            requests.spawn(async move {
                service
                    .oneshot(
                        Request::builder()
                            .uri("/api/status")
                            .header("x-forwarded-for", "203.0.113.9, 10.0.0.8")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap()
            });
        }
        let mut ok = 0;
        let mut limited = 0;
        while let Some(result) = requests.join_next().await {
            let response = result.unwrap();
            match response.status() {
                StatusCode::OK => ok += 1,
                StatusCode::TOO_MANY_REQUESTS => {
                    limited += 1;
                    assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "1");
                }
                status => panic!("unexpected burst status {status}"),
            }
        }
        assert!(ok >= 1);
        assert!(limited >= 1);
        let other_client = router
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .header("x-forwarded-for", "203.0.113.10, 203.0.113.9")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_client.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn writes_have_a_stricter_limit_and_retry_after() {
        let (router, _, _) = test_app("adult-setup-code-0123456789").await;
        let mut limited = None;
        for _ in 0..12 {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/setup")
                        .header("x-forwarded-for", "192.0.2.44, 10.0.0.8")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from("{}"))
                        .unwrap(),
                )
                .await
                .unwrap();
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                limited = Some(response);
                break;
            }
        }
        let limited = limited.expect("write burst should be limited");
        assert_eq!(limited.headers().get(header::RETRY_AFTER).unwrap(), "1");
    }

    #[test]
    fn dates_and_images_are_validated_from_real_values() {
        assert!(valid_iso_date("2028-02-29"));
        assert!(!valid_iso_date("2026-99-99"));
        assert!(!valid_iso_date("2025-02-29"));
        assert_eq!(detected_image_mime(b"not an image"), None);
        let png = BASE64
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
            .unwrap();
        assert_eq!(detected_image_mime(&png), Some("image/png"));
    }

    #[tokio::test]
    async fn entra_ownership_date_upload_and_asset_cache_regressions() {
        let invite = "adult-setup-code-0123456789";
        let (router, state, _) = test_app(invite).await;
        let anonymous = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/setup")
                    .header("x-forwarded-for", "198.51.100.7")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"facilitator":"Morgan","group_name":"Saturday Circle","owner_code":"adult-setup-code-0123456789","adult_confirmed":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            anonymous.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Bearer realm=\"math-circle-board\""
        );
        let wrong = router
            .clone()
            .oneshot(api_request(
                "POST",
                "/api/setup",
                Body::from(r#"{"facilitator":"Morgan","group_name":"Saturday Circle","owner_code":"not-the-code","adult_confirmed":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::FORBIDDEN);

        let setup = router
            .clone()
            .oneshot(api_request(
                "POST",
                "/api/setup",
                Body::from(format!(r#"{{"facilitator":"Morgan","group_name":"Saturday Circle","owner_code":"{invite}","adult_confirmed":true}}"#)),
            ))
            .await
            .unwrap();
        assert_eq!(setup.status(), StatusCode::OK);
        assert!(setup.headers().get(header::SET_COOKIE).is_none());

        let invalid_date = router
            .clone()
            .oneshot(api_request(
                "POST",
                "/api/sessions",
                Body::from(
                    r#"{"title":"Impossible date","session_date":"2026-99-99","focus":"Boundary"}"#,
                ),
            ))
            .await
            .unwrap();
        assert_eq!(invalid_date.status(), StatusCode::BAD_REQUEST);

        let forged_upload = concat!(
            "--mcb\r\n",
            "Content-Disposition: form-data; name=\"image\"; filename=\"hostname.png\"\r\n",
            "Content-Type: image/png\r\n\r\n",
            "not a PNG image\r\n",
            "--mcb--\r\n"
        );
        let forged = router
            .clone()
            .oneshot({
                let mut request =
                    api_request("POST", "/api/attempts/1/upload", Body::from(forged_upload));
                request.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("multipart/form-data; boundary=mcb"),
                );
                request
            })
            .await
            .unwrap();
        assert_eq!(forged.status(), StatusCode::BAD_REQUEST);
        let attachments: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attachments")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(attachments, 0);

        let asset = router
            .oneshot(
                Request::builder()
                    .uri("/assets/app-abc123.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(asset.status(), StatusCode::OK);
        assert_eq!(
            asset.headers().get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=31536000, immutable"
        );
    }

    #[tokio::test]
    async fn legal_routes_are_200_and_unknown_routes_remain_404() {
        let (router, _, _) = test_app("adult-setup-code-0123456789").await;
        for route in ["/privacy", "/terms", "/demo", "/board", "/learners"] {
            let response = router
                .clone()
                .oneshot(Request::builder().uri(route).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "route {route}");
        }
        let missing = router
            .oneshot(
                Request::builder()
                    .uri("/missing-page")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn owner_can_delete_the_entire_board_and_new_invite_is_generated() {
        let invite = "adult-setup-code-0123456789";
        let (router, state, dir) = test_app(invite).await;
        let setup = router
            .clone()
            .oneshot(api_request(
                "POST",
                "/api/setup",
                Body::from(format!(r#"{{"facilitator":"Morgan","group_name":"Saturday Circle","owner_code":"{invite}","adult_confirmed":true}}"#)),
            ))
            .await
            .unwrap();
        assert_eq!(setup.status(), StatusCode::OK);
        let learner = router
            .clone()
            .oneshot(api_request(
                "POST",
                "/api/learners",
                Body::from(r#"{"alias":"Ada"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(learner.status(), StatusCode::OK);
        let deleted = router
            .oneshot(api_request("DELETE", "/api/board", Body::empty()))
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::OK);
        let settings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings")
            .fetch_one(&state.db)
            .await
            .unwrap();
        let learners: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM learners")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!((settings, learners), (0, 0));
        assert!(state.owner_invite_hash.read().await.is_some());
        assert!(dir.join("owner-invite.txt").exists());
        #[cfg(unix)]
        let permissions = std::fs::metadata(dir.join("owner-invite.txt"))
            .unwrap()
            .permissions();
        assert_eq!(
            std::os::unix::fs::PermissionsExt::mode(&permissions) & 0o777,
            0o600
        );
    }

    #[tokio::test]
    async fn learner_roster_stops_at_twelve() {
        let invite = "adult-setup-code-0123456789";
        let (router, state, _) = test_app(invite).await;
        let setup = router
            .clone()
            .oneshot(api_request(
                "POST",
                "/api/setup",
                Body::from(format!(r#"{{"facilitator":"Morgan","group_name":"Saturday Circle","owner_code":"{invite}","adult_confirmed":true}}"#)),
            ))
            .await
            .unwrap();
        assert_eq!(setup.status(), StatusCode::OK);
        for number in 1..=12 {
            sqlx::query("INSERT INTO learners(alias,created_at) VALUES(?,?)")
                .bind(format!("Learner {number}"))
                .bind(now())
                .execute(&state.db)
                .await
                .unwrap();
        }
        let thirteenth = router
            .oneshot(api_request(
                "POST",
                "/api/learners",
                Body::from(r#"{"alias":"Learner 13"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(thirteenth.status(), StatusCode::CONFLICT);
        let learners: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM learners")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(learners, 12);
    }

    #[tokio::test]
    async fn startup_migration_retries_a_locked_sqlite_database() {
        let dir = std::env::temp_dir().join(format!("mcb-lock-test-{}", random_hex(6)));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let path = dir.join("board.db");
        let first = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(&path)
                    .create_if_missing(true)
                    .vfs("unix-dotfile")
                    .busy_timeout(Duration::from_millis(50)),
            )
            .await
            .unwrap();
        sqlx::query("CREATE TABLE lock_holder(id INTEGER PRIMARY KEY)")
            .execute(&first)
            .await
            .unwrap();
        let mut lock = first.begin().await.unwrap();
        sqlx::query("INSERT INTO lock_holder(id) VALUES(1)")
            .execute(&mut *lock)
            .await
            .unwrap();
        let retry_path = path.clone();
        let retry = tokio::spawn(async move {
            open_database_with_retry(
                retry_path,
                Duration::from_millis(50),
                Duration::from_millis(50),
                20,
            )
            .await
        });
        tokio::time::sleep(Duration::from_millis(200)).await;
        lock.rollback().await.unwrap();
        first.close().await;
        tokio::time::timeout(Duration::from_secs(5), retry)
            .await
            .expect("migration retry should finish")
            .unwrap()
            .expect("migration should succeed after lock release")
            .close()
            .await;
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn legacy_schema_migrates_with_a_single_connection() {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE settings(id INTEGER PRIMARY KEY CHECK(id=1),facilitator TEXT NOT NULL,group_name TEXT NOT NULL,created_at INTEGER NOT NULL)")
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO settings VALUES(1,'Sam','Saturday circle',123)")
            .execute(&db)
            .await
            .unwrap();

        migrate(&db).await.unwrap();

        let saved: (String, String, String, i64) = sqlx::query_as(
            "SELECT facilitator,group_name,owner_oid,created_at FROM settings WHERE id=1",
        )
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(
            saved,
            ("Sam".into(), "Saturday circle".into(), "".into(), 123)
        );
        assert!(schema_is_current(&db).await.unwrap());
    }

    #[tokio::test]
    async fn only_zero_byte_database_artifacts_are_removed() {
        let dir = std::env::temp_dir().join(format!("mcb-empty-test-{}", random_hex(6)));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let path = dir.join("board.db");
        let journal = dir.join("board.db-journal");
        tokio::fs::write(&path, []).await.unwrap();
        tokio::fs::write(&journal, [1, 2, 3]).await.unwrap();
        assert!(remove_empty_database_artifacts(&path).await.unwrap());
        assert!(!path.exists());
        assert!(!journal.exists());

        tokio::fs::write(&path, b"database data").await.unwrap();
        tokio::fs::write(&journal, [1, 2, 3]).await.unwrap();
        assert!(!remove_empty_database_artifacts(&path).await.unwrap());
        assert!(path.exists());
        assert!(journal.exists());
        let _ = tokio::fs::remove_dir_all(dir).await;
    }
}
