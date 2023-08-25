mod components;

use components::{NameForm, CounterButton, NameHeader};
use redis::Commands;
use leptos::*;
use uuid::Uuid;
use actix_web::{ web, get, post, App, HttpRequest, HttpResponse, HttpServer, middleware, cookie::Cookie};
use serde::{Deserialize, Serialize};



struct AppState {
    redis_client: redis::Client,
    redis_db: redis::Client,
}

#[derive(Serialize, Deserialize)]
struct MyState {
    count: usize,
    name: String,
}

fn get_session_id(req: HttpRequest) -> String {
    let cookie = req.cookie("session").unwrap_or_else(|| Cookie::new("session", Uuid::new_v4().to_string()));
    let key = cookie.value();
    return key.to_string();
}

#[get("/")]
async fn index(req: HttpRequest, data: web::Data<AppState>) ->  HttpResponse {
    let session_id = get_session_id(req);
    let mut con: redis::Connection = data.redis_client.get_connection().expect("Failed to get Redis connection");
    let mut db: redis::Connection = data.redis_db.get_connection().expect("Failed to get Redis connection");
    let user_uuid: std::result::Result<String, redis::RedisError> = con.get(&session_id);
    let html;
    match user_uuid {
        Ok(value) => {
            let string_state: String = db.get(&value).unwrap();
            let state: MyState = serde_urlencoded::from_str(&string_state).unwrap();
            html = leptos::ssr::render_to_string(move |cx| view! { cx,
                <head>
                    <script src="https://unpkg.com/htmx.org@1.9.4"></script>
                </head>
                <body>
                    <NameHeader name=state.name.to_string()/>
                    <CounterButton count=state.count/>
                </body>
            });
        },
        _ => {
            html = leptos::ssr::render_to_string(move |cx| view! { cx,
                <head>
                    <script src="https://unpkg.com/htmx.org@1.9.4"></script>
                </head>
                <body>
                    <NameForm />
                </body>
            });
        }
    };
    return HttpResponse::Ok().cookie(
        Cookie::build("session", &*session_id)
                .path("/")
                .secure(false)
                .finish(),
    ).content_type("text/html; charset=utf-8").body(html);
}

#[post("/name")]
async fn new_user(req: HttpRequest, params: web::Form<MyState>, data: web::Data<AppState>) -> HttpResponse{
    let session_id = get_session_id(req);
    let use_id = Uuid::new_v4().to_string();

    let mut con = data.redis_client.get_connection().expect("Failed to get Redis connection");
    con.set::<&str, &String, ()>(&session_id, &use_id).unwrap();

    let mut db = data.redis_db.get_connection().expect("Failed to get Redis connection");
    let string_state = serde_urlencoded::to_string(&params).unwrap();
    db.set::<&str, &String, ()>(&use_id, &string_state).unwrap();

    let html: String = leptos::ssr::render_to_string(move |cx| view! { cx,
        <NameHeader name=params.name.to_string()/>
        <CounterButton count=params.count/>
    });
    
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);
}

#[post("/clicked")] 
async fn clicked(req: HttpRequest, data: web::Data<AppState>) ->  HttpResponse {
    let session_id = get_session_id(req);
    let mut con = data.redis_client.get_connection().expect("Failed to get Redis connection");
    let use_id: String = con.get(&session_id).unwrap();

    let mut db = data.redis_db.get_connection().expect("Failed to get Redis connection");
    let string_state: String = db.get(&use_id).unwrap();

    let mut state: MyState = serde_urlencoded::from_str(&string_state).unwrap();
    let count = state.count + 1;
    state.count = count;
    let string_state = serde_urlencoded::to_string(state).unwrap();

    let _: () = db.set(use_id, string_state).unwrap();
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(format!("{}", count));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1/0").expect("Failed to connect to Redis");
    let redis_db = redis::Client::open("redis://127.0.0.1/1").expect("Failed to connect to Redis");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(AppState { redis_client: redis_client.clone(), redis_db: redis_db.clone() }))
            .service(index)
            .service(clicked)
            .service(new_user)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}