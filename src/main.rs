use anyhow::{bail, Context, Result};
use konnect_codex::{
    audit_guidance, disable, doctor, enable, native_status, run_hook, run_mcp, sync, uninstall,
    CompanionPaths, OperationReport, SyncOptions,
};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");
    let paths = CompanionPaths::for_current_user()?;

    match command {
        "sync" => {
            let parsed = parse_sync_args(&args[1..])?;
            let report = sync(SyncOptions {
                paths,
                source: parsed.source,
                konnect_binary: parsed.konnect.unwrap_or(locate_konnect()?),
                config_source: parsed.config.or_else(default_config),
                adapter_binary: env::current_exe()?,
                activate: !parsed.no_activate,
                dry_run: parsed.dry_run,
                prefer_native_skills: parsed.prefer_native_skills,
            })?;
            print_report(report);
        }
        "audit" => {
            let parsed = parse_audit_args(&args[1..])?;
            let konnect = parsed.konnect.unwrap_or(locate_konnect()?);
            print_report(audit_guidance(
                parsed.source.as_deref(),
                &konnect,
                &paths.home,
            )?);
        }
        "disable" => print_report(disable(&paths)?),
        "enable" => print_report(enable(&paths)?),
        "doctor" => print_report(doctor(&paths)?),
        "native-status" => print_report(native_status(&paths)?),
        "uninstall" => {
            let force = args[1..].iter().any(|arg| arg == "--force");
            reject_unknown_flags(&args[1..], &["--force"])?;
            print_report(uninstall(&paths, force)?);
        }
        "mcp" => {
            let code = run_mcp(&paths)?;
            std::process::exit(code);
        }
        "hook" => {
            let name = args.get(1).context("hook requires a hook name")?;
            run_hook(name, args.get(2).map(String::as_str), &paths)?;
        }
        "help" | "--help" | "-h" => print_help(),
        "--version" | "-V" => println!("konnect-codex {}", env!("CARGO_PKG_VERSION")),
        other => bail!("unknown command '{other}'; run `konnect-codex help`"),
    }
    Ok(())
}

#[derive(Default)]
struct ParsedSyncArgs {
    source: Option<PathBuf>,
    konnect: Option<PathBuf>,
    config: Option<PathBuf>,
    no_activate: bool,
    dry_run: bool,
    prefer_native_skills: bool,
}

fn parse_sync_args(args: &[String]) -> Result<ParsedSyncArgs> {
    let mut parsed = ParsedSyncArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--source" => {
                parsed.source = Some(required_path(args, index, "--source")?);
                index += 2;
            }
            "--konnect" => {
                parsed.konnect = Some(required_path(args, index, "--konnect")?);
                index += 2;
            }
            "--config" => {
                parsed.config = Some(required_path(args, index, "--config")?);
                index += 2;
            }
            "--no-activate" => {
                parsed.no_activate = true;
                index += 1;
            }
            "--dry-run" => {
                parsed.dry_run = true;
                index += 1;
            }
            "--prefer-native-skills" => {
                parsed.prefer_native_skills = true;
                index += 1;
            }
            other => bail!("unknown sync option '{other}'"),
        }
    }
    Ok(parsed)
}

#[derive(Default)]
struct ParsedAuditArgs {
    source: Option<PathBuf>,
    konnect: Option<PathBuf>,
}

fn parse_audit_args(args: &[String]) -> Result<ParsedAuditArgs> {
    let mut parsed = ParsedAuditArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--source" => {
                parsed.source = Some(required_path(args, index, "--source")?);
                index += 2;
            }
            "--konnect" => {
                parsed.konnect = Some(required_path(args, index, "--konnect")?);
                index += 2;
            }
            other => bail!("unknown audit option '{other}'"),
        }
    }
    Ok(parsed)
}

fn required_path(args: &[String], index: usize, flag: &str) -> Result<PathBuf> {
    args.get(index + 1)
        .map(PathBuf::from)
        .with_context(|| format!("{flag} requires a path"))
}

fn reject_unknown_flags(args: &[String], allowed: &[&str]) -> Result<()> {
    if let Some(unknown) = args.iter().find(|arg| !allowed.contains(&arg.as_str())) {
        bail!("unknown option '{unknown}'");
    }
    Ok(())
}

fn locate_konnect() -> Result<PathBuf> {
    if let Some(path) = env::var_os("KONNECT_BINARY").map(PathBuf::from) {
        if path.is_file() {
            return Ok(path);
        }
    }
    if let Some(path) = find_on_path(if cfg!(windows) {
        "konnect.exe"
    } else {
        "konnect"
    }) {
        return Ok(path);
    }
    let current = env::current_dir()?;
    let candidates = [
        current
            .join("target")
            .join("release")
            .join(exe_name("konnect")),
        current
            .join("target")
            .join("debug")
            .join(exe_name("konnect")),
    ];
    candidates.into_iter().find(|path| path.is_file()).context(
        "could not find the Konnect executable; pass `--konnect <path>` or set KONNECT_BINARY",
    )
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}

fn exe_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_string()
    }
}

fn default_config() -> Option<PathBuf> {
    let current = env::current_dir().ok()?;
    [current.join("konnect.toml"), current.join("settings.json")]
        .into_iter()
        .find(|path| path.is_file())
}

fn print_report(report: OperationReport) {
    for message in report.messages {
        println!("{message}");
    }
}

fn print_help() {
    println!("Konnect Codex Companion v{}", env!("CARGO_PKG_VERSION"));
    println!("Reversible, capability-complete Codex integration for Konnect.\n");
    println!("USAGE:");
    println!("  konnect-codex sync [--source <path>] [--konnect <path>] [--config <path>]");
    println!("                       [--prefer-native-skills] [--no-activate] [--dry-run]");
    println!("  konnect-codex audit [--source <path>] [--konnect <path>]");
    println!("  konnect-codex doctor");
    println!("  konnect-codex native-status");
    println!("  konnect-codex disable");
    println!("  konnect-codex enable");
    println!("  konnect-codex uninstall [--force]");
    println!("  konnect-codex --version");
    println!("\nINTERNAL:");
    println!("  konnect-codex mcp");
    println!("  konnect-codex hook <user-prompt|pre-pcb-ipc|konnect-skill NAME>");
}
