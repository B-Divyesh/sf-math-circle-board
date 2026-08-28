use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::{
    env,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpListener;
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
    owner_invite_hash: Option<String>,
    owner_invite_path: Option<PathBuf>,
}
type Shared = Arc<AppState>;

#[derive(Debug)]
struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error": self.1}))).into_response()
    }
}
type ApiResult<T> = Result<T, ApiError>;

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
fn cookie(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|v| {
            let (k, val) = v.trim().split_once('=')?;
            (k == key).then(|| val.to_string())
        })
}
async fn require_auth(state: &Shared, headers: &HeaderMap) -> ApiResult<()> {
    let token = cookie(headers, "mcb_session").ok_or(ApiError(
        StatusCode::UNAUTHORIZED,
        "Sign in to continue.".into(),
    ))?;
    let digest = hex::encode(Sha256::digest(token.as_bytes()));
    let found: Option<i64> = sqlx::query_scalar(
        "SELECT expires_at FROM auth_sessions WHERE token_hash=? AND expires_at>?",
    )
    .bind(digest)
    .bind(now())
    .fetch_optional(&state.db)
    .await
    .map_err(db_err)?;
    if found.is_none() {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Your session expired. Sign in again.".into(),
        ));
    }
    Ok(())
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
    passphrase: String,
    group_name: String,
    owner_code: String,
    adult_confirmed: bool,
}
#[derive(Deserialize)]
struct LoginInput {
    passphrase: String,
}

async fn status(State(s): State<Shared>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    let configured: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings")
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    let facilitator: Option<String> =
        sqlx::query_scalar("SELECT facilitator FROM settings LIMIT 1")
            .fetch_optional(&s.db)
            .await
            .map_err(db_err)?;
    let authenticated = require_auth(&s, &headers).await.is_ok();
    Ok(Json(
        json!({"configured":configured>0,"authenticated":authenticated,"facilitator":facilitator}),
    ))
}
async fn setup(
    State(s): State<Shared>,
    Json(input): Json<SetupInput>,
) -> ApiResult<impl IntoResponse> {
    if !valid_label(&input.facilitator, 80)
        || !valid_label(&input.group_name, 100)
        || input.passphrase.chars().count() < 8
        || !input.adult_confirmed
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Use a facilitator name, group name, an adult confirmation, and a passphrase of at least 8 characters.".into(),
        ));
    }
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings")
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    if exists > 0 {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "This board is already owned.".into(),
        ));
    }
    let Some(owner_invite_hash) = &s.owner_invite_hash else {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "This deployment needs an installer-issued adult setup code before it can be claimed."
                .into(),
        ));
    };
    if password_hash(&input.owner_code, "mcb-owner-invite") != *owner_invite_hash {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "That adult setup code did not match this deployment. Ask the deployment operator for the code.".into(),
        ));
    }
    let salt = random_hex(16);
    let hash = password_hash(&input.passphrase, &salt);
    let mut tx = s.db.begin().await.map_err(db_err)?;
    sqlx::query("INSERT INTO settings(id,facilitator,group_name,pass_salt,pass_hash,created_at) VALUES(1,?,?,?,?,?)")
        .bind(input.facilitator.trim()).bind(input.group_name.trim()).bind(salt).bind(hash).bind(now()).execute(&mut *tx).await.map_err(db_err)?;
    tx.commit().await.map_err(db_err)?;
    if let Some(path) = &s.owner_invite_path {
        let _ = tokio::fs::remove_file(path).await;
    }
    create_session(&s.db).await
}
async fn login(
    State(s): State<Shared>,
    Json(input): Json<LoginInput>,
) -> ApiResult<impl IntoResponse> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT pass_salt,pass_hash FROM settings LIMIT 1")
            .fetch_optional(&s.db)
            .await
            .map_err(db_err)?;
    let Some((salt, expected)) = row else {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Set up this board first.".into(),
        ));
    };
    if password_hash(&input.passphrase, &salt) != expected {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "That passphrase did not match.".into(),
        ));
    }
    create_session(&s.db).await
}
async fn create_session(db: &SqlitePool) -> ApiResult<impl IntoResponse> {
    let token = random_hex(32);
    let digest = hex::encode(Sha256::digest(token.as_bytes()));
    sqlx::query("DELETE FROM auth_sessions WHERE expires_at<=?")
        .bind(now())
        .execute(db)
        .await
        .map_err(db_err)?;
    sqlx::query("INSERT INTO auth_sessions(token_hash,expires_at) VALUES(?,?)")
        .bind(digest)
        .bind(now() + 60 * 60 * 24 * 30)
        .execute(db)
        .await
        .map_err(db_err)?;
    let mut h = HeaderMap::new();
    h.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "mcb_session={token}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=2592000"
        ))
        .unwrap(),
    );
    Ok((h, Json(json!({"ok":true}))))
}
async fn logout(State(s): State<Shared>, headers: HeaderMap) -> ApiResult<impl IntoResponse> {
    if let Some(token) = cookie(&headers, "mcb_session") {
        let digest = hex::encode(Sha256::digest(token.as_bytes()));
        let _ = sqlx::query("DELETE FROM auth_sessions WHERE token_hash=?")
            .bind(digest)
            .execute(&s.db)
            .await;
    }
    let mut h = HeaderMap::new();
    h.insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "mcb_session=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0",
        ),
    );
    Ok((h, Json(json!({"ok":true}))))
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
    for statement in [
        "CREATE TABLE IF NOT EXISTS settings(id INTEGER PRIMARY KEY CHECK(id=1),facilitator TEXT NOT NULL,group_name TEXT NOT NULL,pass_salt TEXT NOT NULL,pass_hash TEXT NOT NULL,created_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS auth_sessions(token_hash TEXT PRIMARY KEY,expires_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS learners(id INTEGER PRIMARY KEY AUTOINCREMENT,alias TEXT NOT NULL COLLATE NOCASE UNIQUE,created_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS circle_sessions(id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT NOT NULL,session_date TEXT NOT NULL,focus TEXT NOT NULL DEFAULT '',created_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS problems(id INTEGER PRIMARY KEY AUTOINCREMENT,session_id INTEGER NOT NULL REFERENCES circle_sessions(id) ON DELETE CASCADE,position INTEGER NOT NULL,title TEXT NOT NULL,prompt TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS attempts(id INTEGER PRIMARY KEY AUTOINCREMENT,learner_id INTEGER NOT NULL REFERENCES learners(id) ON DELETE CASCADE,problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE CASCADE,status TEXT NOT NULL DEFAULT 'not_started',thinking TEXT NOT NULL DEFAULT '',strategies TEXT NOT NULL DEFAULT '[]',private_note TEXT NOT NULL DEFAULT '',updated_at INTEGER NOT NULL,UNIQUE(learner_id,problem_id))",
        "CREATE TABLE IF NOT EXISTS attachments(id INTEGER PRIMARY KEY AUTOINCREMENT,attempt_id INTEGER NOT NULL REFERENCES attempts(id) ON DELETE CASCADE,stored_name TEXT NOT NULL,original_name TEXT NOT NULL,mime TEXT NOT NULL,created_at INTEGER NOT NULL)"
    ] {sqlx::query(statement).execute(db).await?;}
    Ok(())
}
fn app(state: Shared, dist: PathBuf) -> Router {
    let api = Router::new()
        .route("/status", get(status))
        .route("/setup", post(setup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/board", get(board))
        .route("/learners", post(add_learner))
        .route("/learners/{id}", delete(delete_learner))
        .route("/sessions", post(add_session))
        .route("/sessions/{id}", delete(delete_session))
        .route("/problems", post(add_problem))
        .route("/problems/{id}", delete(delete_problem))
        .route("/attempts", post(save_attempt))
        .route("/attempts/{id}/upload", post(upload))
        .route("/files/{id}", get(file).delete(delete_file))
        .route("/export", get(export_data))
        .with_state(state);
    Router::new()
        .route("/health", get(health))
        .nest("/api", api)
        .fallback_service(
            ServeDir::new(&dist).not_found_service(ServeFile::new(dist.join("index.html"))),
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
            HeaderValue::from_static("default-src 'self'; img-src 'self' data:; connect-src 'self' https://api.sociobot.in; style-src 'self'; script-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self' https://api.sociobot.in"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(TraceLayer::new_for_http())
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
    let url = format!("sqlite://{}?mode=rwc", data_dir.join("board.db").display());
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("open database");
    migrate(&db).await.expect("migrate database");
    let configured: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings")
        .fetch_one(&db)
        .await
        .expect("read configuration");
    let (owner_invite_hash, owner_invite_path, owner_invite_supplied) = if configured == 0 {
        let (code, supplied) = owner_invite(&data_dir).expect("create owner invite");
        (
            Some(password_hash(&code, "mcb-owner-invite")),
            (!supplied).then(|| data_dir.join("owner-invite.txt")),
            supplied,
        )
    } else {
        (None, None, false)
    };
    tracing::info!(port,data_dir=%data_dir.display(),owner_invite_supplied,"configuration ready; database path supplied or defaulted, owner invite generated or supplied, session tokens generated per sign-in");
    let state = Arc::new(AppState {
        db,
        upload_dir: data_dir.join("uploads"),
        owner_invite_hash,
        owner_invite_path,
    });
    let listener = TcpListener::bind(("0.0.0.0", port)).await.expect("bind");
    axum::serve(listener, app(state, dist))
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
            owner_invite_hash: Some(password_hash(invite, "mcb-owner-invite")),
            owner_invite_path: None,
        });
        (app(state.clone(), dir.clone()), state, dir)
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
    async fn ownership_cookie_date_upload_and_asset_cache_regressions() {
        let invite = "adult-setup-code-0123456789";
        let (router, state, _) = test_app(invite).await;
        let wrong = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/setup")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"facilitator":"Morgan","group_name":"Saturday Circle","passphrase":"lantern-path-2026","owner_code":"not-the-code","adult_confirmed":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::FORBIDDEN);

        let setup = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/setup")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(r#"{{"facilitator":"Morgan","group_name":"Saturday Circle","passphrase":"lantern-path-2026","owner_code":"{invite}","adult_confirmed":true}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(setup.status(), StatusCode::OK);
        let set_cookie = setup
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(set_cookie.contains("HttpOnly; Secure; SameSite=Strict"));
        let session_cookie = set_cookie.split(';').next().unwrap();

        let invalid_date = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/sessions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, session_cookie)
                    .body(Body::from(r#"{"title":"Impossible date","session_date":"2026-99-99","focus":"Boundary"}"#))
                    .unwrap(),
            )
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
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/attempts/1/upload")
                    .header(header::CONTENT_TYPE, "multipart/form-data; boundary=mcb")
                    .header(header::COOKIE, session_cookie)
                    .body(Body::from(forged_upload))
                    .unwrap(),
            )
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
}
