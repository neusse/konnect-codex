use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use std::process::Child;

pub const PLUGIN_NAME: &str = "konnect-codex";
pub const MARKETPLACE_NAME: &str = "personal";
const ADAPTER_DIR: &str = "codex-companion";
const NATIVE_SKILLS: &[&str] = &[
    "konnect",
    "kicad-schematic",
    "kicad-pcb",
    "kicad-manufacture",
    "kicad-review",
    "kicad-library",
];

const PLUGIN_MANIFEST_TEMPLATE: &str =
    include_str!("../template/konnect-codex/.codex-plugin/plugin.json");
const MCP_TEMPLATE: &str = include_str!("../template/konnect-codex/.mcp.json");
const HOOKS_TEMPLATE: &str = include_str!("../template/konnect-codex/hooks/hooks.json");
const COMPATIBILITY_JSON: &str = include_str!("../compatibility.json");
const ENHANCEMENT_POLICY_JSON: &str = include_str!("../policy/enhancements.json");
const UPSTREAM_BASELINE_JSON: &str = include_str!("../policy/upstream-baseline.json");

include!(concat!(env!("OUT_DIR"), "/reviewed_assets.rs"));

#[derive(Clone, Debug)]
pub struct CompanionPaths {
    pub home: PathBuf,
    pub plugin_dir: PathBuf,
    pub agents_dir: PathBuf,
    pub marketplace_path: PathBuf,
    pub data_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub generated_config_path: PathBuf,
    pub disabled_agents_dir: PathBuf,
    pub native_install_guard_path: PathBuf,
}

impl CompanionPaths {
    pub fn for_current_user() -> Result<Self> {
        let home = dirs::home_dir().context("could not locate the user home directory")?;
        Ok(Self::for_home(home))
    }

    pub fn for_home(home: PathBuf) -> Self {
        let data_dir = home.join(".konnect").join(ADAPTER_DIR);
        Self {
            plugin_dir: home.join("plugins").join(PLUGIN_NAME),
            agents_dir: home.join(".codex").join("agents"),
            marketplace_path: home
                .join(".agents")
                .join("plugins")
                .join("marketplace.json"),
            manifest_path: data_dir.join("manifest.json"),
            generated_config_path: data_dir.join("konnect.toml"),
            disabled_agents_dir: data_dir.join("disabled-agents"),
            native_install_guard_path: data_dir.join("native-install-guard.json"),
            home,
            data_dir,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SyncOptions {
    pub paths: CompanionPaths,
    pub source: Option<PathBuf>,
    pub konnect_binary: PathBuf,
    pub config_source: Option<PathBuf>,
    pub adapter_binary: PathBuf,
    pub activate: bool,
    pub dry_run: bool,
    pub prefer_native_skills: bool,
}

#[derive(Clone, Debug, Default)]
pub struct OperationReport {
    pub messages: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PcbPreflightMode {
    Live,
    Offline,
}

const FREEROUTING_EXPORT_SCRIPT: &str = r#"import pcbnew, sys
board = pcbnew.LoadBoard(sys.argv[1])
if board is None:
    raise SystemExit("could not load board")
if not pcbnew.ExportSpecctraDSN(board, sys.argv[2]):
    raise SystemExit("KiCad DSN export failed")
"#;

const FREEROUTING_IMPORT_SCRIPT: &str = r#"import pcbnew, sys
board = pcbnew.LoadBoard(sys.argv[1])
if board is None:
    raise SystemExit("could not load board")
if not pcbnew.ImportSpecctraSES(board, sys.argv[2]):
    raise SystemExit("KiCad SES import failed")
if not pcbnew.SaveBoard(sys.argv[3], board):
    raise SystemExit("KiCad board save failed")
"#;

impl OperationReport {
    fn push(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum InstallState {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SkillMode {
    #[default]
    Reviewed,
    NativePreferred,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ManagedRole {
    Plugin,
    Agent,
    Config,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManagedFile {
    path: PathBuf,
    sha256: String,
    role: ManagedRole,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InstallManifest {
    schema_version: u32,
    adapter_version: String,
    source_root: PathBuf,
    source_fingerprint: String,
    konnect_binary: PathBuf,
    config_source: Option<PathBuf>,
    adapter_binary: PathBuf,
    installed_at_epoch_seconds: u64,
    state: InstallState,
    marketplace_created: bool,
    #[serde(default)]
    skill_mode: SkillMode,
    #[serde(default)]
    supported_konnect_version: String,
    #[serde(default)]
    hook_contexts: BTreeMap<String, String>,
    files: Vec<ManagedFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NativeInstallGuard {
    schema_version: u32,
    marker_preexisting: bool,
}

#[derive(Clone, Debug)]
struct GeneratedFile {
    path: PathBuf,
    content: Vec<u8>,
    role: ManagedRole,
}

#[derive(Clone, Debug)]
struct SourceAssets {
    skills: Vec<(String, PathBuf)>,
    agents: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize)]
struct Compatibility {
    konnect_version: String,
    companion_revision: u32,
    konnect_commit: String,
    guidance_sha256: String,
    hook_sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
struct EnhancementPolicy {
    schema_version: u32,
    supported_konnect_version: String,
    companion_revision: u32,
    enhancements: Vec<Enhancement>,
}

#[derive(Clone, Debug, Deserialize)]
struct Enhancement {
    id: String,
    status: String,
    #[serde(default)]
    assertions: Vec<EnhancementAssertion>,
}

#[derive(Clone, Debug, Deserialize)]
struct EnhancementAssertion {
    target: String,
    contains: String,
}

#[derive(Clone, Debug, Deserialize)]
struct UpstreamBaseline {
    schema_version: u32,
    konnect_version: String,
    konnect_commit: String,
    files: Vec<UpstreamBaselineFile>,
}

#[derive(Clone, Debug, Deserialize)]
struct UpstreamBaselineFile {
    path: String,
    sha256: String,
}

fn native_codex_install_marker(paths: &CompanionPaths) -> PathBuf {
    paths.home.join(".konnect").join(".installed-codex")
}

fn acquire_native_install_guard(paths: &CompanionPaths) -> Result<()> {
    let marker = native_codex_install_marker(paths);
    if paths.native_install_guard_path.exists() {
        let guard: NativeInstallGuard =
            serde_json::from_slice(&fs::read(&paths.native_install_guard_path)?)?;
        if guard.schema_version != 1 {
            bail!(
                "unsupported native install guard schema {}",
                guard.schema_version
            );
        }
        if !marker.exists() {
            write_atomic(&marker, env!("CARGO_PKG_VERSION").as_bytes())?;
        }
        return Ok(());
    }

    let guard = NativeInstallGuard {
        schema_version: 1,
        marker_preexisting: marker.exists(),
    };
    if !guard.marker_preexisting {
        write_atomic(&marker, env!("CARGO_PKG_VERSION").as_bytes())?;
    }
    write_atomic(
        &paths.native_install_guard_path,
        &serde_json::to_vec_pretty(&guard)?,
    )
}

fn release_native_install_guard(paths: &CompanionPaths) -> Result<()> {
    if !paths.native_install_guard_path.exists() {
        return Ok(());
    }
    let guard: NativeInstallGuard =
        serde_json::from_slice(&fs::read(&paths.native_install_guard_path)?)?;
    if guard.schema_version != 1 {
        bail!(
            "unsupported native install guard schema {}",
            guard.schema_version
        );
    }
    if !guard.marker_preexisting {
        let marker = native_codex_install_marker(paths);
        if marker.exists() {
            fs::remove_file(marker)?;
        }
    }
    fs::remove_file(&paths.native_install_guard_path)?;
    Ok(())
}

pub fn sync(options: SyncOptions) -> Result<OperationReport> {
    sync_with_version_probe(options, command_version)
}

fn sync_with_version_probe<F>(options: SyncOptions, version_probe: F) -> Result<OperationReport>
where
    F: FnOnce(&Path) -> Result<String>,
{
    let compatibility = compatibility()?;
    validate_enhancement_policy(&compatibility)?;
    validate_executable(&options.konnect_binary, "Konnect")?;
    let installed_version = version_probe(&options.konnect_binary)
        .context("could not determine the installed Konnect version")?;
    ensure_compatible_konnect_version(&installed_version, &compatibility)?;
    validate_executable(&options.adapter_binary, "konnect-codex")?;

    if let Some(source) = options.source.as_deref() {
        verify_guidance_source(source, &options.konnect_binary, &compatibility)?;
    }
    let native_skills = if options.prefer_native_skills {
        discover_native_skills(&options.paths)
    } else {
        BTreeSet::new()
    };
    let old_manifest = load_manifest_if_present(&options.paths.manifest_path)?;
    let generated_config = render_config(options.config_source.as_deref())?;
    let fingerprint = reviewed_fingerprint(&native_skills, generated_config.as_bytes());
    let generated = generate_files(
        &native_skills,
        &options.paths,
        &options.adapter_binary,
        &fingerprint,
        generated_config,
    )?;

    verify_sync_targets(&generated, old_manifest.as_ref(), &options.paths)?;

    let mut report = OperationReport::default();
    report.push(format!(
        "Reviewed for Konnect v{} ({})",
        compatibility.konnect_version, compatibility.konnect_commit
    ));
    report.push(format!("Konnect: {installed_version} (compatible)"));
    report.push(format!(
        "Skills: {} native, {} reviewed plugin copies",
        native_skills.len(),
        reviewed_domain_skill_count().saturating_sub(native_skills.len())
    ));
    report.push("Codex profile: complete eager MCP catalogue".to_string());
    report.push("Reviewed hooks: 1".to_string());

    if options.dry_run {
        for file in &generated {
            report.push(format!("Would write: {}", file.path.display()));
        }
        report.push(format!("Would register: {PLUGIN_NAME}@{MARKETPLACE_NAME}"));
        return Ok(report);
    }

    fs::create_dir_all(&options.paths.data_dir)?;
    remove_stale_owned_files(&generated, old_manifest.as_ref())?;
    for file in &generated {
        write_atomic(&file.path, &file.content)?;
    }

    if old_manifest
        .as_ref()
        .is_some_and(|manifest| manifest.state == InstallState::Disabled)
    {
        for file in old_manifest
            .as_ref()
            .unwrap()
            .files
            .iter()
            .filter(|file| file.role == ManagedRole::Agent)
        {
            let disabled_path = options
                .paths
                .disabled_agents_dir
                .join(file.path.file_name().unwrap());
            if disabled_path.exists() {
                fs::remove_file(disabled_path)?;
            }
        }
    }

    let marketplace_created =
        patch_marketplace(&options.paths.marketplace_path, old_manifest.is_some())?;

    let mut manifest = InstallManifest {
        schema_version: 1,
        adapter_version: env!("CARGO_PKG_VERSION").to_string(),
        source_root: options
            .source
            .as_ref()
            .map(|path| canonical_or_original(path))
            .unwrap_or_else(|| {
                PathBuf::from(format!(
                    "embedded-konnect-{}",
                    compatibility.konnect_version
                ))
            }),
        source_fingerprint: fingerprint,
        konnect_binary: canonical_or_original(&options.konnect_binary),
        config_source: options
            .config_source
            .as_ref()
            .map(|p| canonical_or_original(p)),
        adapter_binary: canonical_or_original(&options.adapter_binary),
        installed_at_epoch_seconds: now_epoch_seconds(),
        state: InstallState::Disabled,
        marketplace_created: old_manifest
            .as_ref()
            .map(|old| old.marketplace_created)
            .unwrap_or(marketplace_created),
        skill_mode: if options.prefer_native_skills {
            SkillMode::NativePreferred
        } else {
            SkillMode::Reviewed
        },
        supported_konnect_version: compatibility.konnect_version,
        hook_contexts: BTreeMap::new(),
        files: generated
            .iter()
            .map(|file| ManagedFile {
                path: file.path.clone(),
                sha256: sha256_bytes(&file.content),
                role: file.role.clone(),
            })
            .collect(),
    };

    if !options.activate {
        fs::create_dir_all(&options.paths.disabled_agents_dir)?;
        for file in manifest
            .files
            .iter()
            .filter(|file| file.role == ManagedRole::Agent)
        {
            let disabled_path = options
                .paths
                .disabled_agents_dir
                .join(file.path.file_name().unwrap());
            fs::copy(&file.path, disabled_path)?;
            fs::remove_file(&file.path)?;
        }
    }
    write_manifest(&options.paths.manifest_path, &manifest)?;

    if options.activate {
        activate_plugin()?;
        if let Err(error) = acquire_native_install_guard(&options.paths) {
            let _ = remove_plugin_registration();
            return Err(error).context("could not arm native guidance suppression");
        }
        manifest.state = InstallState::Enabled;
        write_manifest(&options.paths.manifest_path, &manifest)?;
        report.push(format!("Enabled: {PLUGIN_NAME}@{MARKETPLACE_NAME}"));
    } else {
        report.push("Generated but not enabled (--no-activate).".to_string());
    }

    report.push(format!("Plugin: {}", options.paths.plugin_dir.display()));
    report.push(format!(
        "Agents: {}",
        manifest
            .files
            .iter()
            .filter(|file| file.role == ManagedRole::Agent)
            .count()
    ));
    Ok(report)
}

pub fn disable(paths: &CompanionPaths) -> Result<OperationReport> {
    let mut manifest = load_manifest(&paths.manifest_path)?;
    let mut report = OperationReport::default();
    if manifest.state == InstallState::Disabled {
        release_native_install_guard(paths)?;
        report.push("Konnect Codex plugin is already disabled.");
        return Ok(report);
    }

    verify_manifest_files(&manifest, paths, false)?;
    remove_plugin_registration().context("could not disable the Codex plugin")?;
    fs::create_dir_all(&paths.disabled_agents_dir)?;
    for file in manifest
        .files
        .iter()
        .filter(|file| file.role == ManagedRole::Agent)
    {
        if file.path.exists() {
            let disabled_path = paths.disabled_agents_dir.join(
                file.path
                    .file_name()
                    .context("managed agent path has no filename")?,
            );
            fs::copy(&file.path, &disabled_path)?;
            fs::remove_file(&file.path)?;
        }
    }
    manifest.state = InstallState::Disabled;
    write_manifest(&paths.manifest_path, &manifest)?;
    release_native_install_guard(paths)?;
    report.push("Disabled plugin hooks, MCP server, skills, and custom agents.");
    report.push("Generated plugin and source manifest were retained for re-enable.");
    Ok(report)
}

pub fn enable(paths: &CompanionPaths) -> Result<OperationReport> {
    let mut manifest = load_manifest(&paths.manifest_path)?;
    let mut report = OperationReport::default();
    if manifest.state == InstallState::Enabled {
        acquire_native_install_guard(paths)?;
        report.push("Konnect Codex plugin is already enabled.");
        return Ok(report);
    }

    for file in manifest
        .files
        .iter()
        .filter(|file| file.role == ManagedRole::Agent)
    {
        let disabled_path = paths.disabled_agents_dir.join(
            file.path
                .file_name()
                .context("managed agent path has no filename")?,
        );
        if file.path.exists() {
            bail!(
                "refusing to replace agent created while disabled: {}",
                file.path.display()
            );
        }
        let content = fs::read(&disabled_path).with_context(|| {
            format!(
                "disabled agent copy is missing: {}",
                disabled_path.display()
            )
        })?;
        if sha256_bytes(&content) != file.sha256 {
            bail!("disabled agent was modified: {}", disabled_path.display());
        }
    }

    activate_plugin()?;
    if let Err(error) = acquire_native_install_guard(paths) {
        let _ = remove_plugin_registration();
        return Err(error).context("could not arm native guidance suppression");
    }
    for file in manifest
        .files
        .iter()
        .filter(|file| file.role == ManagedRole::Agent)
    {
        let disabled_path = paths
            .disabled_agents_dir
            .join(file.path.file_name().unwrap());
        let content = fs::read(&disabled_path)?;
        write_atomic(&file.path, &content)?;
        fs::remove_file(disabled_path)?;
    }
    remove_dir_if_empty(&paths.disabled_agents_dir)?;
    manifest.state = InstallState::Enabled;
    write_manifest(&paths.manifest_path, &manifest)?;
    report.push("Enabled plugin skills, eager MCP server, hooks, and custom agents.");
    Ok(report)
}

pub fn uninstall(paths: &CompanionPaths, force: bool) -> Result<OperationReport> {
    let manifest = load_manifest(&paths.manifest_path)?;
    verify_manifest_files(&manifest, paths, force)?;
    verify_marketplace_entry(&paths.marketplace_path, force)?;

    let mut report = OperationReport::default();
    if manifest.state == InstallState::Enabled {
        if let Err(error) = remove_plugin_registration() {
            report.push(format!(
                "Plugin was already absent or could not be removed by Codex: {error:#}"
            ));
        }
    }

    remove_marketplace_entry(&paths.marketplace_path, manifest.marketplace_created)?;
    for file in &manifest.files {
        if file.path.exists() {
            fs::remove_file(&file.path)
                .with_context(|| format!("could not remove {}", file.path.display()))?;
        }
        if file.role == ManagedRole::Agent {
            if let Some(name) = file.path.file_name() {
                let disabled_path = paths.disabled_agents_dir.join(name);
                if disabled_path.exists() {
                    fs::remove_file(disabled_path)?;
                }
            }
        }
    }

    release_native_install_guard(paths)?;

    remove_tree_if_empty(&paths.plugin_dir)?;
    remove_dir_if_empty(&paths.disabled_agents_dir)?;
    if paths.manifest_path.exists() {
        fs::remove_file(&paths.manifest_path)?;
    }
    remove_dir_if_empty(&paths.data_dir)?;
    if let Some(parent) = paths.data_dir.parent() {
        remove_dir_if_empty(parent)?;
    }
    remove_dir_if_empty(&paths.agents_dir)?;

    report.push("Removed every plugin-owned file and marketplace entry.");
    report.push("Native Konnect skills and all Claude files were left untouched.");
    Ok(report)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcessRecord {
    pid: u32,
    parent_pid: u32,
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct McpSession {
    adapter_pid: u32,
    server_pid: u32,
    owner_pid: u32,
    owner_name: String,
}

fn process_name_is(name: &str, expected_stem: &str) -> bool {
    Path::new(name)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or(name)
        .eq_ignore_ascii_case(expected_stem)
}

fn find_mcp_sessions(processes: &[ProcessRecord]) -> Vec<McpSession> {
    let by_pid: BTreeMap<_, _> = processes
        .iter()
        .map(|process| (process.pid, process))
        .collect();
    let mut sessions = Vec::new();
    for server in processes
        .iter()
        .filter(|process| process_name_is(&process.name, "konnect"))
    {
        let Some(adapter) = by_pid.get(&server.parent_pid) else {
            continue;
        };
        if !process_name_is(&adapter.name, "konnect-codex") {
            continue;
        }
        let owner = by_pid.get(&adapter.parent_pid);
        sessions.push(McpSession {
            adapter_pid: adapter.pid,
            server_pid: server.pid,
            owner_pid: adapter.parent_pid,
            owner_name: owner
                .map(|process| process.name.clone())
                .unwrap_or_else(|| "unknown".to_string()),
        });
    }
    sessions.sort_by_key(|session| (session.adapter_pid, session.server_pid));
    sessions
}

pub fn mcp_sessions() -> Result<OperationReport> {
    let sessions = find_mcp_sessions(&process_snapshot()?);
    let mut report = OperationReport::default();
    report.push(format!("Active Konnect MCP sessions: {}", sessions.len()));
    for session in sessions {
        report.push(format!(
            "adapter PID {} -> server PID {} (owner {} PID {})",
            session.adapter_pid, session.server_pid, session.owner_name, session.owner_pid
        ));
    }
    Ok(report)
}

pub fn stop_mcp_sessions() -> Result<OperationReport> {
    let sessions = find_mcp_sessions(&process_snapshot()?);
    let mut report = OperationReport::default();
    if sessions.is_empty() {
        report.push("No active Konnect MCP sessions found.");
        return Ok(report);
    }

    // Stop the server first. The waiting adapter should then exit normally after
    // forwarding the server's status, preserving the normal MCP shutdown path.
    for session in &sessions {
        let active_sessions = find_mcp_sessions(&process_snapshot()?);
        if !active_sessions.contains(session) {
            continue;
        }
        terminate_process(session.server_pid)
            .with_context(|| format!("could not stop Konnect server PID {}", session.server_pid))?;
    }

    for _ in 0..20 {
        let active = process_snapshot()?;
        if sessions.iter().all(|session| {
            !active
                .iter()
                .any(|process| process.pid == session.adapter_pid)
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let active = process_snapshot()?;
    for session in &sessions {
        if active.iter().any(|process| {
            process.pid == session.adapter_pid
                && process.parent_pid == session.owner_pid
                && process_name_is(&process.name, "konnect-codex")
        }) {
            terminate_process(session.adapter_pid).with_context(|| {
                format!(
                    "Konnect server stopped but adapter PID {} did not exit",
                    session.adapter_pid
                )
            })?;
        }
    }

    report.push(format!(
        "Stopped {} Konnect MCP session(s).",
        sessions.len()
    ));
    report.push("Codex will start a fresh session when Konnect is needed again.");
    Ok(report)
}

pub fn doctor(paths: &CompanionPaths) -> Result<OperationReport> {
    let manifest = load_manifest(&paths.manifest_path)?;
    let compatibility = compatibility()?;
    let mut report = OperationReport::default();
    report.push(format!("State: {:?}", manifest.state).to_ascii_lowercase());
    report.push(format!("Source: {}", manifest.source_root.display()));
    report.push(format!("Skill mode: {:?}", manifest.skill_mode).to_ascii_lowercase());
    report.push(format!(
        "Supported Konnect: {}",
        manifest.supported_konnect_version
    ));
    let active_enhancement_count = validate_enhancement_policy(&compatibility)?;
    let policy = enhancement_policy()?;
    report.push(format!("Companion revision: {}", policy.companion_revision));
    report.push(format!(
        "Codex enhancements: {} active",
        active_enhancement_count
    ));
    report.push(format!(
        "Source fingerprint: {}",
        manifest.source_fingerprint
    ));

    let mut healthy_count = 0usize;
    let mut healthy_plugin_agents = 0usize;
    let plugin_agent_count = manifest
        .files
        .iter()
        .filter(|file| file.role == ManagedRole::Agent)
        .count();
    let mut unhealthy = Vec::new();
    for file in &manifest.files {
        let check_path =
            if manifest.state == InstallState::Disabled && file.role == ManagedRole::Agent {
                paths
                    .disabled_agents_dir
                    .join(file.path.file_name().unwrap())
            } else {
                file.path.clone()
            };
        match hash_file(&check_path) {
            Ok(hash) if hash == file.sha256 => {
                healthy_count += 1;
                if file.role == ManagedRole::Agent {
                    healthy_plugin_agents += 1;
                }
            }
            Ok(_) => unhealthy.push(format!("modified: {}", check_path.display())),
            Err(_) => unhealthy.push(format!("missing: {}", check_path.display())),
        }
    }
    report.push(format!(
        "Owned files: {healthy_count}/{} healthy",
        manifest.files.len()
    ));
    report.push(format!(
        "Plugin agents: {healthy_plugin_agents}/{plugin_agent_count} installed and healthy"
    ));
    report.messages.extend(unhealthy);

    let config_raw = fs::read_to_string(&paths.generated_config_path)?;
    let config: toml::Value = toml::from_str(&config_raw)?;
    let eager = config
        .get("eager_toolsets")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    report.push(format!("Eager toolsets: {eager}"));

    let version = Command::new(&manifest.konnect_binary)
        .arg("--version")
        .output()
        .with_context(|| {
            format!(
                "could not run Konnect at {}",
                manifest.konnect_binary.display()
            )
        })?;
    let installed_version = String::from_utf8_lossy(&version.stdout).trim().to_string();
    let expected_version = format!("konnect {}", compatibility.konnect_version);
    let compatible_version = installed_version == expected_version;
    report.push(format!("Konnect: {installed_version}"));
    report.push(format!("Version compatibility: {compatible_version}"));

    let marketplace_ok = marketplace_has_owned_entry(&paths.marketplace_path)?;
    report.push(format!("Marketplace entry: {marketplace_ok}"));
    let native_install_suppressed =
        paths.native_install_guard_path.exists() && native_codex_install_marker(paths).exists();
    report.push(format!(
        "Native guidance auto-install suppressed: {native_install_suppressed}"
    ));
    report.push(format!(
        "Relevant-prompt hook: {}",
        user_prompt_context("Please review this KiCad PCB").is_some()
    ));

    let session_count = find_mcp_sessions(&process_snapshot()?).len();
    report.push(format!("Active Konnect MCP sessions: {session_count}"));
    if session_count > 1 {
        report.push(format!(
            "Lifecycle warning: {session_count} sessions are active. Run `konnect-codex sessions` to inspect them or `konnect-codex stop-sessions` before an upgrade."
        ));
    }

    report.messages.extend(native_status(paths)?.messages);
    report.push(
        if healthy_count == manifest.files.len()
            && eager
            && marketplace_ok
            && compatible_version
            && (manifest.state == InstallState::Disabled || native_install_suppressed)
        {
            "Health: PASS".to_string()
        } else {
            "Health: FAIL".to_string()
        },
    );
    Ok(report)
}

pub fn native_status(paths: &CompanionPaths) -> Result<OperationReport> {
    let mut report = OperationReport::default();
    let native_skills_dir = paths.home.join(".agents").join("skills");
    let skill_count = NATIVE_SKILLS
        .iter()
        .filter(|name| native_skills_dir.join(name).join("SKILL.md").exists())
        .count();
    let native_agent_count = if paths.agents_dir.exists() {
        fs::read_dir(&paths.agents_dir)?
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                name.ends_with(".toml") && name.contains("kicad") && !name.starts_with("konnect_")
            })
            .count()
    } else {
        0
    };
    let marker = paths.home.join(".konnect").join(".installed-codex");
    report.push(format!(
        "Upstream-native Konnect coverage: {skill_count}/{} skills, {native_agent_count} agents, installer marker {}, plugin suppression {}",
        NATIVE_SKILLS.len(),
        marker.exists(),
        paths.native_install_guard_path.exists()
    ));
    if skill_count > 0 {
        report.push(
            "Native skills are installed. The plugin uses its reviewed skills by default; uninstall native Codex guidance to avoid duplicate names, or sync with --prefer-native-skills.",
        );
    } else {
        report.push("The reviewed plugin skills are the active Codex guidance.");
    }
    Ok(report)
}

#[cfg(windows)]
fn process_snapshot() -> Result<Vec<ProcessRecord>> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        bail!("could not take a Windows process snapshot");
    }

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut processes = Vec::new();
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        let length = entry
            .szExeFile
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(entry.szExeFile.len());
        processes.push(ProcessRecord {
            pid: entry.th32ProcessID,
            parent_pid: entry.th32ParentProcessID,
            name: String::from_utf16_lossy(&entry.szExeFile[..length]),
        });
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }
    unsafe { CloseHandle(snapshot) };
    Ok(processes)
}

#[cfg(not(windows))]
fn process_snapshot() -> Result<Vec<ProcessRecord>> {
    let output = Command::new("ps")
        .args(["-A", "-o", "pid=,ppid=,comm="])
        .output()
        .context("could not query processes with ps")?;
    if !output.status.success() {
        bail!("ps failed while checking Konnect MCP sessions");
    }
    let mut processes = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut fields = line.split_whitespace();
        let (Some(pid), Some(parent_pid), Some(name)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let (Ok(pid), Ok(parent_pid)) = (pid.parse(), parent_pid.parse()) else {
            continue;
        };
        processes.push(ProcessRecord {
            pid,
            parent_pid,
            name: Path::new(name)
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or(name)
                .to_string(),
        });
    }
    Ok(processes)
}

#[cfg(windows)]
fn terminate_process(pid: u32) -> Result<()> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    let process = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if process.is_null() {
        // A process that exited between discovery and cleanup is already stopped.
        if !process_snapshot()?.iter().any(|entry| entry.pid == pid) {
            return Ok(());
        }
        bail!("could not open process PID {pid} for termination");
    }
    let terminated = unsafe { TerminateProcess(process, 0) } != 0;
    unsafe { CloseHandle(process) };
    if !terminated && process_snapshot()?.iter().any(|entry| entry.pid == pid) {
        bail!("Windows could not terminate process PID {pid}");
    }
    Ok(())
}

#[cfg(not(windows))]
fn terminate_process(pid: u32) -> Result<()> {
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .context("could not invoke kill")?;
    if !status.success() && process_snapshot()?.iter().any(|entry| entry.pid == pid) {
        bail!("could not terminate process PID {pid}");
    }
    Ok(())
}

#[cfg(windows)]
struct ChildJob(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl ChildJob {
    fn assign(child: &Child) -> Result<Self> {
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            bail!("could not create a Windows job for the Konnect server");
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const std::ffi::c_void,
                std::mem::size_of_val(&limits) as u32,
            )
        } != 0;
        if !configured {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(job) };
            bail!("could not configure Konnect child-process cleanup");
        }
        let assigned = unsafe {
            AssignProcessToJobObject(
                job,
                child.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE,
            )
        } != 0;
        if !assigned {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(job) };
            bail!("could not assign the Konnect server to its cleanup job");
        }
        Ok(Self(job))
    }
}

#[cfg(windows)]
impl Drop for ChildJob {
    fn drop(&mut self) {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0) };
    }
}

pub fn run_mcp(paths: &CompanionPaths) -> Result<i32> {
    let manifest = load_manifest(&paths.manifest_path)?;
    if manifest.state != InstallState::Enabled {
        bail!("Konnect Codex plugin is disabled; run `konnect-codex enable`");
    }
    acquire_native_install_guard(paths)
        .context("could not suppress Konnect's native guidance auto-installer")?;
    let mut child = Command::new(&manifest.konnect_binary)
        .arg("--client")
        .arg("codex")
        .arg("--config")
        .arg(&paths.generated_config_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| {
            format!(
                "could not start Konnect at {}",
                manifest.konnect_binary.display()
            )
        })?;
    #[cfg(windows)]
    let _child_job = match ChildJob::assign(&child) {
        Ok(job) => job,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    let status = child
        .wait()
        .context("could not wait for the Konnect server")?;
    Ok(status.code().unwrap_or(1))
}

pub fn pcb_preflight(board: &Path, mode: PcbPreflightMode) -> Result<OperationReport> {
    validate_board_path(board)?;
    let editors = pcb_editor_process_count()?;
    match mode {
        PcbPreflightMode::Live if editors != 1 => bail!(
            "live PCB mutation requires exactly one pcbnew process; found {editors}. Close duplicate PCB Editors or open the target board before continuing"
        ),
        PcbPreflightMode::Offline if editors != 0 => bail!(
            "offline PCB mutation requires PCB Editor to be closed; found {editors} pcbnew process(es)"
        ),
        _ => {}
    }
    let mut report = OperationReport::default();
    report.push(format!("PCB preflight: {:?} mode is ready", mode));
    report.push(format!(
        "Target board: {}",
        canonical_or_original(board).display()
    ));
    report.push(format!("PCB Editor processes: {editors}"));
    if mode == PcbPreflightMode::Live {
        report.push(
            "Process ownership is valid. Confirm Konnect's live active-board path matches this target before the first mutation.",
        );
    } else {
        report.push("Offline ownership is exclusive; no live editor can race the file operation.");
    }
    Ok(report)
}

pub fn freerouting_status() -> Result<OperationReport> {
    let python = locate_kicad_python();
    let jar = locate_freerouting_jar();
    let java = locate_program(if cfg!(windows) { "java.exe" } else { "java" });
    let editors = pcb_editor_process_count()?;
    let mut report = OperationReport::default();
    match &python {
        Some(path) if kicad_python_supports_specctra(path) => {
            report.push(format!("KiCad DSN/SES bridge: ready ({})", path.display()))
        }
        Some(path) => report.push(format!(
            "KiCad DSN/SES bridge: unavailable ({} cannot import the required pcbnew API)",
            path.display()
        )),
        None => report
            .push("KiCad DSN/SES bridge: unavailable (set KICAD_PYTHON to KiCad's bundled Python)"),
    }
    report.push(match &jar {
        Some(path) => format!("Freerouting engine: ready ({})", path.display()),
        None => "Freerouting engine: unavailable (set FREEROUTING_JAR or install the KiCad Freerouting plugin)".to_string(),
    });
    report.push(match &java {
        Some(path) => format!("Java runtime: found ({})", path.display()),
        None => "Java runtime: unavailable on PATH".to_string(),
    });
    report.push(format!("PCB Editor processes: {editors}"));
    if python
        .as_ref()
        .is_some_and(|path| kicad_python_supports_specctra(path))
        && jar.is_some()
        && java.is_some()
    {
        report
            .push("Offline route bridge: ready. Close PCB Editor before export, import, or route.");
    }
    Ok(report)
}

pub fn freerouting_export(board: &Path, dsn: &Path) -> Result<OperationReport> {
    pcb_preflight(board, PcbPreflightMode::Offline)?;
    refuse_existing_output(dsn)?;
    ensure_parent_exists(dsn)?;
    let python = require_kicad_python()?;
    if let Err(error) = run_kicad_python(
        &python,
        FREEROUTING_EXPORT_SCRIPT,
        &[board.as_os_str(), dsn.as_os_str()],
        "DSN export",
    ) {
        let _ = fs::remove_file(dsn);
        return Err(error);
    }
    if !dsn.is_file() || fs::metadata(dsn)?.len() == 0 {
        bail!(
            "KiCad reported DSN export success but {} is missing or empty",
            dsn.display()
        );
    }
    sanitize_freerouting_dsn(dsn)?;
    let mut report = OperationReport::default();
    report.push(format!("Exported Freerouting DSN: {}", dsn.display()));
    report.push("Source board was not modified.");
    Ok(report)
}

pub fn freerouting_import(
    board: &Path,
    ses: &Path,
    output: Option<&Path>,
) -> Result<OperationReport> {
    pcb_preflight(board, PcbPreflightMode::Offline)?;
    if !ses.is_file() {
        bail!("Freerouting session file was not found: {}", ses.display());
    }
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_freerouted_board(board));
    refuse_existing_output(&output)?;
    ensure_parent_exists(&output)?;
    let python = require_kicad_python()?;
    if let Err(error) = run_kicad_python(
        &python,
        FREEROUTING_IMPORT_SCRIPT,
        &[board.as_os_str(), ses.as_os_str(), output.as_os_str()],
        "SES import",
    ) {
        let _ = fs::remove_file(&output);
        return Err(error);
    }
    if !output.is_file() || fs::metadata(&output)?.len() == 0 {
        bail!(
            "KiCad reported SES import success but {} is missing or empty",
            output.display()
        );
    }
    let mut report = OperationReport::default();
    report.push(format!(
        "Imported Freerouting SES into: {}",
        output.display()
    ));
    report.push(format!("Original board preserved: {}", board.display()));
    report.push(
        "Open the generated board in one PCB Editor, then run Konnect inventory, unrouted, short, and direct DRC acceptance checks before replacing the original.",
    );
    Ok(report)
}

pub fn freerouting_route(
    board: &Path,
    output: Option<&Path>,
    passes: u32,
) -> Result<OperationReport> {
    pcb_preflight(board, PcbPreflightMode::Offline)?;
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_freerouted_board(board));
    refuse_existing_output(&output)?;
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
    if let Err(error) = freerouting_export(board, &dsn) {
        return Err(error).context(format!("routing workspace retained at {}", work.display()));
    }
    let status = Command::new(&java)
        .arg("-jar")
        .arg(&jar)
        .arg("-de")
        .arg(&dsn)
        .arg("-do")
        .arg(&ses)
        .arg("-mp")
        .arg(passes.to_string())
        .status()
        .with_context(|| format!("could not start Freerouting through {}", java.display()))?;
    if !status.success() || !ses.is_file() || fs::metadata(&ses)?.len() == 0 {
        bail!(
            "Freerouting did not produce a session (status {status}); routing workspace retained at {}",
            work.display()
        );
    }
    let mut report = freerouting_import(board, &ses, Some(&output))
        .with_context(|| format!("routing workspace retained at {}", work.display()))?;
    fs::remove_dir_all(&work)?;
    report.messages.insert(
        0,
        format!(
            "Freerouting completed with a {passes}-pass limit via {}",
            jar.display()
        ),
    );
    Ok(report)
}

fn validate_board_path(board: &Path) -> Result<()> {
    if !board.is_file() {
        bail!("KiCad board was not found: {}", board.display());
    }
    let valid_extension = board
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("kicad_pcb"));
    if !valid_extension {
        bail!("expected a .kicad_pcb file: {}", board.display());
    }
    Ok(())
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
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        if !parent.is_dir() {
            bail!("output directory does not exist: {}", parent.display());
        }
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

fn sanitize_freerouting_dsn(dsn: &Path) -> Result<()> {
    let raw = fs::read_to_string(dsn)?;
    let sanitized = raw.replace(['Ω', 'µ', 'Φ'], "");
    if sanitized != raw {
        write_atomic(dsn, sanitized.as_bytes())?;
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
            .filter_map(std::result::Result::ok)
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

fn pcb_editor_process_count() -> Result<usize> {
    if cfg!(windows) {
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq pcbnew.exe", "/FO", "CSV", "/NH"])
            .output()
            .context("could not query PCB Editor processes with tasklist")?;
        if !output.status.success() {
            bail!("tasklist failed while checking PCB Editor ownership");
        }
        let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        return Ok(stdout
            .lines()
            .filter(|line| line.trim_start().starts_with("\"pcbnew.exe\""))
            .count());
    }
    let output = Command::new("ps")
        .args(["-A", "-o", "comm="])
        .output()
        .context("could not query PCB Editor processes with ps")?;
    if !output.status.success() {
        bail!("ps failed while checking PCB Editor ownership");
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| {
            Path::new(line.trim())
                .file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.eq_ignore_ascii_case("pcbnew"))
        })
        .count())
}

pub fn run_hook(name: &str, argument: Option<&str>, paths: &CompanionPaths) -> Result<()> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;
    let input: JsonValue = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
    let context = match name {
        "pre-pcb-ipc" => {
            let editors = pcb_editor_process_count().unwrap_or(usize::MAX);
            let ownership = if editors == 1 {
                "One PCB Editor process is present. Confirm Konnect's active-board path matches the target before mutation."
            } else if editors == usize::MAX {
                "PCB Editor process ownership could not be determined. Stop and run `konnect-codex pcb-preflight --board <path> --mode live`."
            } else {
                "PCB ownership is unsafe. Do not run this mutation; open exactly one PCB Editor with the target board and rerun preflight."
            };
            Some(format!(
                "This Konnect PCB operation requires exactly one responsive KiCad PCB Editor with the target board open. Detected pcbnew process count: {editors}. {ownership} Confirm a plausible component/pad inventory before mutation. If IPC is unavailable, the editor closes, or a mutator reports file fallback after live work began, stop the PCB phase, reopen the board, and retry the readiness query once. Never mix live IPC and closed-file fallback in one placement or routing sequence."
            ))
        }
        "user-prompt" => input
            .get("prompt")
            .and_then(JsonValue::as_str)
            .and_then(user_prompt_context),
        "konnect-skill" => {
            let skill_name = argument.context("konnect-skill hook requires a skill name")?;
            let manifest = load_manifest(&paths.manifest_path)?;
            Some(
                manifest
                    .hook_contexts
                    .get(skill_name)
                    .with_context(|| format!("hook guidance '{skill_name}' is not installed"))?
                    .clone(),
            )
        }
        other => bail!("unknown hook '{other}'"),
    };
    if let Some(context) = context {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "hookSpecificOutput": {
                    "hookEventName": input
                        .get("hook_event_name")
                        .and_then(JsonValue::as_str)
                        .unwrap_or(if name == "user-prompt" { "UserPromptSubmit" } else { "PreToolUse" }),
                    "additionalContext": context
                }
            }))?
        );
    }
    Ok(())
}

fn user_prompt_context(prompt: &str) -> Option<String> {
    let lower = prompt.to_ascii_lowercase();
    let relevant = [
        "kicad",
        ".kicad_",
        "schematic",
        "circuit",
        "pcb",
        "footprint",
        "gerber",
        "jlcpcb",
        "bill of materials",
        "bom",
        "mpn",
        "datasheet",
        "component sourcing",
        "design rule",
        "erc",
        "drc",
        "freerouting",
        "autoroute",
    ]
    .iter()
    .any(|term| lower.contains(term));
    relevant.then(|| {
        "This is a Konnect/KiCad task. Use the konnect-codex router and the matching bundled domain skill. Use kicad-bom for MPN, datasheet, lifecycle, sourcing, DNP, alternate, or assembly-BOM work. Make every KiCad-source change through Konnect MCP tools, use the visible eager tool catalogue directly, and finish with the strongest available validation. When delegation is available, hand custom library work to konnect_library_builder, a complete schematic build to konnect_schematic_builder, substantial PCB transfer/layout work to konnect_pcb_builder, a comprehensive final review to konnect_design_reviewer, and a read-only firmware/first-power handoff to konnect_bringup_planner. Run applicable work sequentially in library -> schematic -> BOM -> PCB -> review -> bring-up order. The PCB builder must close the visible placement gate before routing and use Freerouting by default for a complete board, with route-import inventory and direct DRC acceptance before zones or manufacturing."
            .to_string()
    })
}

fn discover_assets(source: Option<&Path>, home: &Path) -> Result<SourceAssets> {
    let requested = source
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".claude"));
    let candidates = [
        requested.clone(),
        requested.join("crates").join("konnect").join("assets"),
    ];
    let root = candidates
        .into_iter()
        .find(|candidate| candidate.join("skills").is_dir() && candidate.join("agents").is_dir())
        .with_context(|| {
            format!(
                "{} does not contain Konnect skills/ and agents/ assets",
                requested.display()
            )
        })?;

    let mut skills = Vec::new();
    for entry in fs::read_dir(root.join("skills"))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && entry.path().join("SKILL.md").is_file() {
            skills.push((
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            ));
        }
    }
    skills.sort_by(|a, b| a.0.cmp(&b.0));

    let mut agents = Vec::new();
    for entry in fs::read_dir(root.join("agents"))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension() == Some(OsStr::new("md")) {
            let raw = fs::read_to_string(&path)?;
            if raw.contains("mcp__konnect__") {
                agents.push(path);
            }
        }
    }
    agents.sort();
    if skills.is_empty() || agents.is_empty() {
        bail!(
            "Konnect source at {} must contain at least one skill and one Konnect agent",
            root.display()
        );
    }
    Ok(SourceAssets { skills, agents })
}

fn generate_files(
    native_skills: &BTreeSet<String>,
    paths: &CompanionPaths,
    adapter_binary: &Path,
    fingerprint: &str,
    generated_config: String,
) -> Result<Vec<GeneratedFile>> {
    let mut files = Vec::new();
    let companion_revision = enhancement_policy()?.companion_revision;
    let plugin_version = format!(
        "{}+codex.{}.{}",
        env!("CARGO_PKG_VERSION"),
        companion_revision,
        &fingerprint[..12]
    );

    let mut manifest: JsonValue = serde_json::from_str(PLUGIN_MANIFEST_TEMPLATE)?;
    manifest["version"] = json!(plugin_version);
    files.push(GeneratedFile {
        path: paths.plugin_dir.join(".codex-plugin").join("plugin.json"),
        content: serde_json::to_vec_pretty(&manifest)?,
        role: ManagedRole::Plugin,
    });

    let adapter_command = canonical_or_original(adapter_binary)
        .to_string_lossy()
        .into_owned();
    let mut mcp: JsonValue = serde_json::from_str(MCP_TEMPLATE)?;
    mcp["mcpServers"]["konnect"]["command"] = json!(adapter_command);
    files.push(GeneratedFile {
        path: paths.plugin_dir.join(".mcp.json"),
        content: serde_json::to_vec_pretty(&mcp)?,
        role: ManagedRole::Plugin,
    });

    let mut hooks: JsonValue = serde_json::from_str(HOOKS_TEMPLATE)?;
    if let Some(events) = hooks.get_mut("hooks").and_then(JsonValue::as_object_mut) {
        for groups in events.values_mut().filter_map(JsonValue::as_array_mut) {
            for group in groups {
                if let Some(handlers) = group.get_mut("hooks").and_then(JsonValue::as_array_mut) {
                    for handler in handlers {
                        let suffix = handler
                            .get("command")
                            .and_then(JsonValue::as_str)
                            .and_then(|command| command.split_once(" hook ").map(|(_, tail)| tail))
                            .context("hook template command is missing its hook name")?;
                        handler["command"] = json!(format!(
                            "\"{}\" hook {}",
                            canonical_or_original(adapter_binary).display(),
                            suffix
                        ));
                    }
                }
            }
        }
    }
    files.push(GeneratedFile {
        path: paths.plugin_dir.join("hooks").join("hooks.json"),
        content: serde_json::to_vec_pretty(&hooks)?,
        role: ManagedRole::Plugin,
    });

    for (relative, content) in REVIEWED_SKILL_FILES {
        let name = Path::new(relative)
            .components()
            .next()
            .context("reviewed skill path has no skill name")?
            .as_os_str()
            .to_string_lossy();
        if name != PLUGIN_NAME && native_skills.contains(name.as_ref()) {
            continue;
        }
        files.push(GeneratedFile {
            path: paths.plugin_dir.join("skills").join(relative),
            content: content.to_vec(),
            role: ManagedRole::Plugin,
        });
    }

    for (file_name, content) in REVIEWED_AGENT_FILES {
        files.push(GeneratedFile {
            path: paths.agents_dir.join(file_name),
            content: content.to_vec(),
            role: ManagedRole::Agent,
        });
    }

    files.push(GeneratedFile {
        path: paths.generated_config_path.clone(),
        content: generated_config.into_bytes(),
        role: ManagedRole::Config,
    });
    Ok(files)
}

fn discover_native_skills(paths: &CompanionPaths) -> BTreeSet<String> {
    let marker = paths.home.join(".konnect").join(".installed-codex");
    if !marker.is_file() {
        return BTreeSet::new();
    }

    let skills_dir = paths.home.join(".agents").join("skills");
    reviewed_skill_names()
        .into_iter()
        .filter(|name| name != PLUGIN_NAME)
        .filter(|name| skills_dir.join(name).join("SKILL.md").is_file())
        .collect()
}

fn render_config(source: Option<&Path>) -> Result<String> {
    let mut value = if let Some(path) = source {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("could not read Konnect config {}", path.display()))?;
        if path.extension() == Some(OsStr::new("json")) {
            let json: JsonValue = serde_json::from_str(&raw)?;
            toml::Value::try_from(json)?
        } else {
            toml::from_str::<toml::Value>(&raw)?
        }
    } else {
        toml::Value::Table(toml::map::Map::new())
    };
    let table = value
        .as_table_mut()
        .context("Konnect config root must be a table/object")?;
    table.insert("eager_toolsets".to_string(), toml::Value::Boolean(true));
    table.insert(
        "transport".to_string(),
        toml::Value::String("stdio".to_string()),
    );
    let serialized = toml::to_string_pretty(&value)?;
    Ok(format!(
        "# Generated by konnect-codex. Edit the source config and run sync again.\n{serialized}"
    ))
}

fn compatibility() -> Result<Compatibility> {
    Ok(serde_json::from_str(COMPATIBILITY_JSON)?)
}

fn enhancement_policy() -> Result<EnhancementPolicy> {
    Ok(serde_json::from_str(ENHANCEMENT_POLICY_JSON)?)
}

fn upstream_baseline() -> Result<UpstreamBaseline> {
    Ok(serde_json::from_str(UPSTREAM_BASELINE_JSON)?)
}

fn reviewed_asset_content(target: &str) -> Option<&'static [u8]> {
    if let Some(path) = target.strip_prefix("skills/") {
        REVIEWED_SKILL_FILES
            .iter()
            .find(|(candidate, _)| *candidate == path)
            .map(|(_, content)| *content)
    } else if let Some(path) = target.strip_prefix("agents/") {
        REVIEWED_AGENT_FILES
            .iter()
            .find(|(candidate, _)| *candidate == path)
            .map(|(_, content)| *content)
    } else {
        None
    }
}

fn validate_enhancement_policy(compatibility: &Compatibility) -> Result<usize> {
    let policy = enhancement_policy()?;
    if policy.schema_version != 1
        || policy.supported_konnect_version != compatibility.konnect_version
        || policy.companion_revision != compatibility.companion_revision
    {
        bail!("Codex enhancement policy metadata does not match compatibility.json");
    }

    let mut ids = BTreeSet::new();
    let mut active_count = 0usize;
    for enhancement in policy.enhancements {
        if enhancement.id.trim().is_empty() || !ids.insert(enhancement.id.clone()) {
            bail!("Codex enhancement policy contains a missing or duplicate id");
        }
        if enhancement.status == "active" {
            active_count += 1;
        } else if enhancement.status != "retired" {
            bail!(
                "Codex enhancement {} has unsupported status {}",
                enhancement.id,
                enhancement.status
            );
        }
        for assertion in enhancement.assertions {
            let content = reviewed_asset_content(&assertion.target).with_context(|| {
                format!(
                    "Codex enhancement {} targets missing asset {}",
                    enhancement.id, assertion.target
                )
            })?;
            let raw = std::str::from_utf8(content)?;
            if !raw.contains(&assertion.contains) {
                bail!(
                    "Codex enhancement {} lost assertion {:?} in {}",
                    enhancement.id,
                    assertion.contains,
                    assertion.target
                );
            }
        }
    }
    Ok(active_count)
}

pub fn audit_guidance(
    source: Option<&Path>,
    konnect_binary: &Path,
    home: &Path,
) -> Result<OperationReport> {
    validate_executable(konnect_binary, "Konnect")?;
    let compatibility = compatibility()?;
    let assets = discover_assets(source, home)?;
    verify_upstream_baseline(&assets, &compatibility)?;
    let guidance = guidance_fingerprint(&assets)?;
    let hook = hook_fingerprint(konnect_binary)?;
    let version = command_version(konnect_binary)?;
    let expected_version = format!("konnect {}", compatibility.konnect_version);

    if version != expected_version
        || guidance != compatibility.guidance_sha256
        || hook != compatibility.hook_sha256
    {
        bail!(
            "Konnect guidance drift detected.\n  version: {version} (expected {expected_version})\n  guidance: {guidance} (expected {})\n  hook: {hook} (expected {})\nReview the upstream changes before publishing a matching konnect-codex release.",
            compatibility.guidance_sha256,
            compatibility.hook_sha256
        );
    }

    let mut report = OperationReport::default();
    report.push(format!("Konnect version: {version}"));
    report.push(format!("Reviewed commit: {}", compatibility.konnect_commit));
    report.push(format!(
        "Upstream baseline: {} files verified",
        upstream_baseline()?.files.len()
    ));
    report.push(format!("Guidance fingerprint: {guidance}"));
    report.push(format!("Hook fingerprint: {hook}"));
    report.push("Compatibility audit: PASS");
    Ok(report)
}

fn verify_guidance_source(
    source: &Path,
    konnect_binary: &Path,
    compatibility: &Compatibility,
) -> Result<()> {
    let assets = discover_assets(Some(source), Path::new("."))?;
    verify_upstream_baseline(&assets, compatibility)?;
    let guidance = guidance_fingerprint(&assets)?;
    let hook = hook_fingerprint(konnect_binary)?;
    if guidance != compatibility.guidance_sha256 || hook != compatibility.hook_sha256 {
        bail!(
            "the requested source does not match reviewed Konnect v{}; run `konnect-codex audit --source {}` for details",
            compatibility.konnect_version,
            source.display()
        );
    }
    Ok(())
}

fn command_version(konnect_binary: &Path) -> Result<String> {
    let output = Command::new(konnect_binary).arg("--version").output()?;
    if !output.status.success() {
        bail!(
            "Konnect --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn ensure_compatible_konnect_version(
    installed_version: &str,
    compatibility: &Compatibility,
) -> Result<()> {
    let expected_version = format!("konnect {}", compatibility.konnect_version);
    if installed_version != expected_version {
        bail!(
            "Konnect version mismatch: found `{installed_version}`, but konnect-codex v{} requires `{expected_version}`. Install Konnect v{} and run sync again; no plugin files were changed.",
            env!("CARGO_PKG_VERSION"),
            compatibility.konnect_version
        );
    }
    Ok(())
}

fn hook_fingerprint(konnect_binary: &Path) -> Result<String> {
    let output = Command::new(konnect_binary)
        .args(["skill", "pre-pcb-ipc"])
        .output()?;
    if !output.status.success() {
        bail!(
            "Konnect could not provide pre-pcb-ipc guidance: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(sha256_bytes(&normalize_newlines(&output.stdout)))
}

fn guidance_fingerprint(assets: &SourceAssets) -> Result<String> {
    let mut hasher = Sha256::new();
    for (name, dir) in &assets.skills {
        hasher.update(name.as_bytes());
        for (relative, path) in walk_files(dir)? {
            let relative = relative.to_string_lossy().replace('\\', "/");
            hasher.update(relative.as_bytes());
            hasher.update(normalize_newlines(&fs::read(path)?));
        }
    }
    for path in &assets.agents {
        hasher.update(path.file_name().unwrap().to_string_lossy().as_bytes());
        hasher.update(normalize_newlines(&fs::read(path)?));
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn upstream_file_hashes(assets: &SourceAssets) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    for (name, dir) in &assets.skills {
        for (relative, path) in walk_files(dir)? {
            let relative = relative.to_string_lossy().replace('\\', "/");
            files.insert(
                format!("skills/{name}/{relative}"),
                sha256_bytes(&normalize_newlines(&fs::read(path)?)),
            );
        }
    }
    for path in &assets.agents {
        files.insert(
            format!("agents/{}", path.file_name().unwrap().to_string_lossy()),
            sha256_bytes(&normalize_newlines(&fs::read(path)?)),
        );
    }
    Ok(files)
}

fn verify_upstream_baseline(assets: &SourceAssets, compatibility: &Compatibility) -> Result<()> {
    let baseline = upstream_baseline()?;
    if baseline.schema_version != 1
        || baseline.konnect_version != compatibility.konnect_version
        || baseline.konnect_commit != compatibility.konnect_commit
    {
        bail!("upstream baseline metadata does not match compatibility.json");
    }

    let expected: BTreeMap<_, _> = baseline
        .files
        .into_iter()
        .map(|file| (file.path, file.sha256))
        .collect();
    let actual = upstream_file_hashes(assets)?;
    let mut differences = Vec::new();

    for (path, expected_hash) in &expected {
        match actual.get(path) {
            None => differences.push(format!("removed: {path}")),
            Some(actual_hash) if actual_hash != expected_hash => {
                differences.push(format!("modified: {path}"));
            }
            Some(_) => {}
        }
    }
    for path in actual.keys() {
        if !expected.contains_key(path) {
            differences.push(format!("added: {path}"));
        }
    }

    if !differences.is_empty() {
        bail!(
            "Konnect upstream baseline drift detected:\n  {}\nReview every changed asset and update the baseline plus active enhancement decisions before release.",
            differences.join("\n  ")
        );
    }
    Ok(())
}

fn reviewed_fingerprint(native_skills: &BTreeSet<String>, config: &[u8]) -> String {
    let mut hasher = Sha256::new();
    for (path, content) in REVIEWED_SKILL_FILES {
        hasher.update(path.as_bytes());
        hasher.update(normalize_newlines(content));
    }
    for (path, content) in REVIEWED_AGENT_FILES {
        hasher.update(path.as_bytes());
        hasher.update(normalize_newlines(content));
    }
    for name in native_skills {
        hasher.update(b"native-skill\0");
        hasher.update(name.as_bytes());
    }
    hasher.update(normalize_newlines(config));
    format!("{:x}", hasher.finalize())
}

fn reviewed_skill_names() -> BTreeSet<String> {
    REVIEWED_SKILL_FILES
        .iter()
        .filter_map(|(path, _)| Path::new(path).components().next())
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect()
}

fn reviewed_domain_skill_count() -> usize {
    reviewed_skill_names()
        .into_iter()
        .filter(|name| name != PLUGIN_NAME)
        .count()
}

fn walk_files(root: &Path) -> Result<Vec<(PathBuf, PathBuf)>> {
    fn visit(root: &Path, current: &Path, files: &mut Vec<(PathBuf, PathBuf)>) -> Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                visit(root, &path, files)?;
            } else if entry.file_type()?.is_file() {
                files.push((path.strip_prefix(root)?.to_path_buf(), path));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn verify_sync_targets(
    generated: &[GeneratedFile],
    old: Option<&InstallManifest>,
    paths: &CompanionPaths,
) -> Result<()> {
    if let Some(old) = old.filter(|manifest| manifest.state == InstallState::Disabled) {
        for file in old
            .files
            .iter()
            .filter(|file| file.role == ManagedRole::Agent)
        {
            let disabled_path = paths
                .disabled_agents_dir
                .join(file.path.file_name().unwrap());
            if disabled_path.exists() && hash_file(&disabled_path)? != file.sha256 {
                bail!(
                    "disabled managed agent was modified; preserve it before sync: {}",
                    disabled_path.display()
                );
            }
        }
    }
    for file in generated {
        if !file.path.exists() {
            continue;
        }
        let Some(old_file) =
            old.and_then(|manifest| manifest.files.iter().find(|owned| owned.path == file.path))
        else {
            bail!(
                "refusing to overwrite an unowned path: {}",
                file.path.display()
            );
        };
        let current_hash = hash_file(&file.path)?;
        if current_hash != old_file.sha256 {
            bail!(
                "managed file was modified; preserve or remove it before sync: {}",
                file.path.display()
            );
        }
    }
    Ok(())
}

fn remove_stale_owned_files(
    generated: &[GeneratedFile],
    old: Option<&InstallManifest>,
) -> Result<()> {
    let generated_paths: BTreeSet<&Path> =
        generated.iter().map(|file| file.path.as_path()).collect();
    if let Some(old) = old {
        for file in &old.files {
            if !generated_paths.contains(file.path.as_path()) && file.path.exists() {
                if hash_file(&file.path)? != file.sha256 {
                    bail!("stale managed file was modified: {}", file.path.display());
                }
                fs::remove_file(&file.path)?;
            }
        }
    }
    Ok(())
}

fn verify_manifest_files(
    manifest: &InstallManifest,
    paths: &CompanionPaths,
    force: bool,
) -> Result<()> {
    if force {
        return Ok(());
    }
    let mut modified = Vec::new();
    for file in &manifest.files {
        let path = if manifest.state == InstallState::Disabled && file.role == ManagedRole::Agent {
            paths
                .disabled_agents_dir
                .join(file.path.file_name().unwrap())
        } else {
            file.path.clone()
        };
        if path.exists() && hash_file(&path)? != file.sha256 {
            modified.push(path);
        }
    }
    if !modified.is_empty() {
        bail!(
            "managed files were modified; rerun with --force only if they may be discarded:\n{}",
            modified
                .iter()
                .map(|path| format!("  {}", path.display()))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    Ok(())
}

fn patch_marketplace(path: &Path, is_update: bool) -> Result<bool> {
    let created = !path.exists();
    let mut root = if created {
        json!({
            "name": MARKETPLACE_NAME,
            "interface": { "displayName": "Personal" },
            "plugins": []
        })
    } else {
        serde_json::from_str(&fs::read_to_string(path)?)?
    };
    let name = root
        .get("name")
        .and_then(JsonValue::as_str)
        .context("personal marketplace is missing its name")?;
    if name != MARKETPLACE_NAME {
        bail!(
            "marketplace at {} is named '{name}', expected '{MARKETPLACE_NAME}'",
            path.display()
        );
    }
    let plugins = root
        .get_mut("plugins")
        .and_then(JsonValue::as_array_mut)
        .context("personal marketplace plugins field is not an array")?;
    if let Some(existing) = plugins
        .iter_mut()
        .find(|entry| entry.get("name").and_then(JsonValue::as_str) == Some(PLUGIN_NAME))
    {
        if !is_update {
            bail!("marketplace already contains an unowned '{PLUGIN_NAME}' entry");
        }
        let existing_path = existing
            .pointer("/source/path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        if existing_path != "./plugins/konnect-codex" {
            bail!("existing '{PLUGIN_NAME}' marketplace entry points elsewhere");
        }
        *existing = marketplace_entry();
    } else {
        plugins.push(marketplace_entry());
    }
    write_atomic(path, &serde_json::to_vec_pretty(&root)?)?;
    Ok(created)
}

fn marketplace_entry() -> JsonValue {
    json!({
        "name": PLUGIN_NAME,
        "source": {
            "source": "local",
            "path": "./plugins/konnect-codex"
        },
        "policy": {
            "installation": "AVAILABLE",
            "authentication": "ON_INSTALL"
        },
        "category": "Productivity"
    })
}

fn marketplace_has_owned_entry(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let root: JsonValue = serde_json::from_str(&fs::read_to_string(path)?)?;
    Ok(root
        .get("plugins")
        .and_then(JsonValue::as_array)
        .is_some_and(|plugins| plugins.iter().any(|entry| *entry == marketplace_entry())))
}

fn verify_marketplace_entry(path: &Path, force: bool) -> Result<()> {
    if force || marketplace_has_owned_entry(path)? {
        return Ok(());
    }
    bail!(
        "the '{PLUGIN_NAME}' marketplace entry was removed or modified; use --force only if the plugin entry may be discarded"
    )
}

fn remove_marketplace_entry(path: &Path, remove_empty_marketplace: bool) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut root: JsonValue = serde_json::from_str(&fs::read_to_string(path)?)?;
    let plugins = root
        .get_mut("plugins")
        .and_then(JsonValue::as_array_mut)
        .context("personal marketplace plugins field is not an array")?;
    plugins.retain(|entry| entry.get("name").and_then(JsonValue::as_str) != Some(PLUGIN_NAME));
    if remove_empty_marketplace && plugins.is_empty() {
        fs::remove_file(path)?;
        if let Some(parent) = path.parent() {
            remove_dir_if_empty(parent)?;
        }
    } else {
        write_atomic(path, &serde_json::to_vec_pretty(&root)?)?;
    }
    Ok(())
}

fn activate_plugin() -> Result<()> {
    run_codex_plugin_command("add")
}

fn remove_plugin_registration() -> Result<()> {
    run_codex_plugin_command("remove")
}

fn run_codex_plugin_command(action: &str) -> Result<()> {
    let output = Command::new("codex")
        .args(["plugin", action, "konnect-codex@personal", "--json"])
        .output()
        .with_context(|| "could not run the Codex CLI; ensure `codex` is on PATH")?;
    if !output.status.success() {
        bail!(
            "`codex plugin {action}` failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn load_manifest_if_present(path: &Path) -> Result<Option<InstallManifest>> {
    if path.exists() {
        Ok(Some(load_manifest(path)?))
    } else {
        Ok(None)
    }
}

fn load_manifest(path: &Path) -> Result<InstallManifest> {
    let raw = fs::read_to_string(path).with_context(|| {
        format!(
            "Konnect Codex plugin is not installed (missing {})",
            path.display()
        )
    })?;
    let manifest: InstallManifest = serde_json::from_str(&raw)?;
    if manifest.schema_version != 1 {
        bail!(
            "unsupported plugin manifest schema {}",
            manifest.schema_version
        );
    }
    Ok(manifest)
}

fn write_manifest(path: &Path, manifest: &InstallManifest) -> Result<()> {
    write_atomic(path, &serde_json::to_vec_pretty(manifest)?)
}

fn write_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)?;
    let temp_path = parent.join(format!(
        ".{}.konnect-codex-{}.tmp",
        path.file_name()
            .unwrap_or_else(|| OsStr::new("file"))
            .to_string_lossy(),
        std::process::id()
    ));
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(content)?;
        file.sync_all()?;
    }
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&temp_path, path)?;
    Ok(())
}

fn validate_executable(path: &Path, label: &str) -> Result<()> {
    if !path.is_file() {
        bail!("{label} executable was not found: {}", path.display());
    }
    Ok(())
}

fn canonical_or_original(path: &Path) -> PathBuf {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    #[cfg(windows)]
    {
        let rendered = canonical.to_string_lossy();
        if let Some(rest) = rendered.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = rendered.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    canonical
}

fn sha256_bytes(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn normalize_newlines(content: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(content.len());
    let mut index = 0;
    while index < content.len() {
        if content[index] == b'\r' && content.get(index + 1) == Some(&b'\n') {
            normalized.push(b'\n');
            index += 2;
        } else {
            normalized.push(content[index]);
            index += 1;
        }
    }
    normalized
}

fn hash_file(path: &Path) -> Result<String> {
    Ok(sha256_bytes(&fs::read(path)?))
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn remove_tree_if_empty(root: &Path) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let mut dirs = Vec::new();
    fn collect(path: &Path, dirs: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                collect(&entry.path(), dirs)?;
                dirs.push(entry.path());
            }
        }
        Ok(())
    }
    collect(root, &mut dirs)?;
    dirs.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for dir in dirs {
        remove_dir_if_empty(&dir)?;
    }
    remove_dir_if_empty(root)
}

fn remove_dir_if_empty(path: &Path) -> Result<()> {
    if path.is_dir() && fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn release_versions_are_synchronized() {
        let compatibility = compatibility().unwrap();
        let policy = enhancement_policy().unwrap();
        let baseline = upstream_baseline().unwrap();
        assert_eq!(compatibility.konnect_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(compatibility.companion_revision, policy.companion_revision);
        assert_eq!(policy.schema_version, 1);
        assert_eq!(
            policy.supported_konnect_version,
            compatibility.konnect_version
        );
        assert_eq!(baseline.schema_version, 1);
        assert_eq!(baseline.konnect_version, compatibility.konnect_version);
        assert_eq!(baseline.konnect_commit, compatibility.konnect_commit);

        let manifest: JsonValue = serde_json::from_str(PLUGIN_MANIFEST_TEMPLATE).unwrap();
        assert_eq!(
            manifest.get("version").and_then(JsonValue::as_str),
            Some(env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(compatibility.guidance_sha256.len(), 64);
        assert_eq!(compatibility.hook_sha256.len(), 64);
    }

    #[test]
    fn active_enhancements_are_present_in_reviewed_assets() {
        let policy = enhancement_policy().unwrap();
        let active: Vec<_> = policy
            .enhancements
            .iter()
            .filter(|enhancement| enhancement.status == "active")
            .collect();
        assert_eq!(active.len(), 20);

        let expected_ids = BTreeSet::from([
            "agent-delegation",
            "schematic-evidence-and-collision-gate",
            "schematic-layout-readability-gate",
            "pcb-transfer-integrity",
            "contradictory-verifier-gate",
            "requirements-based-review-defaults",
            "doctor-agent-reporting",
            "native-auto-install-suppression",
            "pcb-builder-delegation",
            "freerouting-first-routing",
            "pcb-live-state-and-placement-gates",
            "custom-part-physical-pin-acceptance",
            "visual-placement-checkpoint",
            "offline-freerouting-bridge",
            "pcb-ownership-preflight",
            "eco-and-power-layout-branches",
            "firmware-bringup-handoff",
            "legacy-sourcing-and-review-evidence",
            "evidence-grounded-review-methodology",
            "bom-lifecycle-workflow",
        ]);
        let actual_ids: BTreeSet<_> = active
            .iter()
            .map(|enhancement| enhancement.id.as_str())
            .collect();
        assert_eq!(actual_ids, expected_ids);

        for enhancement in active {
            for assertion in &enhancement.assertions {
                let content = reviewed_asset_content(&assertion.target)
                    .unwrap_or_else(|| panic!("missing enhancement target {}", assertion.target));
                let raw = std::str::from_utf8(content).unwrap();
                assert!(
                    raw.contains(&assertion.contains),
                    "enhancement {} lost assertion {:?} in {}",
                    enhancement.id,
                    assertion.contains,
                    assertion.target
                );
            }
        }
    }

    #[test]
    fn upstream_baseline_has_unique_normalized_hashes_for_every_asset() {
        let baseline = upstream_baseline().unwrap();
        assert_eq!(baseline.files.len(), 17);
        let mut paths = BTreeSet::new();
        for file in baseline.files {
            assert!(paths.insert(file.path));
            assert_eq!(file.sha256.len(), 64);
            assert!(file
                .sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn native_status_distinguishes_upstream_agents_from_plugin_agents() {
        let temp = TempDir::new().unwrap();
        let paths = CompanionPaths::for_home(temp.path().join("home"));
        fs::create_dir_all(&paths.agents_dir).unwrap();
        fs::write(
            paths.agents_dir.join("konnect_design_reviewer.toml"),
            "managed plugin agent",
        )
        .unwrap();
        let report = native_status(&paths).unwrap().messages.join("\n");
        assert!(report.contains("Upstream-native Konnect coverage"));
        assert!(report.contains("0 agents"));
        assert!(!report.contains("recognizable agents"));
    }

    #[test]
    fn mcp_session_detection_requires_a_direct_companion_server_pair() {
        let processes = vec![
            ProcessRecord {
                pid: 10,
                parent_pid: 1,
                name: "codex.exe".to_string(),
            },
            ProcessRecord {
                pid: 20,
                parent_pid: 10,
                name: "konnect-codex.exe".to_string(),
            },
            ProcessRecord {
                pid: 30,
                parent_pid: 20,
                name: "konnect.exe".to_string(),
            },
            ProcessRecord {
                pid: 40,
                parent_pid: 10,
                name: "konnect.exe".to_string(),
            },
            ProcessRecord {
                pid: 50,
                parent_pid: 10,
                name: "konnect-codex.exe".to_string(),
            },
        ];

        assert_eq!(
            find_mcp_sessions(&processes),
            vec![McpSession {
                adapter_pid: 20,
                server_pid: 30,
                owner_pid: 10,
                owner_name: "codex.exe".to_string(),
            }]
        );
    }

    #[test]
    fn process_name_matching_is_cross_platform_and_case_insensitive() {
        assert!(process_name_is("KONNECT-CODEX.EXE", "konnect-codex"));
        assert!(process_name_is("konnect-codex", "konnect-codex"));
        assert!(!process_name_is("konnect.exe", "konnect-codex"));
    }

    #[test]
    fn native_install_guard_restores_an_absent_marker_after_plugin_use() {
        let temp = TempDir::new().unwrap();
        let paths = CompanionPaths::for_home(temp.path().join("home"));

        acquire_native_install_guard(&paths).unwrap();

        assert_eq!(
            fs::read_to_string(native_codex_install_marker(&paths)).unwrap(),
            env!("CARGO_PKG_VERSION")
        );
        assert!(paths.native_install_guard_path.exists());

        release_native_install_guard(&paths).unwrap();

        assert!(!native_codex_install_marker(&paths).exists());
        assert!(!paths.native_install_guard_path.exists());
    }

    #[test]
    fn mcp_launch_repairs_the_native_install_guard_before_starting_konnect() {
        let temp = TempDir::new().unwrap();
        let paths = CompanionPaths::for_home(temp.path().join("home"));
        let executable = std::env::current_exe().unwrap();

        sync_with_version_probe(
            SyncOptions {
                paths: paths.clone(),
                source: None,
                konnect_binary: executable.clone(),
                config_source: None,
                adapter_binary: executable,
                activate: false,
                dry_run: false,
                prefer_native_skills: false,
            },
            |_| Ok(format!("konnect {}", env!("CARGO_PKG_VERSION"))),
        )
        .unwrap();
        let mut manifest = load_manifest(&paths.manifest_path).unwrap();
        manifest.state = InstallState::Enabled;
        write_manifest(&paths.manifest_path, &manifest).unwrap();

        assert!(!native_codex_install_marker(&paths).exists());
        assert!(!paths.native_install_guard_path.exists());

        let _ = run_mcp(&paths).unwrap();

        assert!(native_codex_install_marker(&paths).exists());
        assert!(paths.native_install_guard_path.exists());
    }

    #[test]
    fn compatibility_fingerprints_ignore_platform_newlines() {
        assert_eq!(normalize_newlines(b"one\r\ntwo\r\n"), b"one\ntwo\n");
        assert_eq!(normalize_newlines(b"one\ntwo\n"), b"one\ntwo\n");
    }

    #[test]
    fn freerouting_outputs_never_replace_the_source_board_by_default() {
        let board = Path::new("clock.kicad_pcb");
        assert_eq!(
            default_freerouted_board(board),
            PathBuf::from("clock.freerouted.kicad_pcb")
        );
    }

    #[test]
    fn freerouting_dsn_sanitizer_removes_known_specctra_incompatible_characters() {
        let temp = TempDir::new().unwrap();
        let dsn = temp.path().join("board.dsn");
        fs::write(&dsn, "(net \"5V_ΩµΦ\")\n").unwrap();
        sanitize_freerouting_dsn(&dsn).unwrap();
        assert_eq!(fs::read_to_string(dsn).unwrap(), "(net \"5V_\")\n");
    }

    #[test]
    fn sync_rejects_a_missing_konnect_before_writing_files() {
        let temp = TempDir::new().unwrap();
        let paths = CompanionPaths::for_home(temp.path().join("home"));
        let error = sync_with_version_probe(
            SyncOptions {
                paths: paths.clone(),
                source: None,
                konnect_binary: temp.path().join(exe_name_for_test("missing-konnect")),
                config_source: None,
                adapter_binary: std::env::current_exe().unwrap(),
                activate: false,
                dry_run: false,
                prefer_native_skills: false,
            },
            |_| panic!("the version probe must not run for a missing executable"),
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("Konnect executable was not found"));
        assert!(!paths.plugin_dir.exists());
        assert!(!paths.data_dir.exists());
    }

    #[test]
    fn sync_rejects_a_mismatched_konnect_before_writing_files() {
        let temp = TempDir::new().unwrap();
        let paths = CompanionPaths::for_home(temp.path().join("home"));
        let fake_konnect = temp.path().join(exe_name_for_test("konnect"));
        fs::write(&fake_konnect, "fake").unwrap();
        let existing_plugin = paths.plugin_dir.join("existing-plugin.txt");
        let existing_state = paths.data_dir.join("existing-state.txt");
        fs::create_dir_all(existing_plugin.parent().unwrap()).unwrap();
        fs::create_dir_all(existing_state.parent().unwrap()).unwrap();
        fs::write(&existing_plugin, "keep plugin").unwrap();
        fs::write(&existing_state, "keep state").unwrap();

        let error = sync_with_version_probe(
            SyncOptions {
                paths: paths.clone(),
                source: None,
                konnect_binary: fake_konnect,
                config_source: None,
                adapter_binary: std::env::current_exe().unwrap(),
                activate: false,
                dry_run: false,
                prefer_native_skills: false,
            },
            |_| Ok("konnect 0.0.0".to_string()),
        )
        .unwrap_err();

        assert!(error.to_string().contains("Konnect version mismatch"));
        assert!(error.to_string().contains("no plugin files were changed"));
        assert_eq!(fs::read_to_string(existing_plugin).unwrap(), "keep plugin");
        assert_eq!(fs::read_to_string(existing_state).unwrap(), "keep state");
    }

    fn exe_name_for_test(stem: &str) -> String {
        if cfg!(windows) {
            format!("{stem}.exe")
        } else {
            stem.to_string()
        }
    }

    #[test]
    fn reviewed_skills_are_codex_native_and_complete() {
        let names = reviewed_skill_names();
        assert_eq!(names.len(), NATIVE_SKILLS.len() + 3);
        assert!(names.contains(PLUGIN_NAME));
        assert!(names.contains("kicad-bringup"));
        assert!(names.contains("kicad-bom"));
        for name in NATIVE_SKILLS {
            assert!(names.contains(*name));
        }

        for (path, content) in REVIEWED_SKILL_FILES {
            let raw = std::str::from_utf8(content).unwrap();
            if path.ends_with("SKILL.md") {
                assert!(raw.starts_with("---"), "{path} is missing frontmatter");
                assert!(raw.contains("\nname:"), "{path} is missing name");
                assert!(
                    raw.contains("\ndescription:"),
                    "{path} is missing description"
                );
                assert!(
                    !raw.contains("argument-hint:"),
                    "{path} has Claude-only frontmatter"
                );
                assert!(!raw.contains("Claude"), "{path} contains Claude wording");
            }
        }
    }

    #[test]
    fn reviewed_agents_are_valid_codex_toml() {
        assert_eq!(REVIEWED_AGENT_FILES.len(), 5);
        let names: BTreeSet<_> = REVIEWED_AGENT_FILES.iter().map(|(path, _)| *path).collect();
        assert_eq!(
            names,
            BTreeSet::from([
                "konnect_design_reviewer.toml",
                "konnect_bringup_planner.toml",
                "konnect_library_builder.toml",
                "konnect_pcb_builder.toml",
                "konnect_schematic_builder.toml",
            ])
        );
        for (path, content) in REVIEWED_AGENT_FILES {
            let raw = std::str::from_utf8(content).unwrap();
            let value: toml::Value = toml::from_str(raw).unwrap();
            assert!(
                value.get("name").and_then(toml::Value::as_str).is_some(),
                "{path}"
            );
            assert!(
                value
                    .get("developer_instructions")
                    .and_then(toml::Value::as_str)
                    .is_some(),
                "{path}"
            );
            assert!(!raw.contains("sonnet"), "{path} pins a Claude model");
            assert!(!raw.contains("maxTurns"), "{path} has Claude-only limits");
        }
    }

    #[test]
    fn generated_config_preserves_values_and_enables_eager_tools() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        fs::write(
            temp.path(),
            "kicad_cli = \"custom-kicad\"\neager_toolsets = false\ntransport = \"http\"\n",
        )
        .unwrap();
        let rendered = render_config(Some(temp.path())).unwrap();
        let value: toml::Value = toml::from_str(&rendered).unwrap();
        assert_eq!(value["kicad_cli"].as_str(), Some("custom-kicad"));
        assert_eq!(value["eager_toolsets"].as_bool(), Some(true));
        assert_eq!(value["transport"].as_str(), Some("stdio"));
    }

    #[test]
    fn prompt_hook_only_adds_context_for_relevant_work() {
        let context = user_prompt_context("Build this KiCad schematic and PCB").unwrap();
        assert!(context.contains("konnect_schematic_builder"));
        assert!(context.contains("konnect_library_builder"));
        assert!(context.contains("konnect_pcb_builder"));
        assert!(context.contains("konnect_design_reviewer"));
        assert!(context.contains("konnect_bringup_planner"));
        assert!(context.contains("kicad-bom"));
        assert!(context.contains("library -> schematic -> BOM -> PCB -> review -> bring-up"));
        assert!(context.contains("Freerouting"));
        assert!(context.contains("placement gate"));
        assert!(user_prompt_context("Use Freerouting for this board").is_some());
        assert!(user_prompt_context("Check MPN and BOM lifecycle risk").is_some());
        assert!(user_prompt_context("Refactor my web API").is_none());
    }

    #[test]
    fn pcb_hook_covers_whole_board_routing_and_state_changes() {
        let hooks: JsonValue = serde_json::from_str(HOOKS_TEMPLATE).unwrap();
        let matcher = hooks["hooks"]["PreToolUse"][0]["matcher"].as_str().unwrap();
        for tool in [
            "autoroute",
            "update_pcb_from_schematic",
            "delete_trace",
            "add_zone",
            "refill_zones",
        ] {
            assert!(matcher.contains(tool), "PCB hook misses {tool}");
        }
    }

    #[test]
    fn marketplace_round_trip_preserves_unrelated_entries() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("marketplace.json");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&json!({
                "name": "personal",
                "interface": {"displayName": "Mine"},
                "plugins": [{"name": "other", "source": {"source": "local", "path": "./plugins/other"}}]
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(!patch_marketplace(&path, false).unwrap());
        assert!(marketplace_has_owned_entry(&path).unwrap());
        remove_marketplace_entry(&path, false).unwrap();
        let root: JsonValue = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(root["interface"]["displayName"], "Mine");
        assert_eq!(root["plugins"].as_array().unwrap().len(), 1);
        assert_eq!(root["plugins"][0]["name"], "other");
    }

    #[test]
    fn sync_and_uninstall_are_reversible_without_touching_native_skills() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("home");
        let native_skill = home
            .join(".agents")
            .join("skills")
            .join("native-test")
            .join("SKILL.md");
        fs::create_dir_all(native_skill.parent().unwrap()).unwrap();
        fs::write(&native_skill, "keep me").unwrap();

        let fake_konnect = temp.path().join(if cfg!(windows) {
            "konnect.exe"
        } else {
            "konnect"
        });
        fs::write(&fake_konnect, "fake").unwrap();
        let config = temp.path().join("konnect.toml");
        fs::write(&config, "log_level = \"debug\"\n").unwrap();
        let paths = CompanionPaths::for_home(home);
        let adapter = std::env::current_exe().unwrap();

        sync_with_version_probe(
            SyncOptions {
                paths: paths.clone(),
                source: None,
                konnect_binary: fake_konnect,
                config_source: Some(config),
                adapter_binary: adapter,
                activate: false,
                dry_run: false,
                prefer_native_skills: false,
            },
            |_| Ok(format!("konnect {}", env!("CARGO_PKG_VERSION"))),
        )
        .unwrap();

        assert!(paths
            .plugin_dir
            .join("skills")
            .join("konnect")
            .join("SKILL.md")
            .exists());
        assert!(paths
            .disabled_agents_dir
            .join("konnect_schematic_builder.toml")
            .exists());
        assert!(paths
            .disabled_agents_dir
            .join("konnect_pcb_builder.toml")
            .exists());
        assert!(!paths
            .agents_dir
            .join("konnect_schematic_builder.toml")
            .exists());
        assert!(!paths.agents_dir.join("konnect_pcb_builder.toml").exists());
        assert!(marketplace_has_owned_entry(&paths.marketplace_path).unwrap());

        let native_konnect_skill = paths
            .home
            .join(".agents")
            .join("skills")
            .join("konnect")
            .join("SKILL.md");
        fs::create_dir_all(native_konnect_skill.parent().unwrap()).unwrap();
        fs::write(&native_konnect_skill, "native Konnect skill").unwrap();
        fs::create_dir_all(paths.home.join(".konnect")).unwrap();
        fs::write(
            paths.home.join(".konnect").join(".installed-codex"),
            env!("CARGO_PKG_VERSION"),
        )
        .unwrap();

        sync_with_version_probe(
            SyncOptions {
                paths: paths.clone(),
                source: None,
                konnect_binary: temp.path().join(if cfg!(windows) {
                    "konnect.exe"
                } else {
                    "konnect"
                }),
                config_source: Some(temp.path().join("konnect.toml")),
                adapter_binary: std::env::current_exe().unwrap(),
                activate: false,
                dry_run: false,
                prefer_native_skills: true,
            },
            |_| Ok(format!("konnect {}", env!("CARGO_PKG_VERSION"))),
        )
        .unwrap();

        assert!(!paths
            .plugin_dir
            .join("skills")
            .join("konnect")
            .join("SKILL.md")
            .exists());
        assert_eq!(
            fs::read_to_string(&native_konnect_skill).unwrap(),
            "native Konnect skill"
        );

        acquire_native_install_guard(&paths).unwrap();
        assert!(paths.native_install_guard_path.exists());

        uninstall(&paths, false).unwrap();
        assert!(!paths.plugin_dir.exists());
        assert!(!paths.data_dir.exists());
        assert!(!paths.marketplace_path.exists());
        assert_eq!(fs::read_to_string(native_skill).unwrap(), "keep me");
        assert!(native_codex_install_marker(&paths).exists());
    }

    #[cfg(windows)]
    #[test]
    fn command_paths_do_not_use_windows_verbatim_prefixes() {
        let current = std::env::current_exe().unwrap();
        let normalized = canonical_or_original(&current);
        assert!(!normalized.to_string_lossy().starts_with(r"\\?\"));
        assert!(normalized.is_absolute());
    }
}
