#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;
use yew::prelude::*;

mod app;
mod background;
mod contents;
mod navbar;

fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<app::App>();
}
