use crate::*;

#[derive(Clone, Debug)]
pub struct Query {
    pub doc: Option<String>,
    pub shader: Option<String>,
}

impl Query {
    pub fn new() -> Query {
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
