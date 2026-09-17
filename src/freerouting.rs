use crate::{pcb_preflight, OperationReport, PcbPreflightMode};
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPORT_SCRIPT: &str = r#"import pcbnew, sys
board = pcbnew.LoadBoard(sys.argv[1])
if board is None:
    raise SystemExit("could not load board")
if not pcbnew.ExportSpecctraDSN(board, sys.argv[2]):
    raise SystemExit("KiCad DSN export failed")
"#;

const IMPORT_SCRIPT: &str = r#"import pcbnew, sys
board = pcbnew.LoadBoard(sys.argv[1])
if board is None:
    raise SystemExit("could not load board")
if not pcbnew.ImportSpecctraSES(board, sys.argv[2]):
    raise SystemExit("KiCad SES import failed")
if not pcbnew.SaveBoard(sys.argv[3], board):
    raise SystemExit("KiCad board save failed")
"#;

pub fn status() -> Result<OperationReport> {
    let python = locate_kicad_python();
    let python_ready = python
        .as_ref()
        .is_some_and(|path| kicad_python_supports_specctra(path));
    let jar = locate_freerouting_jar();
    let java = locate_program(if cfg!(windows) { "java.exe" } else { "java" });
    let mut messages = Vec::new();

    messages.push(match (&python, python_ready) {
        (Some(path), true) => format!("KiCad-native DSN/SES bridge: ready ({})", path.display()),
        (Some(path), false) => format!(
            "KiCad-native DSN/SES bridge: unavailable ({} does not expose the required pcbnew APIs)",
            path.display()
        ),
        (None, _) => "KiCad-native DSN/SES bridge: unavailable (set KICAD_PYTHON to KiCad's bundled Python)".to_string(),
    });
    messages.push(match &jar {
        Some(path) => format!("Freerouting engine: ready ({})", path.display()),
        None => "Freerouting engine: unavailable (set FREEROUTING_JAR or install the KiCad Freerouting plugin)".to_string(),
    });
    messages.push(match &java {
        Some(path) => format!("Java runtime: found ({})", path.display()),
        None => "Java runtime: unavailable on PATH".to_string(),
    });
    messages.push(if python_ready && jar.is_some() && java.is_some() {
        "Companion whole-board route fallback: ready. Close PCB Editor before export, import, or route.".to_string()
    } else {
        "Companion whole-board route fallback: unavailable. Stop with INCOMPLETE if the native Konnect route is unsupported.".to_string()
    });

    Ok(OperationReport { messages })
}

pub fn export(board: &Path, dsn: &Path) -> Result<OperationReport> {
    pcb_preflight(board, PcbPreflightMode::Offline)?;
    let source_digest = file_digest(board)?;
    refuse_existing_output(dsn)?;
    ensure_parent_exists(dsn)?;
    let python = require_kicad_python()?;
    if let Err(error) = run_kicad_python(
        &python,
        EXPORT_SCRIPT,
        &[board.as_os_str(), dsn.as_os_str()],
        "DSN export",
    ) {
        let _ = fs::remove_file(dsn);
        return Err(error);
    }
    require_nonempty_file(dsn, "KiCad DSN export")?;
    sanitize_dsn(dsn)?;
    require_unchanged(board, &source_digest, "DSN export")?;
    Ok(OperationReport {
        messages: vec![
            format!("Exported KiCad-native Freerouting DSN: {}", dsn.display()),
            format!("Source board preserved: {}", board.display()),
            format!("Source SHA-256: {source_digest}"),
        ],
    })
}

pub fn import(board: &Path, ses: &Path, output: Option<&Path>) -> Result<OperationReport> {
    pcb_preflight(board, PcbPreflightMode::Offline)?;
    let source_digest = file_digest(board)?;
    require_nonempty_file(ses, "Freerouting session")?;
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_freerouted_board(board));
    refuse_existing_output(&output)?;
    ensure_parent_exists(&output)?;
    let python = require_kicad_python()?;
    if let Err(error) = run_kicad_python(
        &python,
        IMPORT_SCRIPT,
        &[board.as_os_str(), ses.as_os_str(), output.as_os_str()],
        "SES import",
    ) {
        let _ = fs::remove_file(&output);
        return Err(error);
    }
    require_nonempty_file(&output, "KiCad SES import")?;
    require_unchanged(board, &source_digest, "SES import")?;
    Ok(OperationReport {
        messages: vec![
            format!("Imported Freerouting SES into: {}", output.display()),
            format!("Original board preserved: {}", board.display()),
            format!("Original SHA-256: {source_digest}"),
            "Open only the generated board, then run Konnect inventory, unrouted, short, visual, and direct DRC acceptance checks before replacing the original.".to_string(),
        ],
    })
}

pub fn route(board: &Path, output: Option<&Path>, passes: u32) -> Result<OperationReport> {
    if passes == 0 {
        bail!("route pass limit must be greater than zero");
    }
    pcb_preflight(board, PcbPreflightMode::Offline)?;
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_freerouted_board(board));
    refuse_existing_output(&output)?;
    ensure_parent_exists(&output)?;
    let jar = locate_freerouting_jar().context(
        "Freerouting JAR not found; install the KiCad Freerouting plugin or set FREEROUTING_JAR",
    )?;
    let java = locate_program(if cfg!(windows) { "java.exe" } else { "java" })
        .context("Java was not found on PATH")?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let work = std::env::temp_dir().join(format!("konnect-codex-freerouting-{stamp}"));
    fs::create_dir_all(&work)?;
    let dsn = work.join("board.dsn");
    let ses = work.join("board.ses");

    if let Err(error) = export(board, &dsn) {
        return Err(error).context(format!("routing workspace retained at {}", work.display()));
    }
    let status = Command::new(&java)
        .args(command_args(&jar, &dsn, &ses, passes))
        .status()
        .with_context(|| format!("could not start Freerouting through {}", java.display()))?;
    if !status.success() || require_nonempty_file(&ses, "Freerouting route").is_err() {
        bail!(
            "Freerouting did not produce a session (status {status}); routing workspace retained at {}",
            work.display()
        );
    }
    let mut report = import(board, &ses, Some(&output))
        .with_context(|| format!("routing workspace retained at {}", work.display()))?;
    fs::remove_dir_all(&work)?;
    report.messages.insert(0, "Route status: UNACCEPTED. Freerouting produced an SES; direct validation still decides whether the route is usable.".to_string());
    report.messages.insert(
        1,
        format!(
            "Freerouting ran with a {passes}-pass limit via {}",
            jar.display()
        ),
    );
    Ok(report)
}

fn command_args(jar: &Path, dsn: &Path, ses: &Path, passes: u32) -> Vec<OsString> {
    vec![
        OsString::from("-jar"),
        jar.as_os_str().to_owned(),
        OsString::from("--gui.enabled=false"),
        OsString::from("-de"),
        dsn.as_os_str().to_owned(),
        OsString::from("-do"),
        ses.as_os_str().to_owned(),
        OsString::from(format!("--router.max_passes={passes}")),
    ]
}

fn refuse_existing_output(output: &Path) -> Result<()> {
    if output.exists() {
        bail!(
            "refusing to overwrite existing output: {}",
            output.display()
        );
    }
    Ok(())
}

fn ensure_parent_exists(output: &Path) -> Result<()> {
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        if !parent.is_dir() {
            bail!("output directory does not exist: {}", parent.display());
        }
    }
    Ok(())
}

fn require_nonempty_file(path: &Path, operation: &str) -> Result<()> {
    if !path.is_file() || fs::metadata(path)?.len() == 0 {
        bail!(
            "{operation} did not produce a non-empty file: {}",
            path.display()
        );
    }
    Ok(())
}

fn file_digest(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn require_unchanged(path: &Path, expected: &str, operation: &str) -> Result<()> {
    let actual = file_digest(path)?;
    if actual != expected {
        bail!(
            "source board changed during {operation}: {} (before {expected}, after {actual})",
            path.display()
        );
    }
    Ok(())
}

fn default_freerouted_board(board: &Path) -> PathBuf {
    let stem = board.file_stem().and_then(OsStr::to_str).unwrap_or("board");
    board.with_file_name(format!("{stem}.freerouted.kicad_pcb"))
}

fn require_kicad_python() -> Result<PathBuf> {
    let python = locate_kicad_python()
        .context("KiCad Python not found; set KICAD_PYTHON to KiCad's bundled Python executable")?;
    if !kicad_python_supports_specctra(&python) {
        bail!(
            "{} does not expose pcbnew.ExportSpecctraDSN and pcbnew.ImportSpecctraSES",
            python.display()
        );
    }
    Ok(python)
}

fn run_kicad_python(python: &Path, script: &str, args: &[&OsStr], operation: &str) -> Result<()> {
    let output = Command::new(python)
        .arg("-c")
        .arg(script)
        .args(args)
        .output()
        .with_context(|| format!("could not run KiCad Python for {operation}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!("KiCad {operation} failed: {stderr}");
    }
    Ok(())
}

fn sanitize_dsn(dsn: &Path) -> Result<()> {
    let raw = fs::read_to_string(dsn)?;
    let sanitized = raw.replace(['Ω', 'µ', 'Φ'], "");
    if sanitized != raw {
        fs::write(dsn, sanitized.as_bytes())?;
    }
    Ok(())
}

fn kicad_python_supports_specctra(python: &Path) -> bool {
    Command::new(python)
        .arg("-c")
        .arg("import pcbnew; assert hasattr(pcbnew, 'ExportSpecctraDSN') and hasattr(pcbnew, 'ImportSpecctraSES')")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn locate_kicad_python() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("KICAD_PYTHON").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    let mut candidates = Vec::new();
    if cfg!(windows) {
        if let Some(local) = dirs::data_local_dir() {
            for version in ["10.0", "9.0"] {
                candidates.push(
                    local
                        .join("Programs")
                        .join("KiCad")
                        .join(version)
                        .join("bin")
                        .join("python.exe"),
                );
            }
        }
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            for version in ["10.0", "9.0"] {
                candidates.push(
                    PathBuf::from(&program_files)
                        .join("KiCad")
                        .join(version)
                        .join("bin")
                        .join("python.exe"),
                );
            }
        }
    } else {
        for name in ["python3", "python"] {
            if let Some(path) = locate_program(name) {
                candidates.push(path);
            }
        }
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn locate_freerouting_jar() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("FREEROUTING_JAR").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    let documents = dirs::document_dir()?;
    for version in ["10.0", "9.0"] {
        let jar_dir = documents
            .join("KiCad")
            .join(version)
            .join("3rdparty")
            .join("plugins")
            .join("app_freerouting_kicad-plugin")
            .join("jar");
        let Ok(entries) = fs::read_dir(jar_dir) else {
            continue;
        };
        let mut jars: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|name| name.starts_with("freerouting-") && name.ends_with(".jar"))
            })
            .collect();
        jars.sort();
        if let Some(path) = jars.pop() {
            return Some(path);
        }
    }
    None
}

fn locate_program(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .map(|directory| directory.join(name))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_arguments_are_headless_and_bounded() {
        let args = command_args(
            Path::new("router.jar"),
            Path::new("input.dsn"),
            Path::new("output.ses"),
            75,
        );
        let rendered = args
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>();
        assert!(rendered.contains(&"--gui.enabled=false".into()));
        assert!(rendered.contains(&"--router.max_passes=75".into()));
        assert!(rendered.contains(&"input.dsn".into()));
        assert!(rendered.contains(&"output.ses".into()));
    }

    #[test]
    fn default_output_preserves_the_original_board() {
        assert_eq!(
            default_freerouted_board(Path::new("clock.kicad_pcb")),
            PathBuf::from("clock.freerouted.kicad_pcb")
        );
    }
}
