#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;
use yew::prelude::*;

mod background;
mod contents;
mod navbar;

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let query = Query::from_location();
        if query.doc.as_deref() == Some("none") {
            let shader_name = query.shader.unwrap_or_else(|| String::from("default"));
            html! { <background::BackGround shader_name={ shader_name } /> }
        } else {
            let doc_name = query.doc.clone();
            let shader_name = query
                .shader
                .or(query.doc)
                .unwrap_or_else(|| String::from("default"));
            html! {
                <>
                <navbar::NavBar />
                <contents::Contents doc_name={ doc_name } />
                <iframe class="background" src={ format!("./index.html?doc=none&shader={shader_name}") } />
                </>
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Query {
    pub doc: Option<String>,
    pub shader: Option<String>,
}

impl Query {
    pub fn from_location() -> Query {
        use qstring::QString;
        let location = gloo::utils::window().location();
        let raw_query = location.search().expect_throw("failed to get query");
        let qstr = QString::from(raw_query.as_str());
        Query {
            doc: qstr.get("doc").map(String::from),
            shader: qstr.get("shader").map(String::from),
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    if let Some(doc) = Query::from_location().doc {
        gloo::utils::document().set_title(&format!("yotabaito: {doc}"));
    }
    yew::start_app::<App>();
}
