use crate::*;

pub struct Contents;

fn markdown2html(markdown: &str) -> String {
    use pulldown_cmark::{html, *};
    let parser = Parser::new(markdown);
    let mut res = String::new();
    html::push_html(&mut res, parser);
    res
}

fn html2div(html: &str) -> Html {
    let div = (|| -> Option<web_sys::Element> {
        web_sys::window()?.document()?.create_element("div").ok()
    })()
    .unwrap();
    div.set_inner_html(&html);
    Html::VRef(div.into())
}

impl Component for Contents {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let content = markdown2html(include_str!("../texts/profile.md"));
        html! {
            <div class="contents">{ html2div(&content) }</div>
        }
    }
}
