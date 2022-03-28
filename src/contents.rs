use crate::*;
use std::collections::HashMap;

pub struct Contents {
    contents: HashMap<&'static str, &'static str>,
}

mod texts {
    include!(concat!(env!("OUT_DIR"), "/texts.rs"));
}

fn html2div(html: &str) -> Html {
    let div = gloo::utils::document()
        .create_element("div")
        .expect_throw("failed to create div");
    div.set_inner_html(&html);
    Html::VRef(div.into())
}

impl Component for Contents {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            contents: texts::get_markdowns(),
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let location = gloo::utils::window().location();
        let hash = location.hash().unwrap_or(String::new());
        let content = match self.contents.get(&hash[1..]) {
            Some(got) => got,
            None => "404 not found",
        };
        html! { <div class="contents">{ html2div(&content) }</div> }
    }
}
