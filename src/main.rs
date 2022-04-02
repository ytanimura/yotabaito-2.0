#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;

mod background;
mod contents;
mod query;
use query::Query;

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let query = Query::from_location();
        let doc_name = query.doc.clone();
        let shader_name = query
            .shader
            .or(query.doc)
            .unwrap_or_else(|| String::from("default"));
        html! {
            <>
            <contents::Contents doc_name={ doc_name } />
            <background::BackGround shader_name={ shader_name } />
            </>
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
