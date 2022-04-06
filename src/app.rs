use crate::*;

pub struct App {
    from_mobile: bool,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let from_mobile = from_mobile();
        set_title();
        set_html_class(from_mobile);
        Self { from_mobile }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let query = Query::from_location();
        if query.doc.as_deref() == Some("none") {
            let shader_name = query.shader.unwrap_or_else(|| String::from("default"));
            html! { <background::BackGround shader_name={ shader_name } /> }
        } else {
            let doc_name = query.doc.clone();
            let shader_name = query
                .shader
                .or(query.doc)
                .unwrap_or_else(|| String::from("default"));
            let rarefaction = !self.from_mobile;
            html! {
                <>
                <navbar::NavBar rarefaction={ rarefaction } />
                <contents::Contents doc_name={ doc_name } />
                <div class="copyright">{ "Copyright © 2022 YOSHINORI TANIMURA, All right reserved." }</div>
                <iframe class="background" src={ format!("./index.html?doc=none&shader={shader_name}") } />
                </>
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Query {
    pub doc: Option<String>,
    pub shader: Option<String>,
}

impl Query {
    pub fn from_location() -> Query {
        use qstring::QString;
        let location = gloo::utils::window().location();
        let raw_query = location.search().expect_throw("failed to get query");
        let qstr = QString::from(raw_query.as_str());
        Query {
            doc: qstr.get("doc").map(String::from),
            shader: qstr.get("shader").map(String::from),
        }
    }
}

fn from_mobile() -> bool {
    let agent: String = gloo::utils::window()
        .navigator()
        .user_agent()
        .expect_throw("failed to get userAgent")
        .to_ascii_lowercase();
    gloo::console::log!(&agent);
    agent.contains("android") || agent.contains("iphone") || agent.contains("ipad")
}

fn set_title() {
    let document = gloo::utils::document();
    if let Some(doc) = Query::from_location().doc {
        document.set_title(&format!("yotabaito: {doc}"));
    }
}

fn set_html_class(from_mobile: bool) {
    let html = gloo::utils::document()
        .get_element_by_id("root")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    if from_mobile {
        html.set_class_name("mobile");
    } else {
        html.set_class_name("pc");
    }
}
