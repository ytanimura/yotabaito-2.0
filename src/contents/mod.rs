use crate::*;

pub struct Contents {
    div: NodeRef,
}

mod texts {
    include!(concat!(env!("OUT_DIR"), "/texts.rs"));
}

fn get_text() -> &'static str {
    let hash = Query::new().doc;
    if let Some(hash) = hash {
        texts::get_texts()
            .get(hash.as_str())
            .unwrap_or(&"<h1>404 not found</h1>")
    } else {
        include_str!("top-contents.html")
    }
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
        html! { <div class="contents" ref={ self.div.clone() } /> }
    }

    fn rendered(&mut self, _: &Context<Self>, _: bool) {
        let div = self.div.cast::<web_sys::HtmlDivElement>().unwrap();
        div.set_inner_html(get_text());
    }
}
