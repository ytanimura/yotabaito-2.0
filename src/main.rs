#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use yew::prelude::*;

struct App;

fn markdown2html(markdown: &str) -> String {
    use pulldown_cmark::{html, *};
    let parser = Parser::new(markdown);
    let mut res = String::new();
    html::push_html(&mut res, parser);
    res
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let content = markdown2html(include_str!("../texts/profile.md"));
        let div = (|| -> Option<web_sys::Element> {
            web_sys::window()?.document()?.create_element("div").ok()
        })()
        .unwrap();
        div.set_inner_html(&content);
        html! {
            <div class="contents">{ Html::VRef(div.into()) }</div>
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
