use std::path::Path;
use std::fs::DirEntry;

fn main() -> std::io::Result<()> {
    let mut out_code = String::from(
        "use std::collections::HashMap;pub(crate) fn get_markdowns()->HashMap<&'static str,&'static str>{vec!["
    );
    set_text_dir(&mut out_code, "./texts")?;
    out_code += "].into_iter().collect()}";
	std::fs::write(std::env::var("OUT_DIR").unwrap() + "/texts.rs", &out_code)
}

fn set_text_dir(out_code: &mut String, path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::read_dir(path)?.try_for_each(|entry| set_text(out_code, entry?))
}

fn set_text(out_code: &mut String, entry: DirEntry) -> std::io::Result<()> {
    if entry.file_type()?.is_dir() {
        set_text_dir(out_code, entry.path())?;
    } else {
        let markdown = std::fs::read_to_string(entry.path())?;
        let html = markdown2html(&markdown);
        *out_code += &format!("({:?}, {:?}),", entry.path().file_stem().unwrap(), html);
    }
    Ok(())
}

fn markdown2html(markdown: &str) -> String {
    use pulldown_cmark::{html, *};
    let parser = Parser::new(markdown);
    let mut res = String::new();
    html::push_html(&mut res, parser);
    res
}
