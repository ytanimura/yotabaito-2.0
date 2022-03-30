use std::fs::DirEntry;
use std::path::Path;

fn main() -> std::io::Result<()> {
    out_texts("./texts", "/texts.rs", &markdown2html)?;
    out_texts("./shaders", "/shaders.rs", &optimize_shader)
}

fn out_texts(
    input_dir_path: &str,
    out_dir_path: &str,
    closure: &impl Fn(String) -> String,
) -> std::io::Result<()> {
    let mut out_code = String::from(
        "use std::collections::HashMap;pub(crate) fn get_texts()->HashMap<&'static str,&'static str>{vec!["
    );
    set_text_dir(&mut out_code, input_dir_path, closure)?;
    out_code += "].into_iter().collect()}";
    std::fs::write(std::env::var("OUT_DIR").unwrap() + out_dir_path, &out_code)
}

fn set_text_dir(
    out_code: &mut String,
    path: impl AsRef<Path>,
    closure: &impl Fn(String) -> String,
) -> std::io::Result<()> {
    std::fs::read_dir(path)?.try_for_each(move |entry| set_text(out_code, entry?, closure))
}

fn set_text(
    out_code: &mut String,
    entry: DirEntry,
    closure: &impl Fn(String) -> String,
) -> std::io::Result<()> {
    if entry.file_type()?.is_dir() {
        set_text_dir(out_code, entry.path(), closure)?;
    } else {
        let file_entity = std::fs::read_to_string(entry.path())?;
        let entity = closure(file_entity);
        *out_code += &format!("({:?}, {:?}),", entry.path().file_stem().unwrap(), entity);
    }
    Ok(())
}

fn markdown2html(markdown: String) -> String {
    use pulldown_cmark::{html, *};
    let parser = Parser::new(&markdown);
    let mut res = String::new();
    html::push_html(&mut res, parser);
    res
}

fn optimize_shader(shader: String) -> String {
    use std::io::{BufRead, BufReader};
    let mut res = String::new();
    BufReader::new(shader.as_bytes())
        .lines()
        .try_for_each(|line| -> std::io::Result<()> {
            let line = line?;
            if !line.is_empty() && (line.len() < 2 || (&line[0..2] != "//" && &line[0..2] != "/*"))
            {
                res += &line;
                res += "\n";
            }
            Ok(())
        })
        .unwrap();
    res
}
