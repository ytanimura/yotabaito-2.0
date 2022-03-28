#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;

mod background;
mod contents;

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { <><contents::Contents /><background::BackGround /></> }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
