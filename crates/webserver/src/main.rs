use askama::Template;
use axum::{
    extract::{Form, Query},
    response::{Html, IntoResponse, Response, Redirect},
    routing::get,
    Router
};
use axum_sessions::{
    async_session::MemoryStore,
    extractors::{ReadableSession, WritableSession},
    SessionLayer
};
use lazy_static::lazy_static;
use serde::Deserialize;
use rand::Rng;
use std::{
    cmp::min,
    fs,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4}
};
use toml;
use tower_http::services::ServeDir;

const SETTINGS_PATH: &str = "settings.toml";

lazy_static! {
    static ref SETTINGS: Settings = {
        let s = fs::read_to_string(SETTINGS_PATH).expect("Error reading the settings file!");
        toml::from_str::<Settings>(&s).expect("Settings cannot be parsed!")
    };
}


#[tokio::main]
async fn main() {
    let store = MemoryStore::new();
    let secret = rand::thread_rng().gen::<[u8; 128]>();
    let session_layer = SessionLayer::new(store, &secret);

    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login_get).post(login_post))
        .route("/logout", get(logout))
        // TODO - hide media behind session as well?
        .nest_service("/media", ServeDir::new(&SETTINGS.media_dir))
        .layer(session_layer);

    let addr = SocketAddr::V4(
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), SETTINGS.port)
    );
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(session: ReadableSession, params: Query<IndexParams>) -> Response {
    if !session.get("auth").unwrap_or(false) {
        return Redirect::to("/login").into_response();
    }

    let mut paths = fs::read_dir(&SETTINGS.media_dir)
        .expect("Can't read media dir!")
        .filter_map(|f| f.ok())
        .filter_map(|f| f.file_name().into_string().ok())
        .collect::<Vec<_>>();

    paths.sort();

    let offset = params.offset.unwrap_or(0);
    let end = min(offset + SETTINGS.img_per_page, paths.len());

    let prev = match offset.saturating_sub(SETTINGS.img_per_page) {
        a if a != offset => Some(a),
        _ => None
    };
    let next = if paths.len().saturating_sub(offset) > SETTINGS.img_per_page {
        Some(offset + SETTINGS.img_per_page)
    } else {
        None
    };

    let template = IndexTemplate { 
        paths: &paths[offset..end],
        prev,
        next
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(e.to_string()).into_response()
    }
}

async fn login_get() -> Html<String>  {
    let template = LoginTemplate;
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(e.to_string()) 
    }
}

async fn login_post(
    mut session: WritableSession,
    Form(input): Form<LoginInput>
) -> Response {
    if input.password != SETTINGS.password {
        return Html("<H1>Wrong password!</H1>").into_response();
    }
    session.insert("auth", true)
        .expect("Session write error!");
    Redirect::to("/").into_response()
}

async fn logout(mut session: WritableSession) -> Response {
    session.destroy();
    Redirect::to("/login").into_response()
}

#[derive(Deserialize)]
struct IndexParams {
    offset: Option<usize>
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    paths: &'a [String],
    prev: Option<usize>,
    next: Option<usize>
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Deserialize)]
struct LoginInput {
    pub password: String
}

#[derive(Deserialize)]
struct Settings {
    pub port: u16,
    pub media_dir: String,
    pub img_per_page: usize,
    pub password: String
}