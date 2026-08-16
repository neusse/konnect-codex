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
use std::time::{SystemTime, UNIX_EPOCH};

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
const OVERLAY_SKILL: &str = include_str!("../template/konnect-codex/skills/konnect-codex/SKILL.md");

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
}

#[derive(Clone, Debug, Default)]
pub struct OperationReport {
    pub messages: Vec<String>,
}

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
    hook_contexts: BTreeMap<String, String>,
    files: Vec<ManagedFile>,
}

#[derive(Clone, Debug)]
struct GeneratedFile {
    path: PathBuf,
    content: Vec<u8>,
    role: ManagedRole,
}

#[derive(Clone, Debug)]
struct SourceAssets {
    root: PathBuf,
    skills: Vec<(String, PathBuf)>,
    agents: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
struct SourceHook {
    name: String,
    event: String,
    matcher: Option<String>,
    context: String,
}

#[derive(Serialize)]
struct CodexAgent {
    name: String,
    description: String,
    developer_instructions: String,
}

pub fn sync(options: SyncOptions) -> Result<OperationReport> {
    validate_executable(&options.konnect_binary, "Konnect")?;
    validate_executable(&options.adapter_binary, "konnect-codex")?;

    let assets = discover_assets(options.source.as_deref(), &options.paths.home)?;
    let source_hooks = discover_source_hooks(&assets.root, &options.konnect_binary)?;
    let native_skills = discover_native_skills(&assets, &options.paths);
    let old_manifest = load_manifest_if_present(&options.paths.manifest_path)?;
    let generated_config = render_config(options.config_source.as_deref())?;
    let fingerprint = source_fingerprint(
        &assets,
        &source_hooks,
        &native_skills,
        generated_config.as_bytes(),
    )?;
    let generated = generate_files(
        &assets,
        &source_hooks,
        &native_skills,
        &options.paths,
        &options.adapter_binary,
        &fingerprint,
        generated_config,
    )?;

    verify_sync_targets(&generated, old_manifest.as_ref(), &options.paths)?;

    let mut report = OperationReport::default();
    report.push(format!(
        "Source: {} ({} skills, {} agents)",
        assets.root.display(),
        assets.skills.len(),
        assets.agents.len()
    ));
    report.push(format!(
        "Skills: {} native, {} companion copies",
        native_skills.len(),
        assets.skills.len().saturating_sub(native_skills.len())
    ));
    report.push("Codex profile: complete eager MCP catalogue".to_string());
    report.push(format!(
        "Translated hooks: {}",
        if source_hooks.is_empty() {
            "1 built-in fallback".to_string()
        } else {
            source_hooks.len().to_string()
        }
    ));

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
        source_root: assets.root,
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
        hook_contexts: source_hooks
            .iter()
            .map(|hook| (hook.name.clone(), hook.context.clone()))
            .collect(),
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
        report.push("Konnect Codex companion is already disabled.");
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
    report.push("Disabled plugin hooks, MCP server, skills, and custom agents.");
    report.push("Generated plugin and source manifest were retained for re-enable.");
    Ok(report)
}

pub fn enable(paths: &CompanionPaths) -> Result<OperationReport> {
    let mut manifest = load_manifest(&paths.manifest_path)?;
    let mut report = OperationReport::default();
    if manifest.state == InstallState::Enabled {
        report.push("Konnect Codex companion is already enabled.");
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

    report.push("Removed every companion-owned file and marketplace entry.");
    report.push("Native Konnect skills and all Claude files were left untouched.");
    Ok(report)
}

pub fn doctor(paths: &CompanionPaths) -> Result<OperationReport> {
    let manifest = load_manifest(&paths.manifest_path)?;
    let mut report = OperationReport::default();
    report.push(format!("State: {:?}", manifest.state).to_ascii_lowercase());
    report.push(format!("Source: {}", manifest.source_root.display()));
    report.push(format!(
        "Source fingerprint: {}",
        manifest.source_fingerprint
    ));

    let mut healthy_count = 0usize;
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
            Ok(hash) if hash == file.sha256 => healthy_count += 1,
            Ok(_) => unhealthy.push(format!("modified: {}", check_path.display())),
            Err(_) => unhealthy.push(format!("missing: {}", check_path.display())),
        }
    }
    report.push(format!(
        "Owned files: {healthy_count}/{} healthy",
        manifest.files.len()
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
    report.push(format!(
        "Konnect: {}",
        String::from_utf8_lossy(&version.stdout).trim()
    ));

    let marketplace_ok = marketplace_has_owned_entry(&paths.marketplace_path)?;
    report.push(format!("Marketplace entry: {marketplace_ok}"));
    report.push(format!(
        "Relevant-prompt hook: {}",
        user_prompt_context("Please review this KiCad PCB").is_some()
    ));

    report.messages.extend(native_status(paths)?.messages);
    report.push(
        if healthy_count == manifest.files.len() && eager && marketplace_ok {
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
        "Native Konnect coverage: {skill_count}/{} skills, {native_agent_count} recognizable agents, installer marker {}",
        NATIVE_SKILLS.len(),
        marker.exists()
    ));
    if skill_count == NATIVE_SKILLS.len() && native_agent_count >= 2 {
        report.push(
            "Native support may be approaching parity; verify hooks and eager tool discovery before uninstalling the companion.",
        );
    } else {
        report
            .push("Native support is partial; the companion is still adding material capability.");
    }
    Ok(report)
}

pub fn run_mcp(paths: &CompanionPaths) -> Result<i32> {
    let manifest = load_manifest(&paths.manifest_path)?;
    if manifest.state != InstallState::Enabled {
        bail!("Konnect Codex companion is disabled; run `konnect-codex enable`");
    }
    let status = Command::new(&manifest.konnect_binary)
        .arg("--client")
        .arg("codex")
        .arg("--config")
        .arg(&paths.generated_config_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| {
            format!(
                "could not start Konnect at {}",
                manifest.konnect_binary.display()
            )
        })?;
    Ok(status.code().unwrap_or(1))
}

pub fn run_hook(name: &str, argument: Option<&str>, paths: &CompanionPaths) -> Result<()> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;
    let input: JsonValue = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
    let context = match name {
        "pre-pcb-ipc" => Some(
            "This Konnect PCB operation requires KiCad to be running with the target board open. If IPC is unavailable, ask the user to open the .kicad_pcb file in KiCad and retry once. Preserve the error after that retry rather than looping."
                .to_string(),
        ),
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
        "design rule",
        "erc",
        "drc",
    ]
    .iter()
    .any(|term| lower.contains(term));
    relevant.then(|| {
        "This is a Konnect/KiCad task. Use the konnect-codex router and the matching bundled domain skill. Make every KiCad-source change through Konnect MCP tools, use the visible eager tool catalogue directly, and finish with the strongest available validation. For complete schematic construction or comprehensive review, use the Konnect custom agent when delegation is available."
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
    Ok(SourceAssets {
        root,
        skills,
        agents,
    })
}

fn discover_source_hooks(root: &Path, konnect_binary: &Path) -> Result<Vec<SourceHook>> {
    let settings_path = root.join("settings.json");
    if !settings_path.is_file() {
        return Ok(Vec::new());
    }
    let settings: JsonValue = serde_json::from_str(&fs::read_to_string(settings_path)?)?;
    let Some(events) = settings.get("hooks").and_then(JsonValue::as_object) else {
        return Ok(Vec::new());
    };
    let mut hooks = Vec::new();
    let mut seen = BTreeSet::new();
    for (event, groups) in events {
        let Some(groups) = groups.as_array() else {
            continue;
        };
        for group in groups {
            let matcher = group
                .get("matcher")
                .and_then(JsonValue::as_str)
                .map(str::to_string);
            let Some(handlers) = group.get("hooks").and_then(JsonValue::as_array) else {
                continue;
            };
            for handler in handlers {
                let Some(command) = handler.get("command").and_then(JsonValue::as_str) else {
                    continue;
                };
                if !command.to_ascii_lowercase().contains("konnect") {
                    continue;
                }
                let words: Vec<&str> = command.split_whitespace().collect();
                let Some(skill_index) = words.iter().position(|word| *word == "skill") else {
                    continue;
                };
                let Some(name) = words.get(skill_index + 1) else {
                    continue;
                };
                let name = name.trim_matches(['\'', '"']).to_string();
                let key = format!("{event}\0{}\0{name}", matcher.as_deref().unwrap_or(""));
                if !seen.insert(key) {
                    continue;
                }
                let output = Command::new(konnect_binary)
                    .args(["skill", &name])
                    .output()
                    .with_context(|| format!("could not read Konnect hook guidance '{name}'"))?;
                if !output.status.success() {
                    bail!(
                        "Konnect could not provide hook guidance '{name}': {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                hooks.push(SourceHook {
                    name,
                    event: event.clone(),
                    matcher: matcher.clone(),
                    context: String::from_utf8(output.stdout)?,
                });
            }
        }
    }
    hooks.sort_by(|a, b| (&a.event, &a.name).cmp(&(&b.event, &b.name)));
    Ok(hooks)
}

fn generate_files(
    assets: &SourceAssets,
    source_hooks: &[SourceHook],
    native_skills: &BTreeSet<String>,
    paths: &CompanionPaths,
    adapter_binary: &Path,
    fingerprint: &str,
    generated_config: String,
) -> Result<Vec<GeneratedFile>> {
    let mut files = Vec::new();
    let plugin_version = format!("{}+codex.{}", env!("CARGO_PKG_VERSION"), &fingerprint[..12]);

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
    if !source_hooks.is_empty() {
        let events = hooks
            .get_mut("hooks")
            .and_then(JsonValue::as_object_mut)
            .context("hook template is missing its hooks object")?;
        events.remove("PreToolUse");
        for source_hook in source_hooks {
            let groups = events
                .entry(source_hook.event.clone())
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .context("generated hook event is not an array")?;
            let mut group = json!({
                "hooks": [{
                    "type": "command",
                    "command": format!("konnect-codex hook konnect-skill {}", source_hook.name),
                    "timeout": 5,
                    "statusMessage": format!("Loading Konnect guidance: {}", source_hook.name)
                }]
            });
            if let Some(matcher) = &source_hook.matcher {
                group["matcher"] = json!(matcher);
            }
            groups.push(group);
        }
    }
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

    files.push(GeneratedFile {
        path: paths
            .plugin_dir
            .join("skills")
            .join("konnect-codex")
            .join("SKILL.md"),
        content: OVERLAY_SKILL.as_bytes().to_vec(),
        role: ManagedRole::Plugin,
    });

    for (name, skill_dir) in &assets.skills {
        if native_skills.contains(name) {
            continue;
        }
        for (relative_path, source_path) in walk_files(skill_dir)? {
            let content = fs::read(&source_path)?;
            let content = if relative_path == Path::new("SKILL.md") {
                transform_skill(&String::from_utf8(content)?)
                    .as_bytes()
                    .to_vec()
            } else {
                content
            };
            files.push(GeneratedFile {
                path: paths
                    .plugin_dir
                    .join("skills")
                    .join(name)
                    .join(relative_path),
                content,
                role: ManagedRole::Plugin,
            });
        }
    }

    let mut used_agent_names = BTreeSet::new();
    for source_path in &assets.agents {
        let raw = fs::read_to_string(source_path)?;
        let (file_name, converted) = convert_agent(&raw)?;
        if !used_agent_names.insert(file_name.clone()) {
            bail!("two source agents convert to the same Codex filename: {file_name}");
        }
        files.push(GeneratedFile {
            path: paths.agents_dir.join(file_name),
            content: converted.into_bytes(),
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

fn discover_native_skills(assets: &SourceAssets, paths: &CompanionPaths) -> BTreeSet<String> {
    let marker = paths.home.join(".konnect").join(".installed-codex");
    if !marker.is_file() {
        return BTreeSet::new();
    }

    let skills_dir = paths.home.join(".agents").join("skills");
    assets
        .skills
        .iter()
        .filter(|(name, _)| skills_dir.join(name).join("SKILL.md").is_file())
        .map(|(name, _)| name.clone())
        .collect()
}

fn transform_skill(raw: &str) -> String {
    raw.lines()
        .filter(|line| !line.trim_start().starts_with("argument-hint:"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("guides Claude", "guides Codex")
        .replace("Claude through", "Codex through")
        .replace("Claude to", "Codex to")
        .replace("Claude's", "Codex's")
}

fn convert_agent(raw: &str) -> Result<(String, String)> {
    let stripped = raw
        .strip_prefix("---")
        .context("agent is missing YAML frontmatter")?;
    let (frontmatter, body) = stripped
        .split_once("\n---")
        .context("agent YAML frontmatter is not terminated")?;
    let fields = parse_simple_frontmatter(frontmatter);
    let source_name = fields
        .get("name")
        .context("agent frontmatter is missing name")?;
    let description = fields
        .get("description")
        .context("agent frontmatter is missing description")?
        .trim_matches('"')
        .to_string();

    let stem = source_name
        .trim_end_matches("-agent")
        .trim_start_matches("kicad-")
        .replace("schematic-build", "schematic-builder")
        .replace("design-review", "design-reviewer")
        .replace('-', "_");
    let name = format!("konnect_{stem}");
    let file_name = format!("{name}.toml");

    let mut instructions = body.trim_start_matches(['\r', '\n']).to_string();
    instructions = instructions.replace(
        "Load the required toolsets immediately:",
        "The Codex companion eagerly exposes all Konnect toolsets. Confirm the required tools are visible; use these router calls only when running against a lazy server:",
    );
    instructions = instructions.replace(
        "If the project involves PCB layout, also load:",
        "If the project involves PCB layout and the server is lazy, also load:",
    );
    let prelude = "Use Konnect MCP tools for every KiCad-source mutation. Never edit .kicad_sch, .kicad_pcb, .kicad_pro, .kicad_sym, .kicad_mod, fp-lib-table, or sym-lib-table as text. The companion normally exposes every tool schema eagerly, so call visible domain tools directly. Treat the supplied quality bars as engineering defaults and reconcile them with the user's requirements and component datasheets. Validate every requested artifact before completion, and report unsupported server capabilities explicitly.\n\n";
    let agent = CodexAgent {
        name,
        description,
        developer_instructions: format!("{prelude}{instructions}"),
    };
    Ok((file_name, toml::to_string_pretty(&agent)?))
}

fn parse_simple_frontmatter(raw: &str) -> BTreeMap<String, String> {
    raw.lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
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

fn source_fingerprint(
    assets: &SourceAssets,
    source_hooks: &[SourceHook],
    native_skills: &BTreeSet<String>,
    config: &[u8],
) -> Result<String> {
    let mut hasher = Sha256::new();
    for (name, dir) in &assets.skills {
        hasher.update(name.as_bytes());
        for (relative, path) in walk_files(dir)? {
            hasher.update(relative.to_string_lossy().as_bytes());
            hasher.update(fs::read(path)?);
        }
    }
    for path in &assets.agents {
        hasher.update(path.file_name().unwrap().to_string_lossy().as_bytes());
        hasher.update(fs::read(path)?);
    }
    for hook in source_hooks {
        hasher.update(hook.name.as_bytes());
        hasher.update(hook.event.as_bytes());
        hasher.update(hook.matcher.as_deref().unwrap_or("").as_bytes());
        hasher.update(hook.context.as_bytes());
    }
    for name in native_skills {
        hasher.update(b"native-skill\0");
        hasher.update(name.as_bytes());
    }
    hasher.update(config);
    hasher.update(OVERLAY_SKILL.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
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
        "the '{PLUGIN_NAME}' marketplace entry was removed or modified; use --force only if the companion entry may be discarded"
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
            "Konnect Codex companion is not installed (missing {})",
            path.display()
        )
    })?;
    let manifest: InstallManifest = serde_json::from_str(&raw)?;
    if manifest.schema_version != 1 {
        bail!(
            "unsupported companion manifest schema {}",
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
    fn skill_conversion_is_client_neutral() {
        let converted = transform_skill(
            "---\nname: test\nargument-hint: \"[task]\"\n---\nThis skill guides Claude through PCB work.\n",
        );
        assert!(converted.contains("This skill guides Codex through PCB work."));
        assert!(!converted.contains("argument-hint"));
    }

    #[test]
    fn agent_conversion_removes_claude_only_fields() {
        let source = r#"---
name: kicad-schematic-build-agent
description: "Builds circuits."
model: sonnet
tools:
  - mcp__konnect__*
maxTurns: 40
---

## Instructions

Load the required toolsets immediately:
`load_toolset("sch_components")`
"#;
        let (name, converted) = convert_agent(source).unwrap();
        assert_eq!(name, "konnect_schematic_builder.toml");
        assert!(converted.contains("name = \"konnect_schematic_builder\""));
        assert!(converted.contains("eagerly exposes all Konnect toolsets"));
        assert!(!converted.contains("sonnet"));
        assert!(!converted.contains("maxTurns"));
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
        assert!(user_prompt_context("Review this KiCad schematic").is_some());
        assert!(user_prompt_context("Refactor my web API").is_none());
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
        let source = temp.path().join("source");
        let skill_dir = source.join("skills").join("konnect");
        let agent_dir = source.join("agents");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::create_dir_all(&agent_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: konnect\ndescription: Test.\n---\nThis skill guides Claude through KiCad work.\n",
        )
        .unwrap();
        fs::write(
            agent_dir.join("kicad-schematic-build-agent.md"),
            "---\nname: kicad-schematic-build-agent\ndescription: \"Build test circuits.\"\nmodel: sonnet\ntools:\n  - mcp__konnect__*\n---\n\n## Instructions\nBuild the requested circuit.\n",
        )
        .unwrap();
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

        sync(SyncOptions {
            paths: paths.clone(),
            source: Some(source),
            konnect_binary: fake_konnect,
            config_source: Some(config),
            adapter_binary: adapter,
            activate: false,
            dry_run: false,
        })
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
        assert!(!paths
            .agents_dir
            .join("konnect_schematic_builder.toml")
            .exists());
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
            "0.5.1",
        )
        .unwrap();

        sync(SyncOptions {
            paths: paths.clone(),
            source: Some(temp.path().join("source")),
            konnect_binary: temp.path().join(if cfg!(windows) {
                "konnect.exe"
            } else {
                "konnect"
            }),
            config_source: Some(temp.path().join("konnect.toml")),
            adapter_binary: std::env::current_exe().unwrap(),
            activate: false,
            dry_run: false,
        })
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

        uninstall(&paths, false).unwrap();
        assert!(!paths.plugin_dir.exists());
        assert!(!paths.data_dir.exists());
        assert!(!paths.marketplace_path.exists());
        assert_eq!(fs::read_to_string(native_skill).unwrap(), "keep me");
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
