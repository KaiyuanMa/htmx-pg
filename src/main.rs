mod templates;
use actix_web::{ get, post, delete, App, HttpRequest, HttpResponse, HttpServer, middleware, web, cookie::SameSite, patch};
use askama::Template;
use log::info;
use templates::Todo;
use actix_files as fs;
use actix_session::{storage::CookieSessionStore, SessionMiddleware, Session};
use actix_web::cookie::Key;
use templates::{Index, TodoItem, TodoList, EditItem, ItemCount};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn get_items_left (todos: &Vec<Todo>) -> i32 {
    let mut count = 0;
    for todo in todos {
        if todo.done == false {
            count += 1;
        }
    }
    return count;
}
#[derive(Deserialize)]
struct IndexQuery {
    filter: String
}

#[get("/")]
async fn index(req: HttpRequest, session: Session, params: web::Query<IndexQuery>) ->  HttpResponse {
    let todos = session.get::<Vec<Todo>>("todo").unwrap().unwrap_or(vec![]);
    let filtered_todos:Vec<Todo>;
    let filter = params.into_inner().filter;
    match filter.as_ref() {
        "all" => filtered_todos = todos,
        "active" => filtered_todos = todos.into_iter().filter(|todo| todo.done == false).collect(),
        "completed" => filtered_todos = todos.into_iter().filter(|todo| todo.done == true).collect(),
        _ => filtered_todos = todos
    }
    let count = filtered_todos.len() as i32;
    let todo_items = TodoList {todos: filtered_todos};
    let item_count = ItemCount {items_left:count};
    let html = Index {todos: todo_items, item_count, filter}.render().unwrap();
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoParams {
    todo: String
}

#[post("/todos")]
async fn add_todo(_req: HttpRequest, params: web::Form<TodoParams>, session: Session) -> HttpResponse {
    let mut todos = session.get::<Vec<Todo>>("todo").unwrap().unwrap_or(vec![]);
    let name = params.into_inner().todo;
    let new_todo = Todo {id: Uuid::new_v4(), name, done: false};
    todos.push(new_todo.clone());
    let todo = TodoItem {todo:new_todo}.render().unwrap();

    let count = get_items_left(&todos);
    let item_count = ItemCount {items_left:count}.render().unwrap();
    let html = format!("{}{}", todo, item_count);
    session.insert("todo", todos).unwrap();
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);
}

#[get("/todos/edit/{id}")]
async fn edit_item(req: HttpRequest, session: Session, path: web::Path<uuid::Uuid>,) -> HttpResponse {
    let id = path.into_inner();
    let todos = session.get::<Vec<Todo>>("todo").unwrap().unwrap_or(vec![]);
    let todo = todos.into_iter().find(|todo| todo.id == id).unwrap();
    let html = EditItem {todo}.render().unwrap();
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);
}


#[patch("/todos/{id}")]
async fn update_item_html(_req: HttpRequest, session: Session, path: web::Path<uuid::Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let mut todos = session.get::<Vec<Todo>>("todo").unwrap().unwrap_or(vec![]);
    let count = get_items_left(&todos) - 1;
    let mut todo = todos.iter().position(|todo| todo.id == id).unwrap();
    todos[todo].done = !todos[todo].done;
    let todo = TodoItem { todo:todos[todo].clone()}.render().unwrap();
    let item_count = ItemCount {items_left:count}.render().unwrap();
    let html = format!("{}{}", todo, item_count);
    session.insert("todo", todos).unwrap();
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);
}


#[post("/todos/update/{id}")]
async fn update_item(_req: HttpRequest, session: Session, path: web::Path<uuid::Uuid>, params: web::Form<TodoParams>) -> HttpResponse {
    let id = path.into_inner();
    let name = params.into_inner().todo;
    let todos = session.get::<Vec<Todo>>("todo").unwrap().unwrap_or(vec![]);
    let count = get_items_left(&todos);
    let mut todo = todos.into_iter().find(|todo| todo.id == id).unwrap();
    todo.name = name;
    let todo = TodoItem {todo}.render().unwrap();

    let item_count = ItemCount {items_left:count}.render().unwrap();
    let html = format!("{}{}", todo, item_count);
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);
}

#[delete("/todos/{id}")]
async fn delete_item(_req: HttpRequest, session: Session, path: web::Path<uuid::Uuid>) -> HttpResponse {
    let id: Uuid = path.into_inner();
    let todos = session.get::<Vec<Todo>>("todo").unwrap().unwrap_or(vec![]);
    let todos = todos.into_iter().filter(|todo| todo.id != id).collect::<Vec<Todo>>();
    let count = get_items_left(&todos);
    let item_count: String = ItemCount {items_left:count}.render().unwrap();
    session.insert("todo", todos).unwrap();
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(item_count);
}

#[post("/todos/clear-completed")]
async fn clear_completed(_req: HttpRequest, session: Session) -> HttpResponse {
    let todos = session.get::<Vec<Todo>>("todo").unwrap().unwrap_or(vec![]);
    let todos = todos.into_iter().filter(|todo| todo.done == false).collect::<Vec<Todo>>();
    let count = get_items_left(&todos);
    let item_count: String = ItemCount {items_left:count}.render().unwrap();
    let todo_list = TodoList {todos: todos.clone()};
    session.insert("todo", todos).unwrap();
    let html = format!("{}{}", todo_list, item_count);
    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("server at http://localhost:8080/?filter=all");

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_same_site(SameSite::None)
                    .cookie_secure(true)
                    .build(),
            )
            .service(fs::Files::new("/static", "./static"))
            .wrap(middleware::Logger::default())
            .service(index)
            .service(add_todo)
            .service(update_item_html)
            .service(delete_item)
            .service(edit_item)
            .service(update_item)
            .service(clear_completed)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}


