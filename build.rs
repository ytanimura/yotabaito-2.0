use std::fs::DirEntry;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let hash = download_selfie();
    out_texts("./texts", "/texts.rs", "&'static str", &markdown2html)?;
    let closure = move |shader| parse_shader(shader, &hash);
    out_texts("./shaders", "/shaders.rs", "ShaderSource", &closure)
}

fn out_texts(
    input_dir_path: &str,
    out_dir_path: &str,
    value_type: &str,
    closure: &impl Fn(String) -> String,
) -> std::io::Result<()> {
    let mut out_code = format!("use std::collections::HashMap;pub(crate) fn get_texts()->HashMap<&'static str,{value_type}>{{vec![");
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
        *out_code += &format!("({:?}, {}),", entry.path().file_stem().unwrap(), entity);
    }
    Ok(())
}

fn markdown2html(markdown: String) -> String {
    use pulldown_cmark::{html, *};
    let parser = Parser::new(&markdown);
    let mut res = String::new();
    html::push_html(&mut res, parser);
    format!("{:?}", res)
}

fn parse_shader(shader: String, hash: &str) -> String {
    use std::io::{BufRead, BufReader};
    let mut res = String::from("ShaderSource {");
    let mut first_line = true;
    BufReader::new(shader.as_bytes())
        .lines()
        .try_for_each(|line| -> std::io::Result<()> {
            let line = line?;
            if first_line {
                if line.len() > 10 && &line[0..10] == "#iChannel0" {
                    let path = Path::new(&line[12..line.len() - 1]);
                    let mut path = path.file_name().unwrap().to_str().unwrap();
                    if path == "selfie.jpg" {
                        path = hash;
                    }
                    res += &format!("texture_url:Some(\"./{path}\"),source:\"");
                    first_line = false;
                    return Ok(());
                } else {
                    res += "texture_url:None,source:\"";
                    first_line = false;
                }
            }
            if !line.is_empty() && (line.len() < 2 || (&line[0..2] != "//" && &line[0..2] != "/*"))
            {
                res += line.trim();
                res += "\n";
            }
            Ok(())
        })
        .unwrap();
    res + "\"}"
}

// my selfie url, disable after release page
const SELFIE_URL: &str =
    "https://drive.google.com/uc?id=1CppW3rG8--B-MdkMSoGx-o2orEcjnqRD&export=download";
const HASH_LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabscefghijklmnopqrstuvwxyz0123456789";

fn download_selfie() -> String {
    let hash = (0..50)
        .map(|_| {
            let idx = rand::random::<u32>() as usize % HASH_LETTERS.len();
            HASH_LETTERS[idx]
        })
        .collect::<Vec<u8>>();
    let path = String::from_utf8(hash).unwrap() + ".jpg";
    let out_path = String::from("dist/.stage/") + &path;
    if let Err(e) = std::process::Command::new("curl")
        .args([SELFIE_URL, "-Lo", &out_path])
        .output()
    {
        eprintln!("{e}");
    }
    path
}
