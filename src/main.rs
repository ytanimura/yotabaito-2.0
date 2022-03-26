#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use yew::prelude::*;

mod contents;
mod background;

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
