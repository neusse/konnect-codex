use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let skills = manifest_dir.join(".codex").join("skills");
    let agents = manifest_dir.join(".codex").join("agents");
    println!("cargo:rerun-if-changed={}", skills.display());
    println!("cargo:rerun-if-changed={}", agents.display());

    let generated = format!(
        "pub const REVIEWED_SKILL_FILES: &[(&str, &[u8])] = &{};\n\
         pub const REVIEWED_AGENT_FILES: &[(&str, &[u8])] = &{};\n",
        render_tree(&skills),
        render_tree(&agents)
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("reviewed_assets.rs");
    fs::write(output, generated).unwrap();
}

fn render_tree(root: &Path) -> String {
    let mut files = Vec::new();
    visit(root, &mut files);
    files.sort();
    let entries = files
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            format!(
                "({relative:?}, include_bytes!({absolute:?}) as &[u8])",
                absolute = path.to_string_lossy()
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[\n{entries}\n]")
}

fn visit(current: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(current).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            visit(&path, files);
        } else if entry.file_type().unwrap().is_file() {
            files.push(path);
        }
    }
}
