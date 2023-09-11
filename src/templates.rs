use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Todo {
    pub id: Uuid,
    pub name: String,
    pub done: bool
}

#[derive(Template)]
#[template(path = "index.html")] 
pub struct Index {
    pub item_count: ItemCount,
    pub todos: TodoList,
    pub filter: String
}

#[derive(Template)]
#[template(path = "todo-list.html")] 
pub struct TodoList {
    pub todos: Vec<Todo>
}

#[derive(Template)]
#[template(path = "todo-item.html")] 
pub struct TodoItem {
    pub todo: Todo
}

#[derive(Template)]
#[template(path = "edit-item.html")] 
pub struct EditItem {
    pub todo: Todo
}

#[derive(Template)]
#[template(path = "item-count.html")] 
pub struct ItemCount {
    pub items_left: i32
}