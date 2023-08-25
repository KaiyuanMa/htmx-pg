use leptos::*;

#[component]
pub fn NameForm(cx: Scope) -> impl IntoView {
    return view! {cx,
        <form id="main"
            hx-post="/name"
            hx-target="#main"
            hx-trigger="submit"
        >
            <input name="name" type="text" placeholder="name here"/>
            <input name="count"/>
            <button type="submit">
                "Submit"
            </button>
        </form>
    }
}

#[component]
pub fn CounterButton(cx: Scope, count: usize) -> impl IntoView {
    return view! {cx, 
        <div id="counting"> {format!("{}", count)} </div>
        <button
            hx-post="/clicked"
            hx-target="#counting"
        >
            "Click Me!"
        </button>
    }
}

#[component]
pub fn NameHeader(cx: Scope, name: String) -> impl IntoView {
    return view! {cx,
        <h1> {format!("Hello, {}!", name)} </h1>
    }
}