use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Contents {
    div: NodeRef,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub doc_name: Option<String>,
}

mod texts {
    include!(concat!(env!("OUT_DIR"), "/texts.rs"));
}

fn get_text(hash: Option<&str>) -> &'static str {
    if let Some(hash) = hash {
        texts::get_texts()
            .get(hash)
            .unwrap_or(&"<h1>404 not found</h1>")
    } else {
        include_str!("top-contents.html")
    }
}

impl Component for Contents {
    type Message = ();
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { <div class="contents" ref={ self.div.clone() } /> }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _: bool) {
        let div = self.div.cast::<HtmlDivElement>().unwrap();
        div.set_inner_html(get_text(ctx.props().doc_name.as_deref()));
    }
}
