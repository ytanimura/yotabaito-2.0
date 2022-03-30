use crate::*;

pub struct Contents {
    div: NodeRef,
}

mod texts {
    include!(concat!(env!("OUT_DIR"), "/texts.rs"));
}

fn get_text() -> &'static str {
    use qstring::QString;
    let location = gloo::utils::window().location();
    let raw_query = location.search().expect_throw("failed to get query");
    let query = QString::from(raw_query.as_str());
    let hash = query.get("doc").unwrap_or("profile");
    texts::get_texts()
        .get(&hash)
        .unwrap_or(&"<h1>404 not found</h1>")
}

impl Component for Contents {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            div: Default::default(),
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { <div class="contents" ref={ self.div.clone() }  /> }
    }

    fn rendered(&mut self, _: &Context<Self>, _: bool) {
        let div = self.div.cast::<web_sys::HtmlDivElement>().unwrap();
        div.set_inner_html(get_text());
    }
}
