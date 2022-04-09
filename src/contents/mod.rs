use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Contents {
    div: NodeRef,
    hide_doc: NodeRef,
    view_doc: NodeRef,
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
            .copied()
            .unwrap_or("<h1>404 not found</h1>")
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
        let outer0 = NodeRef::default();
        let outer1 = outer0.clone();
        let hide_doc0 = self.hide_doc.clone();
        let hide_doc1 = self.hide_doc.clone();
        let view_doc0 = self.view_doc.clone();
        let view_doc1 = self.view_doc.clone();
        html! {
            <>
            <div class="outer_box" ref={ outer0.clone() }><div class="contents" ref={ self.div.clone() } /></div>
            <img src="./hidedoc.svg" class="docswitch" hidden=false ref={ self.hide_doc.clone() }
                onclick= { move |_| {
                    let outer0 = outer0.cast::<HtmlDivElement>().unwrap();
                    outer0.style().set_property("display", "none").unwrap();
                    outer0.set_hidden(true);
                    hide_doc0.cast::<HtmlImageElement>().unwrap().set_hidden(true);
                    view_doc0.cast::<HtmlImageElement>().unwrap().set_hidden(false);
                } } />
            <img src="./viewdoc.svg" class="docswitch" hidden=true ref={ self.view_doc.clone() }
                onclick= { move |_| {
                    let outer1 = outer1.cast::<HtmlDivElement>().unwrap();
                    outer1.style().set_property("display", "block").unwrap();
                    outer1.set_hidden(false);
                    hide_doc1.cast::<HtmlImageElement>().unwrap().set_hidden(false);
                    view_doc1.cast::<HtmlImageElement>().unwrap().set_hidden(true);
                } } />
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _: bool) {
        let div = self.div.cast::<HtmlDivElement>().unwrap();
        div.set_inner_html(get_text(ctx.props().doc_name.as_deref()));
    }
}
