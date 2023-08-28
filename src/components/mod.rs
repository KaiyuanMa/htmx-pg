use askama::{Template, DynTemplate};


#[derive(Template)]
#[template(path = "index.html")] 
pub struct IndexTemplate {
    pub content:Box<dyn DynTemplate>,
}

#[derive(Template)]
#[template(path = "count_button.html")] 
pub struct CountButtonTemplate { 
    pub count: usize, 
}

#[derive(Template)]
#[template(path = "name_form.html")]
pub struct NameFromTemplate {}

#[derive(Template)]
#[template(path = "name_header.html")]
pub struct NameHeaderTemplate {
    pub name: String,
}