mod ai;
mod auth;
mod brand_tools;
mod dev_server;
mod editor_server;
mod jobs_cloud;
mod jobs_tools;
mod licenses_tools;
mod mcp_tools;
mod media;
mod plugin_tools;
mod publish_tools;
mod remote_assets;
mod storyboard_tools;
mod sync_cloud;
mod sync_tools;
mod telemetry_tools;
mod test_runner;
mod workspace_tools;

pub mod mcp;
pub mod receipt;
#[cfg(test)]
mod test_support;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sha2::Digest;

#[derive(Parser)]
#[command(
    name = "vidra",
    version,
    about = "Vidra — Programmable video infrastructure",
    long_about = "Vidra is a programmable, AI-native video infrastructure platform.\nDefine video in code, render with GPU acceleration, deploy anywhere.\n\nOne engine. Every interface. Any scale."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a VidraScript file to video
    Render {
        /// Path to the .vidra file to render
        #[arg()]
        file: PathBuf,

        /// Output file path (default: output/<name>.mp4 or output/<name>_<target>.mp4)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format: mp4, webm, gif, apng (auto-detected from extension if not set)
        #[arg(short, long)]
        format: Option<String>,

        /// Comma-separated list of target aspect ratios (e.g. 16:9,9:16,1:1)
        #[arg(long)]
        targets: Option<String>,

        /// Push render to the managed cloud cluster
        #[arg(long)]
        cloud: bool,

        /// Path to a CSV or JSON data file for batch template rendering
        #[arg(long)]
        data: Option<PathBuf>,

        /// Web capture backend: auto, platform, playwright (default: auto)
        #[arg(long, default_value = "auto")]
        web_backend: String,
    },

    /// Check a VidraScript file for errors (parse + type check)
    Check {
        /// Path to the .vidra file to check
        #[arg()]
        file: PathBuf,
    },

    /// Display version and engine info
    Info,

    /// Start the live preview dev server
    Dev {
        /// Path to the .vidra file to preview
        #[arg()]
        file: PathBuf,
    },

    /// Scaffold a new Vidra project
    Init {
        /// Name of the project directory to create
        #[arg()]
        name: String,

        /// Scaffold the project with a Starter Kit
        #[arg(long)]
        kit: Option<String>,
    },

    /// Format a VidraScript file
    Fmt {
        /// Path to the .vidra file to format
        #[arg()]
        file: PathBuf,

        /// Check mode: exit with error if input is not formatted (for CI)
        #[arg(long)]
        check: bool,
    },

    /// Run visual regression tests on a VidraScript file
    Test {
        /// Path to the .vidra file to test
        #[arg()]
        file: PathBuf,

        /// Update the snapshot baselines
        #[arg(long)]
        update: bool,
    },

    /// Start the Language Server
    Lsp,

    /// Start the Model Context Protocol Server (JSON-RPC)
    ///
    /// This command starts an MCP stdio server. Do not run this manually.
    /// Configure your AI assistant (e.g. Claude Desktop, Cursor) with:
    ///
    /// { "command": "bunx", "args": ["--bun", "@sansavision/vidra@latest", "mcp"] }
    ///
    /// Available tools for LLMs:
    /// - vidra-create_project
    /// - vidra-add_scene
    /// - vidra-render_preview
    /// - vidra-edit_layer
    Mcp,

    /// Inspect the render tree of a VidraScript file
    Inspect {
        /// Path to the .vidra file to inspect
        #[arg()]
        file: PathBuf,

        /// Frame to inspect and evaluate computed properties
        #[arg(long, short)]
        frame: Option<u64>,
    },

    /// Benchmark performance across resolutions
    Bench {
        /// Path to the .vidra file to benchmark
        #[arg()]
        file: PathBuf,

        /// Save latest benchmark against baseline for regression test
        #[arg(long)]
        update: bool,
    },

    /// Add a pre-built template to your project
    Add {
        /// Name of the template to add (e.g. 'social-post', 'lower-third', 'branded-intro')
        template: String,
    },

    /// Authentication and API key management
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    /// Telemetry configuration and management
    Telemetry {
        #[command(subcommand)]
        command: TelemetryCommands,
    },

    /// Generate a visual storyboard from a text prompt
    Storyboard {
        /// Text prompt to generate the storyboard
        #[arg()]
        prompt: String,

        /// Output path for the generated grid (defaults to storyboard.png)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate a shareable link for a video file
    Share {
        /// File to share, defaults to latest render
        #[arg()]
        file: Option<PathBuf>,
    },

    /// Manage team workspaces
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },

    /// Manage Vidra brand kits
    Brand {
        #[command(subcommand)]
        command: BrandCommands,
    },

    /// Render locally and upload as a cloud preview
    Preview {
        /// Path to the .vidra file to render and preview
        #[arg()]
        file: PathBuf,

        /// Upload and generate a shareable link
        #[arg(long)]
        share: bool,
    },

    /// Sync project metadata and assets with Vidra Cloud
    Sync {
        #[command(subcommand)]
        command: SyncCommands,
    },

    /// Cloud render jobs management
    Jobs {
        #[command(subcommand)]
        command: JobsCommands,
    },

    /// Upload files to cloud project storage
    Upload {
        /// Path to the file or directory
        #[arg()]
        path: PathBuf,
    },

    /// Manage cloud-stored assets
    Assets {
        #[command(subcommand)]
        command: AssetsCommands,
    },

    /// Search Vidra Commons for resources
    Search {
        /// Search query terms
        #[arg()]
        query: String,
    },

    /// Browse trending resources in Vidra Commons
    Explore,

    /// View licenses for project assets
    Licenses,

    /// Publish a resource to Vidra Commons
    Publish {
        /// Path to the resource package to publish
        #[arg()]
        path: PathBuf,
    },

    /// Manage engine plugins
    Plugins {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// Open the render observability dashboard
    Dashboard,

    /// Run an environment health check
    Doctor,

    /// Launch the visual editor for a Vidra project
    Editor {
        /// Path to the .vidra file to edit (default: main.vidra)
        #[arg(default_value = "main.vidra")]
        file: PathBuf,

        /// Port for the editor server (default: 3001)
        #[arg(long, default_value_t = 3001)]
        port: u16,

        /// Auto-open the editor in the default browser
        #[arg(long)]
        open: bool,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Login and obtain a Vidra License Token (VLT)
    Login,
    /// Create a scoped API key
    CreateKey {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        scope: String,
    },
    /// List active API keys
    ListKeys,
    /// Revoke an API key
    RevokeKey {
        #[arg()]
        key_id: String,
    },
}

#[derive(Subcommand)]
enum TelemetryCommands {
    /// Display current telemetry settings and data
    Show,
    /// Change telemetry tier (anonymous, identified, diagnostics, off)
    Set {
        #[arg()]
        tier: String,
    },
    /// Export all collected telemetry data
    Export,
    /// Request deletion of all telemetry data
    Delete,
}

#[derive(Subcommand)]
enum SyncCommands {
    /// Push local changes to cloud
    Push,
    /// Pull remote changes from cloud
    Pull,
    /// Show current sync status
    Status,
    /// Smart asset hydration (download missing assets on demand)
    Assets,
}

#[derive(Subcommand)]
enum JobsCommands {
    /// List pending render jobs from cloud
    List,
    /// Pull, render locally, upload result
    Run,
    /// Like run but for all pending jobs
    RunAll,
    /// Daemon mode (continuous poll and execute)
    Watch,
}

#[derive(Subcommand)]
enum AssetsCommands {
    /// List available assets in the cloud
    List,
    /// Pull an asset from cloud storage
    Pull {
        /// The name or ID of the asset
        #[arg()]
        name: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List installed plugins
    List,
    /// Install a plugin from registry
    Install {
        #[arg()]
        name: String,
    },
    /// Remove a plugin
    Remove {
        #[arg()]
        name: String,
    },
    /// Show info about a plugin
    Info {
        #[arg()]
        name: String,
    },
}

#[derive(Subcommand)]
enum BrandCommands {
    /// Create a new brand kit
    Create {
        #[arg()]
        name: String,
    },
    /// List available brand kits
    List,
    /// Apply a brand kit to the current project
    Apply {
        #[arg()]
        name: String,
    },
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// Create a new team workspace
    Create {
        #[arg()]
        name: String,
    },
    /// List your workspaces
    List,
    /// Switch active workspace
    Switch {
        #[arg()]
        name: String,
    },
    /// Invite a member to the workspace
    Invite {
        #[arg()]
        email: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let is_mcp_or_lsp = matches!(cli.command, Commands::Mcp | Commands::Lsp);

    let subscriber = tracing_subscriber::fmt().with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    );

    if is_mcp_or_lsp {
        // MCP/LSP: write logs to stderr with no ANSI colors to avoid polluting
        // the JSON-RPC stream on stdout.
        subscriber
            .with_ansi(false)
            .with_writer(std::io::stderr)
            .init();
    } else {
        subscriber.init();
    }

    match cli.command {
        Commands::Render {
            file,
            output,
            format,
            targets,
            cloud,
            data,
            web_backend,
        } => {
            std::env::set_var("VIDRA_WEB_BACKEND", &web_backend);

            // If using platform webview on macOS, run the render on a
            // background thread while the main thread pumps the RunLoop.
            // This is required because WKWebView must run on the main thread.
            #[cfg(target_os = "macos")]
            if web_backend == "platform" || (web_backend == "auto" && cfg!(target_os = "macos")) {
                use std::sync::mpsc;
                let (tx, rx) = mpsc::channel::<Result<()>>();

                std::thread::spawn(move || {
                    let result = cmd_render(file, output, format, targets, cloud, data);
                    let _ = tx.send(result);
                });

                // Pump the main thread RunLoop until the render finishes
                loop {
                    match rx.try_recv() {
                        Ok(result) => return result,
                        Err(mpsc::TryRecvError::Empty) => {
                            vidra_web::platform::pump_main_runloop_once();
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            return Err(anyhow::anyhow!("Render thread panicked"));
                        }
                    }
                }
            }

            #[allow(unreachable_code)]
            cmd_render(file, output, format, targets, cloud, data)
        }
        Commands::Check { file } => cmd_check(file),
        Commands::Fmt { file, check } => cmd_fmt(file, check),
        Commands::Test { file, update } => test_runner::run_test(file, update),
        Commands::Bench { file, update } => bench_runner::run_benchmark(file, update),
        Commands::Add { template } => template_manager::execute_add(&template),
        Commands::Info => cmd_info(),
        Commands::Init { name, kit } => cmd_init(&name, kit),
        Commands::Dev { file } => run_async(dev_server::run_dev_server(file)),
        Commands::Lsp => run_async(async {
            vidra_lsp::start_lsp().await;
            Ok(())
        }),
        Commands::Mcp => {
            // Redirect stdout → stderr so that any println! from sub-tools
            // (e.g. cmd_render, cmd_preview) goes to stderr instead of
            // contaminating the JSON-RPC stream.  The saved fd is the
            // *real* stdout that run_mcp_server will use for JSON-RPC.
            let saved_stdout = mcp::redirect_stdout_to_stderr();
            run_async(async move {
                mcp::run_mcp_server(saved_stdout).await?;
                Ok(())
            })
        }
        Commands::Inspect { file, frame } => cmd_inspect(file, frame),
        Commands::Auth { command } => match command {
            AuthCommands::Login => auth::login(),
            AuthCommands::CreateKey { name, scope } => auth::create_api_key(&name, &scope),
            AuthCommands::ListKeys => auth::list_api_keys(),
            AuthCommands::RevokeKey { key_id } => auth::revoke_api_key(&key_id),
        },
        Commands::Telemetry { command } => match command {
            TelemetryCommands::Show => telemetry_show(),
            TelemetryCommands::Set { tier } => telemetry_set(&tier),
            TelemetryCommands::Export => telemetry_export(),
            TelemetryCommands::Delete => telemetry_delete(),
        },
        Commands::Doctor => cmd_doctor(),
        Commands::Storyboard { prompt, output } => cmd_storyboard(&prompt, output),
        Commands::Share { file } => cmd_share(file),
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Create { name } => cmd_workspace_create(&name),
            WorkspaceCommands::List => cmd_workspace_list(),
            WorkspaceCommands::Switch { name } => cmd_workspace_switch(&name),
            WorkspaceCommands::Invite { email } => cmd_workspace_invite(&email),
        },
        Commands::Brand { command } => match command {
            BrandCommands::Create { name } => cmd_brand_create(&name),
            BrandCommands::List => cmd_brand_list(),
            BrandCommands::Apply { name } => cmd_brand_apply(&name),
        },
        Commands::Preview { file, share } => cmd_preview(file, share),
        Commands::Sync { command } => match command {
            SyncCommands::Push => cmd_sync_push(),
            SyncCommands::Pull => cmd_sync_pull(),
            SyncCommands::Status => cmd_sync_status(),
            SyncCommands::Assets => cmd_sync_assets(),
        },
        Commands::Jobs { command } => match command {
            JobsCommands::List => cmd_jobs_list(),
            JobsCommands::Run => cmd_jobs_run(false),
            JobsCommands::RunAll => cmd_jobs_run(true),
            JobsCommands::Watch => cmd_jobs_watch(),
        },
        Commands::Upload { path } => cmd_upload(&path),
        Commands::Assets { command } => match command {
            AssetsCommands::List => cmd_assets_list(),
            AssetsCommands::Pull { name } => cmd_assets_pull(&name),
        },
        Commands::Search { query } => cmd_search(&query),
        Commands::Explore => cmd_explore(),
        Commands::Licenses => cmd_licenses(),
        Commands::Publish { path } => cmd_publish(&path),
        Commands::Plugins { command } => match command {
            PluginCommands::List => cmd_plugin_list(),
            PluginCommands::Install { name } => cmd_plugin_install(&name),
            PluginCommands::Remove { name } => cmd_plugin_remove(&name),
            PluginCommands::Info { name } => cmd_plugin_info(&name),
        },
        Commands::Dashboard => cmd_dashboard(),
        Commands::Editor { file, port, open } => {
            run_async(editor_server::run_editor_server(file, port, open))
        }
    }
}

fn run_async<F>(future: F) -> Result<()>
where
    F: std::future::Future<Output = Result<()>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize async runtime")?;
    runtime.block_on(future)
}

fn cmd_share(file: Option<PathBuf>) -> Result<()> {
    let target = file.unwrap_or_else(|| PathBuf::from("output/output.mp4"));
    if !target.exists() {
        anyhow::bail!("file not found: {}", target.display());
    }

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let enqueued = sync_tools::enqueue_upload_path(&project_root, &target)?;

    let hash = sha256_file_prefixed(&target)?;
    let share_id = hash.trim_start_matches("sha256:");
    let base_link = if let Some(base) = sync_cloud::cloud_base_url_from_env() {
        format!("{}/share/{}", base, share_id)
    } else {
        format!("vidra://share/{}", share_id)
    };

    println!("🔗 Shareable link for {}", target.display());
    if enqueued > 0 {
        println!("   ✓ Queued for upload: {} file(s)", enqueued);
    } else {
        println!("   ✓ Already queued (or previously uploaded)");
    }

    if let Some(base) = sync_cloud::cloud_base_url_from_env() {
        let uploaded = sync_cloud::push_uploads_to_cloud(&project_root, &base)?;
        if uploaded.uploaded > 0 {
            println!("   ✓ Uploaded: {} file(s)", uploaded.uploaded);
        }
        if !uploaded.failures.is_empty() {
            println!("   ⚠️  Upload failures ({}):", uploaded.failures.len());
            for f in &uploaded.failures {
                println!("      - {}", f);
            }
            anyhow::bail!("share upload completed with failures");
        }
        println!("   Your link: {}", base_link);
        return Ok(());
    }

    println!("   Your link: {}", base_link);
    println!("   (Set VIDRA_CLOUD_URL then run `vidra sync push` to publish)");
    Ok(())
}

fn cmd_preview(file: PathBuf, share: bool) -> Result<()> {
    println!("🎬 Generating local preview for {}...", file.display());
    let out = std::path::PathBuf::from("output").join("preview.mp4");
    cmd_render(
        file,
        Some(out.clone()),
        Some("mp4".to_string()),
        None,
        false,
        None,
    )?;

    if share {
        println!("🔗 Sharing preview...");
        cmd_share(Some(out))?;
    } else {
        println!("   Preview ready at {}", out.display());
    }

    Ok(())
}

fn telemetry_show() -> Result<()> {
    let cfg = telemetry_tools::load_telemetry_config()?;
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let snapshot = telemetry_tools::compute_snapshot(&cwd)?;

    println!("📊 Vidra Telemetry Configuration");
    println!("   Current Tier: {}", cfg.tier.as_str());
    println!("   Updated: {}", cfg.updated_at.to_rfc3339());
    println!(
        "   Pending Upload: {} render receipt(s), {} upload(s)",
        snapshot.receipts_queued, snapshot.uploads_queued
    );
    Ok(())
}

fn telemetry_set(tier: &str) -> Result<()> {
    let t = telemetry_tools::parse_tier(tier)?;
    let cfg = telemetry_tools::save_telemetry_config(t)?;
    println!(
        "✅ Telemetry tier successfully set to: {}",
        cfg.tier.as_str()
    );
    Ok(())
}

fn telemetry_export() -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let out = std::path::PathBuf::from("vidra_telemetry_export.zip");
    telemetry_tools::export_telemetry_zip(&cwd, &out)?;
    println!("📦 Exported telemetry data to {}", out.display());
    Ok(())
}

fn telemetry_delete() -> Result<()> {
    telemetry_tools::delete_local_telemetry_data()?;
    println!("🗑️  Local telemetry config deleted.");
    println!(
        "   (Note: receipts/uploads queues are retained; use `vidra sync` to manage sync state.)"
    );
    Ok(())
}

fn cmd_doctor() -> Result<()> {
    use std::path::Path;

    let mut warnings: Vec<String> = Vec::new();

    println!("🩺 Vidra Doctor");
    println!("   CLI version: {}", env!("CARGO_PKG_VERSION"));
    println!(
        "   OS: {} ({})",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    println!("   CWD: {}", cwd.display());

    let has_toml = Path::new("vidra.config.toml").exists();
    let has_json = Path::new("vidra.config.json").exists();
    let has_config = has_toml || has_json;
    println!(
        "   Project config: {}",
        if has_config {
            if has_toml {
                "vidra.config.toml"
            } else {
                "vidra.config.json"
            }
        } else {
            "(none found)"
        }
    );
    if !has_config {
        warnings.push("no vidra.config.* in current directory".to_string());
    }

    if let Some(url) = sync_cloud::cloud_base_url_from_env() {
        println!("   Cloud endpoint: {}", url);
    } else {
        println!("   Cloud endpoint: (unset) — local-first mode");
    }

    // Basic writeability checks for common directories.
    if let Err(e) = std::fs::create_dir_all("output") {
        warnings.push(format!("failed to create output/: {}", e));
    }
    if let Err(e) = std::fs::create_dir_all(".vidra") {
        warnings.push(format!("failed to create .vidra/: {}", e));
    } else {
        let probe = std::path::PathBuf::from(".vidra").join("doctor_write_test.tmp");
        match std::fs::write(&probe, b"ok") {
            Ok(()) => {
                let _ = std::fs::remove_file(&probe);
            }
            Err(e) => warnings.push(format!(".vidra not writable: {}", e)),
        }
    }

    // Receipt signing key (used for render receipts).
    match crate::receipt::load_or_create_device_signing_key() {
        Ok(_) => println!("   Receipt signing key: OK"),
        Err(e) => warnings.push(format!("receipt signing key unavailable: {:#}", e)),
    }

    // VLT token (plan + offline validation).
    match crate::auth::Vlt::load_local().and_then(|v| v.validate_offline().map(|_| v)) {
        Ok(vlt) => println!(
            "   VLT: {} (plan: {})",
            vlt.payload.vlt_id, vlt.payload.plan
        ),
        Err(e) => warnings.push(format!("VLT: {:#}", e)),
    }

    // Receipts queue status.
    if let Some(dir) = sync_tools::receipts_root_dir() {
        match sync_tools::receipt_sync_status(&dir) {
            Ok(s) => println!("   Render receipts: {} queued, {} sent", s.queued, s.sent),
            Err(e) => warnings.push(format!("failed to read receipts queue: {:#}", e)),
        }
    } else {
        warnings.push("failed to resolve receipts directory (~/.vidra/receipts)".to_string());
    }

    // Upload queue status (project-local).
    match sync_tools::upload_sync_status(&cwd) {
        Ok(s) => println!("   Upload queue: {} queued, {} sent", s.queued, s.sent),
        Err(e) => warnings.push(format!("failed to read upload queue: {:#}", e)),
    }

    // Asset manifest status.
    let manifest_path = std::path::PathBuf::from(".vidra").join("asset_manifest.json");
    if manifest_path.exists() {
        match sync_tools::read_asset_manifest(&manifest_path) {
            Ok(m) => {
                let stats = sync_tools::asset_manifest_stats_from_manifest(&m);
                println!(
                    "   Asset manifest: {} assets ({} missing, {} remote-url) — {}",
                    stats.total,
                    stats.missing,
                    stats.remote_urls,
                    m.generated_at.to_rfc3339()
                );
            }
            Err(e) => warnings.push(format!(
                "asset manifest unreadable ({}): {:#}",
                manifest_path.display(),
                e
            )),
        }
    } else {
        println!("   Asset manifest: (none) — run `vidra sync assets`");
    }

    // Jobs queue status (local-first).
    if let Some(jobs_root) = jobs_tools::jobs_root_dir() {
        match jobs_tools::list_queued_jobs(&jobs_root) {
            Ok(j) => println!("   Jobs queue: {} queued", j.len()),
            Err(e) => warnings.push(format!("failed to read jobs queue: {:#}", e)),
        }
    }

    if warnings.is_empty() {
        println!("   Status: OK");
        return Ok(());
    }

    println!("   Status: WARN ({} issue(s))", warnings.len());
    for w in &warnings {
        println!("   - {}", w);
    }
    Ok(())
}

fn cmd_render(
    file: PathBuf,
    output: Option<PathBuf>,
    format: Option<String>,
    targets: Option<String>,
    cloud: bool,
    data: Option<PathBuf>,
) -> Result<()> {
    let start = Instant::now();

    // Best-effort config load (rendering a standalone file without a project folder is allowed).
    let config = vidra_core::VidraConfig::load_from_file(std::path::Path::new("vidra.config.toml"))
        .unwrap_or_default();

    if cloud {
        let Some(jobs_root) = jobs_tools::jobs_root_dir() else {
            anyhow::bail!("failed to resolve ~/.vidra/jobs");
        };
        jobs_tools::ensure_jobs_dirs(&jobs_root)?;

        let project_root =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let job_id = {
            let ts = chrono::Utc::now().to_rfc3339();
            let mut hasher = sha2::Sha256::new();
            hasher.update(project_root.to_string_lossy().as_bytes());
            hasher.update(file.to_string_lossy().as_bytes());
            hasher.update(ts.as_bytes());
            let digest = hasher.finalize();
            let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
            format!("job_{}", &hex[..10])
        };

        let spec = jobs_tools::JobSpec {
            job_id: job_id.clone(),
            project_root: project_root.clone(),
            vidra_file: file.clone(),
            output,
            format,
            targets,
            data,
            created_at: chrono::Utc::now(),
        };

        let _path = jobs_tools::write_job_to_dir(&jobs_tools::jobs_queued_dir(&jobs_root), &spec)?;
        println!("☁️  Enqueued render job (local queue)");
        println!("   Job ID: {}", job_id);
        println!("   Project: {}", project_root.display());
        println!("   File: {}", file.display());
        println!("   Next: run `vidra jobs run` (or `vidra jobs watch`)");
        return Ok(());
    }

    // Read source file
    let source = std::fs::read_to_string(&file)
        .with_context(|| format!("failed to read file: {}", file.display()))?;

    // If --data is set, do batch rendering
    if let Some(ref data_path) = data {
        println!("📊 Vidra Batch Render (data-driven)");
        println!("   Source:    {}", file.display());
        println!("   Data:      {}", data_path.display());

        let dataset =
            vidra_ir::data::DataSet::load(data_path).map_err(|e| anyhow::anyhow!("{}", e))?;

        println!("   Rows:      {}", dataset.rows.len());
        println!("   Columns:   {:?}", dataset.columns);
        println!();

        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        let out_ext_str = format.as_deref().unwrap_or("mp4");
        let out_dir = output
            .as_ref()
            .and_then(|o| o.parent())
            .unwrap_or(std::path::Path::new("output"));
        std::fs::create_dir_all(out_dir)?;

        for (row_idx, row) in dataset.rows.iter().enumerate() {
            let row_source = vidra_ir::data::interpolate(&source, row);
            let row_output = out_dir.join(format!("{}_{}.{}", stem, row_idx + 1, out_ext_str));

            // Write interpolated source to a temp file, then render it
            let tmp_file =
                std::env::temp_dir().join(format!("vidra_batch_{}_{}.vidra", stem, row_idx));
            std::fs::write(&tmp_file, &row_source)?;

            println!("   ── Row {} ──", row_idx + 1);
            match cmd_render(
                tmp_file.clone(),
                Some(row_output.clone()),
                format.clone(),
                targets.clone(),
                false,
                None,
            ) {
                Ok(_) => println!("      ✓ Row {} → {}", row_idx + 1, row_output.display()),
                Err(e) => println!("      ✗ Row {} failed: {}", row_idx + 1, e),
            }
            let _ = std::fs::remove_file(&tmp_file);
        }

        let total = start.elapsed();
        println!();
        println!(
            "   ⚡ Batch complete: {} rows in {:.2}s",
            dataset.rows.len(),
            total.as_secs_f64()
        );
        return Ok(());
    }

    let file_name = file.file_name().unwrap_or_default().to_string_lossy();

    println!("🎬 Vidra Render Engine v{}", env!("CARGO_PKG_VERSION"));
    println!("   Source: {}", file.display());

    let is_json = file.extension().map_or(false, |e| e == "json");

    let mut parse_time_secs = 0.0;
    let mut type_time_secs = 0.0;
    let mut compile_time_secs = 0.0;

    let (mut ast, base_ir) = if is_json {
        println!("   ✓ Detected IR JSON");
        let proj: vidra_ir::Project = serde_json::from_str(&source)
            .with_context(|| format!("failed to parse IR JSON: {}", file.display()))?;
        (None, Some(proj))
    } else {
        // Phase 1: Parse
        let parse_start = Instant::now();
        let ast = parse_and_resolve_imports(&file)?;
        let parse_time = parse_start.elapsed();
        parse_time_secs = parse_time.as_secs_f64();
        println!("   ✓ Parsed in {:.1}ms", parse_time_secs * 1000.0);

        // Phase 2: Type Check & Lint
        let type_start = Instant::now();
        let checker = vidra_lang::TypeChecker::new(file_name.clone());
        let diagnostics = match checker.check(&ast) {
            Ok(diags) => diags,
            Err(diags) => {
                for d in &diags {
                    if d.severity != vidra_lang::checker::DiagnosticSeverity::Error {
                        println!("   ⚠️ {}", d);
                    }
                }
                let msgs: Vec<String> = diags
                    .into_iter()
                    .filter(|d| d.severity == vidra_lang::checker::DiagnosticSeverity::Error)
                    .map(|e| e.to_string())
                    .collect();
                anyhow::bail!("Type errors:\n  {}", msgs.join("\n  "));
            }
        };
        for diag in diagnostics {
            let icon = match diag.severity {
                vidra_lang::checker::DiagnosticSeverity::Warning => "⚠️",
                vidra_lang::checker::DiagnosticSeverity::Info => "ℹ️",
                _ => "❌",
            };
            println!("   {} {}", icon, diag);
        }
        let type_time = type_start.elapsed();
        type_time_secs = type_time.as_secs_f64();
        println!(
            "   ✓ Type checked and linted in {:.1}ms",
            type_time_secs * 1000.0
        );

        (Some(ast), None)
    };

    let base_minor = if let Some(a) = &ast {
        a.width.min(a.height)
    } else {
        base_ir
            .as_ref()
            .unwrap()
            .settings
            .width
            .min(base_ir.as_ref().unwrap().settings.height)
    };

    let target_list = if let Some(t_str) = targets {
        t_str
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        vec!["default".to_string()]
    };

    let multiple_targets = target_list.len() > 1;

    for target in target_list {
        let mut project = if let Some(ref mut a) = ast {
            if target != "default" {
                if let Some((w_str, h_str)) = target.split_once(':') {
                    if let (Ok(x), Ok(y)) = (w_str.parse::<f64>(), h_str.parse::<f64>()) {
                        if x > y {
                            a.height = base_minor;
                            a.width = (base_minor as f64 * (x / y)).round() as u32;
                        } else if y > x {
                            a.width = base_minor;
                            a.height = (base_minor as f64 * (y / x)).round() as u32;
                        } else {
                            a.width = base_minor;
                            a.height = base_minor;
                        }
                        println!(
                            "\n▶ Generating target: {} ({}x{})",
                            target, a.width, a.height
                        );
                    } else {
                        println!("   ⚠️ Invalid target format: {}", target);
                        continue;
                    }
                }
            }
            let compile_start = Instant::now();
            let p = vidra_lang::Compiler::compile(&a).map_err(|e| anyhow::anyhow!("{}", e))?;
            let compile_time = compile_start.elapsed();
            compile_time_secs = compile_time.as_secs_f64();
            println!("   ✓ Compiled to IR in {:.1}ms", compile_time_secs * 1000.0);
            p
        } else {
            let mut ir = base_ir.clone().unwrap();
            if target != "default" {
                if let Some((w_str, h_str)) = target.split_once(':') {
                    if let (Ok(x), Ok(y)) = (w_str.parse::<f64>(), h_str.parse::<f64>()) {
                        if x > y {
                            ir.settings.height = base_minor;
                            ir.settings.width = (base_minor as f64 * (x / y)).round() as u32;
                        } else if y > x {
                            ir.settings.width = base_minor;
                            ir.settings.height = (base_minor as f64 * (y / x)).round() as u32;
                        } else {
                            ir.settings.width = base_minor;
                            ir.settings.height = base_minor;
                        }
                        println!(
                            "\n▶ Generating target: {} ({}x{})",
                            target, ir.settings.width, ir.settings.height
                        );
                    } else {
                        println!("   ⚠️ Invalid target format: {}", target);
                        continue;
                    }
                }
            }
            ir
        };
        println!(
            "   ├ {}x{} @ {}fps",
            project.settings.width, project.settings.height, project.settings.fps
        );
        println!(
            "   ├ {} scene(s), {:.1}s total",
            project.scenes.len(),
            project.total_duration().as_seconds()
        );
        println!("   └ {} total frames", project.total_frames());

        // Resolve relative asset paths against the .vidra file's directory so that
        // assets referenced as e.g. "music.mp3" work regardless of the CWD.
        let source_dir = file.parent().unwrap_or(std::path::Path::new("."));
        for asset in project.assets.all_mut() {
            if asset.path.is_relative() && !asset.path.exists() {
                let resolved = source_dir.join(&asset.path);
                if resolved.exists() {
                    asset.path = resolved;
                }
            }
        }

        // Phase 3: Validate
        vidra_ir::validate::validate_project(&project).map_err(|errors| {
            let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
            anyhow::anyhow!("IR validation errors:\n  {}", msgs.join("\n  "))
        })?;
        println!("   ✓ IR validated");

        // Phase 4: Remote asset fetching + caching (http/https -> local cache paths)
        let remote_report = remote_assets::prepare_project_remote_assets(&mut project, &config)?;
        if remote_report.downloaded > 0 || remote_report.reused_from_cache > 0 {
            println!(
                "   ✓ Remote assets: {} downloaded, {} cached",
                remote_report.downloaded, remote_report.reused_from_cache
            );
        }

        // Phase 3.25: Media materialization (waveforms, etc)
        let media_report = media::prepare_project_media(&mut project, &config)?;
        if media_report.waveforms_materialized > 0 {
            println!(
                "   ✓ Media materialized: {} waveform(s)",
                media_report.waveforms_materialized
            );
        }

        // Phase 3.5: AI materialization (TTS/captions/etc) — gated by config.ai.enabled
        let ai_report = ai::prepare_project_ai(&mut project, &config)?;
        if ai_report.tts_layers_materialized > 0 {
            println!(
                "   ✓ AI materialized: {} TTS layer(s)",
                ai_report.tts_layers_materialized
            );
        }
        if ai_report.autocaption_layers_materialized > 0 {
            println!(
                "   ✓ AI materialized: {} AutoCaption layer(s)",
                ai_report.autocaption_layers_materialized
            );
        }
        if ai_report.bg_removals_materialized > 0 {
            println!(
                "   ✓ AI materialized: {} background removal(s)",
                ai_report.bg_removals_materialized
            );
        }

        // Hash the final IR (after all materialization) for render receipts.
        let ir_hash = {
            let bytes = serde_json::to_vec(&project).unwrap_or_default();
            let mut hasher = sha2::Sha256::new();
            hasher.update(&bytes);
            let digest = hasher.finalize();
            let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
            format!("sha256:{}", hex)
        };

        // Phase 4: Render
        let render_start = Instant::now();
        let result =
            vidra_render::RenderPipeline::render(&project).map_err(|e| anyhow::anyhow!("{}", e))?;
        let render_time = render_start.elapsed();
        let render_fps = result.frame_count as f64 / render_time.as_secs_f64();
        println!(
            "   ✓ Rendered {} frames in {:.1}ms ({:.0} fps)",
            result.frame_count,
            render_time.as_secs_f64() * 1000.0,
            render_fps
        );

        // Phase 5: Encode — detect output format
        // Determine the output format: explicit --format flag > file extension > default mp4
        let out_ext = format.as_deref().unwrap_or_else(|| {
            output
                .as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .unwrap_or("mp4")
        });
        let out_ext = match out_ext {
            "webm" => "webm",
            "gif" => "gif",
            "apng" | "png" => "apng",
            _ => "mp4",
        };

        let output_path = if multiple_targets || target != "default" {
            let stem = file.file_stem().unwrap_or_default().to_string_lossy();
            let suffix = target.replace(':', "x");
            let mut p = if let Some(ref o) = output {
                o.clone()
            } else {
                PathBuf::from("output")
            };

            if output.is_some() && output.as_ref().unwrap().is_file() {
                let mut dir = p.parent().unwrap_or(std::path::Path::new("")).to_path_buf();
                let f_stem = p.file_stem().unwrap_or_default().to_string_lossy();
                dir.push(format!("{}_{}.{}", f_stem, suffix, out_ext));
                dir
            } else {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
                p.push(format!("{}_{}.{}", stem, suffix, out_ext));
                p
            }
        } else {
            output.clone().unwrap_or_else(|| {
                let stem = file.file_stem().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("output/{}.{}", stem, out_ext))
            })
        };

        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let encode_start = Instant::now();
        let audio_tracks = extract_audio_tracks(&project);

        match out_ext {
            "webm" => {
                vidra_encode::WebmEncoder::encode(
                    &result.frames,
                    &audio_tracks,
                    result.width,
                    result.height,
                    result.fps,
                    &output_path,
                    None,
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            "gif" => {
                vidra_encode::GifEncoder::encode(
                    &result.frames,
                    result.width,
                    result.height,
                    result.fps,
                    &output_path,
                    None,
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            "apng" => {
                vidra_encode::ApngEncoder::encode(
                    &result.frames,
                    result.width,
                    result.height,
                    result.fps,
                    &output_path,
                    None,
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            _ => {
                vidra_encode::FfmpegEncoder::encode(
                    &result.frames,
                    &audio_tracks,
                    result.width,
                    result.height,
                    result.fps,
                    &output_path,
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            }
        }

        let encode_time = encode_start.elapsed();
        println!(
            "   ✓ Encoded to {} ({}) in {:.1}ms",
            output_path.display(),
            out_ext.to_uppercase(),
            encode_time.as_secs_f64() * 1000.0
        );

        let total_time = start.elapsed();
        println!();
        println!(
            "   ⚡ Total: {:.2}s (parse: {:.0}ms → typeck: {:.0}ms → compile: {:.0}ms → render: {:.0}ms → encode: {:.0}ms)",
            total_time.as_secs_f64(),
            parse_time_secs * 1000.0,
            type_time_secs * 1000.0,
            compile_time_secs * 1000.0,
            render_time.as_secs_f64() * 1000.0,
            encode_time.as_secs_f64() * 1000.0,
        );
        println!("   📦 Output: {}", output_path.display());

        // Generate Render Receipt (PRD 8.5)
        if let Some(receipts_dir) = crate::receipt::receipts_dir() {
            let signing_key = crate::receipt::load_or_create_device_signing_key();

            let output_hash =
                sha256_file_prefixed(&output_path).unwrap_or_else(|_| "sha256:".to_string());
            let vlt_id = match crate::auth::Vlt::load_local()
                .and_then(|v| v.validate_offline().map(|_| v))
            {
                Ok(vlt) => vlt.payload.vlt_id,
                Err(_) => "vlt_missing".to_string(),
            };

            if let Ok(signing_key) = signing_key {
                let output_format = format!(
                    "{}_{}x{}_{}fps",
                    out_ext, project.settings.width, project.settings.height, project.settings.fps
                );

                if let Ok(receipt) = crate::receipt::RenderReceipt::new(
                    project.id.clone(),
                    ir_hash.clone(),
                    output_hash,
                    output_format,
                    render_time.as_millis() as u64,
                    result.frame_count as u64,
                    crate::receipt::HardwareInfo::basic(),
                    vlt_id,
                    &signing_key,
                ) {
                    if receipt.save_to_dir(&receipts_dir).is_ok() {
                        println!("   🧾 Generated Render Receipt (signed)");
                    }
                }
            }
        }
    }

    Ok(())
}

fn extract_audio_tracks(project: &vidra_ir::Project) -> Vec<vidra_encode::ffmpeg::AudioTrack> {
    let mut tracks = Vec::new();
    let mut current_time = 0.0;

    for scene in &project.scenes {
        for layer in &scene.layers {
            extract_layer_audio(layer, project, current_time, &mut tracks);
        }
        current_time += scene.duration.as_seconds();
    }
    tracks
}

fn extract_layer_audio(
    layer: &vidra_ir::layer::Layer,
    project: &vidra_ir::Project,
    time_offset: f64,
    tracks: &mut Vec<vidra_encode::ffmpeg::AudioTrack>,
) {
    match &layer.content {
        vidra_ir::layer::LayerContent::Audio {
            asset_id,
            trim_start,
            trim_end,
            volume,
            role,
            duck,
        } => {
            if let Some(asset) = project.assets.get(asset_id) {
                tracks.push(vidra_encode::ffmpeg::AudioTrack {
                    path: std::path::PathBuf::from(&asset.path),
                    trim_start: time_offset + trim_start.as_seconds(),
                    trim_end: trim_end.map(|d| d.as_seconds()),
                    volume: *volume,
                    role: role.clone(),
                    duck: *duck,
                });
            }
        }
        vidra_ir::layer::LayerContent::TTS {
            volume,
            audio_asset_id: Some(asset_id),
            ..
        } => {
            if let Some(asset) = project.assets.get(asset_id) {
                tracks.push(vidra_encode::ffmpeg::AudioTrack {
                    path: std::path::PathBuf::from(&asset.path),
                    trim_start: time_offset,
                    trim_end: None,
                    volume: *volume,
                    role: Some("narration".to_string()),
                    duck: None,
                });
            } else {
                tracing::warn!("TTS layer references missing audio asset_id: {}", asset_id);
            }
        }
        _ => {}
    }
    for child in &layer.children {
        extract_layer_audio(child, project, time_offset, tracks);
    }
}

fn cmd_check(file: PathBuf) -> Result<()> {
    let _source = std::fs::read_to_string(&file)
        .with_context(|| format!("failed to read file: {}", file.display()))?;

    let _file_name = file.file_name().unwrap_or_default().to_string_lossy();

    println!("🔍 Checking {}", file.display());

    // Parse
    let ast = parse_and_resolve_imports(&file)?;
    println!("   ✓ Parse OK");

    // Type Check
    let checker =
        vidra_lang::TypeChecker::new(file.file_name().unwrap().to_string_lossy().to_string());
    match checker.check(&ast) {
        Ok(diags) => {
            for diag in diags {
                let icon = match diag.severity {
                    vidra_lang::checker::DiagnosticSeverity::Warning => "⚠️",
                    vidra_lang::checker::DiagnosticSeverity::Info => "ℹ️",
                    _ => "❌",
                };
                println!("   {} {}", icon, diag);
            }
        }
        Err(diags) => {
            for d in &diags {
                if d.severity != vidra_lang::checker::DiagnosticSeverity::Error {
                    println!("   ⚠️ {}", d);
                }
            }
            let msgs: Vec<String> = diags
                .into_iter()
                .filter(|d| d.severity == vidra_lang::checker::DiagnosticSeverity::Error)
                .map(|e| e.to_string())
                .collect();
            anyhow::bail!("Type checking failed:\n  {}", msgs.join("\n  "));
        }
    }
    println!("   ✓ Type check OK");

    // Compile
    let project = vidra_lang::Compiler::compile(&ast).map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("   ✓ Compile OK");

    // Validate
    vidra_ir::validate::validate_project(&project).map_err(|errors| {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("Validation errors:\n  {}", msgs.join("\n  "))
    })?;
    println!("   ✓ Validate OK");

    println!();
    println!("   ✅ No errors found.");

    Ok(())
}

fn cmd_fmt(file: PathBuf, check: bool) -> Result<()> {
    let source = std::fs::read_to_string(&file)
        .with_context(|| format!("failed to read file: {}", file.display()))?;

    let ast = parse_and_resolve_imports(&file)?;

    let formatted = vidra_lang::Formatter::format(&ast);

    if check {
        if source != formatted {
            anyhow::bail!("File is not properly formatted: {}", file.display());
        }
        println!("   ✨ {} is properly formatted", file.display());
    } else {
        std::fs::write(&file, formatted)
            .with_context(|| format!("failed to write formatter output: {}", file.display()))?;
        println!("   ✨ Formatted {}", file.display());
    }

    Ok(())
}

fn cmd_info() -> Result<()> {
    println!("🎬 Vidra Video Engine");
    println!("   Version:   {}", env!("CARGO_PKG_VERSION"));
    println!("   Renderer:  CPU (single-threaded, Phase 0)");
    println!("   Encoder:   FFmpeg (H.264)");
    println!(
        "   FFmpeg:    {}",
        if vidra_encode::FfmpegEncoder::is_available() {
            "available ✓"
        } else {
            "NOT FOUND ✗"
        }
    );
    println!("   Language:  VidraScript");
    println!();
    println!("   Repository: https://github.com/vidra-dev/vidra");
    println!("   Website:    https://vidra.dev");
    Ok(())
}

fn cmd_init(name: &str, kit: Option<String>) -> Result<()> {
    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create project structure
    std::fs::create_dir_all(&project_dir)
        .with_context(|| format!("failed to create project directory: {}", name))?;
    std::fs::create_dir_all(project_dir.join("assets"))
        .with_context(|| format!("failed to create assets directory in: {}", name))?;

    // Create vidra.config.toml
    let mut config = vidra_core::VidraConfig::default();
    config.project.name = name.to_string();
    config.project.resolution = "1920x1080".to_string();
    config.project.fps = 60;
    if let Some(ref k) = kit {
        config.brand.kit = Some(k.clone());
    }

    config
        .save_to_file(&project_dir.join("vidra.config.toml"))
        .map_err(|e| anyhow::anyhow!("{}", e))
        .with_context(|| format!("failed to create vidra.config.toml in: {}", name))?;

    // Create main.vidra starter template
    let main_content = if let Some(ref kit_name) = kit {
        match kit_name.as_str() {
            "youtube-intro" => r#"
project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("intro") {
            text("Welcome to my Channel!", font: @brand.font, size: 96, color: @brand.primary)
            position(960, 540)
        }
    }
}
"#
            .trim_start()
            .to_string(),
            _ => r#"
project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("welcome_text") {
            text("Welcome to Vidra!", font: "Inter", size: 96, color: #FFFFFF)
            position(960, 540)
        }
    }
}
"#
            .trim_start()
            .to_string(),
        }
    } else {
        r#"
project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("welcome_text") {
            text("Welcome to Vidra!", font: "Inter", size: 96, color: #FFFFFF)
            position(960, 540)
        }
    }
}
"#
        .trim_start()
        .to_string()
    };

    std::fs::write(project_dir.join("main.vidra"), main_content)
        .with_context(|| format!("failed to create main.vidra in: {}", name))?;

    println!("✅ Created new Vidra project in directory: {}", name);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  vidra render main.vidra");
    Ok(())
}

fn cmd_inspect(file: PathBuf, target_frame: Option<u64>) -> Result<()> {
    let source = std::fs::read_to_string(&file)
        .with_context(|| format!("failed to read file: {}", file.display()))?;

    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
    let is_json = file.extension().map_or(false, |e| e == "json");

    let project = if is_json {
        serde_json::from_str::<vidra_ir::Project>(&source)
            .with_context(|| format!("failed to parse IR JSON: {}", file.display()))?
    } else {
        let ast = parse_and_resolve_imports(&file)?;
        let checker = vidra_lang::TypeChecker::new(file_name.clone());
        if let Err(diags) = checker.check(&ast) {
            let errors: Vec<_> = diags
                .into_iter()
                .filter(|d| d.severity == vidra_lang::checker::DiagnosticSeverity::Error)
                .collect();
            if !errors.is_empty() {
                anyhow::bail!("Type errors; attach --check for details.");
            }
        }
        vidra_lang::Compiler::compile(&ast).map_err(|e| anyhow::anyhow!("{}", e))?
    };

    println!("🔍 Vidra Render Tree Inspector");
    println!(
        "📦 Project: {} ({}x{} @ {}fps)",
        file.display(),
        project.settings.width,
        project.settings.height,
        project.settings.fps
    );

    if let Some(f) = target_frame {
        println!(
            "⏱️  Evaluating at Frame: {} ({:.2}s)",
            f,
            f as f64 / project.settings.fps
        );
    }

    println!("├── 🎬 Scenes ({} total)", project.scenes.len());
    let mut current_global: u64 = 0;

    for (i, scene) in project.scenes.iter().enumerate() {
        let sc_frames = scene.frame_count(project.settings.fps);
        let evaluates_here = target_frame.map_or(false, |f| {
            f >= current_global && f < current_global + sc_frames
        });

        let is_last_scene = i == project.scenes.len() - 1 && project.assets.count() == 0;
        let scene_prefix = if is_last_scene {
            "└──"
        } else {
            "├──"
        };
        let node_prefix = if is_last_scene { "    " } else { "│   " };

        println!(
            "{} 🎞️  Scene '{}' [{:.2}s]",
            scene_prefix,
            scene.id,
            scene.duration.as_seconds()
        );

        let mut layers_to_print: Vec<&vidra_ir::layer::Layer> = Vec::new();
        // If we specified a frame but it's not in this scene, we skip printing layers.
        if target_frame.is_none() || evaluates_here {
            // Wait, even if we specify a frame, we might want to see the whole tree, just with values.
            // Let's print all layers but only evaluate values if evaluated_here.
            for layer in &scene.layers {
                layers_to_print.push(layer);
            }
        }

        let eval_time = if evaluates_here {
            let local_f = target_frame.unwrap() - current_global;
            Some(vidra_core::Duration::from_seconds(
                local_f as f64 / project.settings.fps,
            ))
        } else {
            None
        };

        for (j, layer) in layers_to_print.iter().enumerate() {
            let is_last_layer = j == layers_to_print.len() - 1;
            print_layer(layer, &node_prefix, is_last_layer, eval_time);
        }

        current_global += sc_frames;
    }

    // Print assets only if there are any
    if project.assets.count() > 0 {
        println!("└── 🗃️  Assets ({} total)", project.assets.count());
        let assets: Vec<_> = project.assets.all().collect();
        for (i, asset) in assets.iter().enumerate() {
            let is_last_asset = i == assets.len() - 1;
            let ast_prefix = if is_last_asset {
                "    └──"
            } else {
                "    ├──"
            };
            println!(
                "{} [{}] {} -> {}",
                ast_prefix,
                asset.asset_type,
                asset.id,
                asset.path.display()
            );
        }
    }

    Ok(())
}

fn print_layer(
    layer: &vidra_ir::layer::Layer,
    prefix: &str,
    is_last: bool,
    eval_time: Option<vidra_core::Duration>,
) {
    let layer_prefix = if is_last { "└──" } else { "├──" };
    let nested_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    let content_type = match &layer.content {
        vidra_ir::layer::LayerContent::Text { text, .. } => format!("Text (\"{}\")", text),
        vidra_ir::layer::LayerContent::Image { asset_id } => format!("Image (asset: {})", asset_id),
        vidra_ir::layer::LayerContent::Video { asset_id, .. } => {
            format!("Video (asset: {})", asset_id)
        }
        vidra_ir::layer::LayerContent::Audio {
            asset_id, volume, ..
        } => format!("Audio (asset: {}, vol: {:.2})", asset_id, volume),
        vidra_ir::layer::LayerContent::Waveform {
            asset_id,
            width,
            height,
            ..
        } => {
            format!("Waveform (audio: {}, {}x{})", asset_id, width, height)
        }
        vidra_ir::layer::LayerContent::Spritesheet {
            asset_id,
            frame_width,
            frame_height,
            fps,
            ..
        } => {
            format!(
                "Spritesheet (asset: {}, tile: {}x{}, fps: {:.2})",
                asset_id, frame_width, frame_height, fps
            )
        }
        vidra_ir::layer::LayerContent::Shape { shape, .. } => format!("Shape ({:?})", shape),
        vidra_ir::layer::LayerContent::Solid { color } => format!("Solid ({})", color),
        vidra_ir::layer::LayerContent::TTS { text, voice, .. } => {
            format!("TTS (\"{}\" voice: {})", text, voice)
        }
        vidra_ir::layer::LayerContent::AutoCaption { asset_id, .. } => {
            format!("AutoCaption (source: {})", asset_id)
        }
        vidra_ir::layer::LayerContent::Shader { asset_id, .. } => {
            format!("Shader (asset: {})", asset_id)
        }
        vidra_ir::layer::LayerContent::Web { source, .. } => format!("Web (source: {})", source),
        vidra_ir::layer::LayerContent::Empty => "Component/Group".to_string(),
    };

    let mut state_suffix = String::new();
    if let Some(t) = eval_time {
        let mut x = layer.transform.position.x;
        let mut y = layer.transform.position.y;
        let mut scale_x = layer.transform.scale.x;
        let mut scale_y = layer.transform.scale.y;
        let mut opacity = layer.transform.opacity;

        for anim in &layer.animations {
            if let Some(val) = anim.evaluate(t) {
                match anim.property {
                    vidra_ir::animation::AnimatableProperty::PositionX => x = val,
                    vidra_ir::animation::AnimatableProperty::PositionY => y = val,
                    vidra_ir::animation::AnimatableProperty::ScaleX => scale_x = val,
                    vidra_ir::animation::AnimatableProperty::ScaleY => scale_y = val,
                    vidra_ir::animation::AnimatableProperty::Opacity => opacity = val,
                    vidra_ir::animation::AnimatableProperty::Rotation => {}
                    _ => {} // Extended properties (fontSize, color, etc.)
                }
            }
        }
        state_suffix = format!(
            " -> pos({:.1}, {:.1}) scale({:.2}, {:.2}) opacity({:.2})",
            x, y, scale_x, scale_y, opacity
        );
    }

    println!(
        "{}{} 🏷️  Layer '{}' ({}){}",
        prefix, layer_prefix, layer.id, content_type, state_suffix
    );

    // Print animations
    for anim in &layer.animations {
        let is_last_anim = layer.children.is_empty()
            && layer.animations.last().unwrap() as *const _ == anim as *const _;
        let anim_prefix = if is_last_anim {
            "└──"
        } else {
            "├──"
        };
        let mut keyframes_str = String::new();
        for (k, keyframe) in anim.keyframes.iter().enumerate() {
            keyframes_str.push_str(&format!(
                "[{:.2}s -> {:.1}]",
                keyframe.time.as_seconds(),
                keyframe.value
            ));
            if k < anim.keyframes.len() - 1 {
                keyframes_str.push_str(" ");
            }
        }

        let eval_str = if let Some(t) = eval_time {
            if let Some(val) = anim.evaluate(t) {
                format!(" => {:.2}", val)
            } else {
                " => (inactive)".to_string()
            }
        } else {
            "".to_string()
        };

        println!(
            "{}{} ✨ Animation {} : {}{}",
            nested_prefix, anim_prefix, anim.property, keyframes_str, eval_str
        );
    }

    // Print children
    for (i, child) in layer.children.iter().enumerate() {
        let is_last_child = i == layer.children.len() - 1;
        print_layer(child, &nested_prefix, is_last_child, eval_time);
    }
}

pub(crate) fn parse_and_resolve_imports(
    file: &std::path::Path,
) -> Result<vidra_lang::ast::ProjectNode> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("failed to read file: {}", file.display()))?;

    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
    let mut lexer = vidra_lang::Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut parser = vidra_lang::Parser::new(tokens, file_name.to_string());
    let mut ast = parser.parse().map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut visited_files = std::collections::HashSet::new();
    visited_files.insert(std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf()));

    let mut pending_imports: std::collections::VecDeque<(PathBuf, String)> =
        std::collections::VecDeque::new();
    for imp in ast.imports.drain(..) {
        pending_imports.push_back((file.to_path_buf(), imp.path));
    }

    while let Some((base_file, import_path)) = pending_imports.pop_front() {
        let parent = base_file.parent().unwrap_or(std::path::Path::new(""));
        let target_path = if import_path.starts_with("/") {
            PathBuf::from(&import_path)
        } else {
            parent.join(&import_path)
        };
        let canonical_target =
            std::fs::canonicalize(&target_path).unwrap_or_else(|_| target_path.clone());
        if visited_files.contains(&canonical_target) {
            continue; // prevent loops
        }
        visited_files.insert(canonical_target.clone());

        let imported_src = std::fs::read_to_string(&target_path)
            .with_context(|| format!("failed to read imported file: {}", target_path.display()))?;
        let mut lexer = vidra_lang::Lexer::new(&imported_src);
        let tokens = lexer.tokenize().map_err(|e| anyhow::anyhow!("{}", e))?;
        let mut parser = vidra_lang::Parser::new(tokens, import_path.clone());
        let (mut new_imports, mut new_assets, new_components) = parser
            .parse_module()
            .map_err(|e| anyhow::anyhow!("{}: {}", import_path, e))?;

        // Merge components into `ast`
        for comp in new_components {
            if !ast.components.iter().any(|c| c.name == comp.name) {
                ast.components.push(comp);
            }
        }
        ast.assets.append(&mut new_assets);
        for imp in new_imports.drain(..) {
            pending_imports.push_back((canonical_target.clone(), imp.path));
        }
    }
    Ok(ast)
}

#[allow(dead_code)]
fn cmd_format(file: PathBuf) -> Result<()> {
    println!("✨ Formatting {}", file.display());
    Ok(())
}

fn sha256_file_prefixed(path: &std::path::Path) -> Result<String> {
    use std::io::Read;

    let mut f = std::fs::File::open(path)
        .with_context(|| format!("failed to open file for hashing: {}", path.display()))?;

    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = f
            .read(&mut buf)
            .context("failed to read file for hashing")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(format!("sha256:{}", hex))
}

fn cmd_sync_push() -> Result<()> {
    println!("☁️  Pushing local changes to Vidra Cloud...");

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    if sync_cloud::cloud_base_url_from_env().is_some() {
        let report = sync_cloud::push_all_to_cloud(&project_root)?;
        println!(
            "   ✓ Uploaded to cloud: {} receipt(s), {} upload(s)",
            report.receipts_uploaded, report.uploads_uploaded
        );
        if !report.receipt_failures.is_empty() || !report.upload_failures.is_empty() {
            if !report.receipt_failures.is_empty() {
                println!(
                    "   ⚠️  Receipt failures ({}):",
                    report.receipt_failures.len()
                );
                for f in &report.receipt_failures {
                    println!("      - {}", f);
                }
            }
            if !report.upload_failures.is_empty() {
                println!("   ⚠️  Upload failures ({}):", report.upload_failures.len());
                for f in &report.upload_failures {
                    println!("      - {}", f);
                }
            }
            anyhow::bail!("cloud push completed with failures");
        }
        return Ok(());
    }

    if let Some(dir) = sync_tools::receipts_root_dir() {
        let moved = sync_tools::push_receipts_local(&dir).unwrap_or(0);
        println!("   ✓ Queued receipts pushed (local): {}", moved);
    } else {
        println!("   ⚠️  Could not resolve receipts directory");
    }
    let moved_uploads = sync_tools::push_uploads_local(&project_root).unwrap_or(0);
    if moved_uploads > 0 {
        println!("   ✓ Queued uploads pushed (local): {}", moved_uploads);
    }
    Ok(())
}

fn cmd_sync_pull() -> Result<()> {
    println!("☁️  Pulling remote changes...");
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    if sync_cloud::cloud_base_url_from_env().is_some() {
        let report = sync_cloud::pull_all_from_cloud(&project_root)?;
        println!(
            "   ✓ Downloaded: {} receipt(s), {} upload(s)",
            report.receipts_downloaded, report.uploads_downloaded
        );
        if !report.receipt_failures.is_empty() || !report.upload_failures.is_empty() {
            if !report.receipt_failures.is_empty() {
                println!(
                    "   ⚠️  Receipt failures ({}):",
                    report.receipt_failures.len()
                );
                for f in &report.receipt_failures {
                    println!("      - {}", f);
                }
            }
            if !report.upload_failures.is_empty() {
                println!("   ⚠️  Upload failures ({}):", report.upload_failures.len());
                for f in &report.upload_failures {
                    println!("      - {}", f);
                }
            }
            anyhow::bail!("cloud pull completed with failures");
        }
        return Ok(());
    }

    println!("   Cloud endpoint: (unset) — nothing to pull.");
    println!("   Tip: set VIDRA_CLOUD_URL to enable cloud pull.");
    Ok(())
}

fn cmd_sync_status() -> Result<()> {
    println!("☁️  Vidra Cloud Sync Status:");
    println!("   Local branch: main");

    if let Some(url) = sync_cloud::cloud_base_url_from_env() {
        println!("   Cloud endpoint: {}", url);
    } else {
        println!("   Cloud endpoint: (unset) — local-only queues");
    }

    if let Some(dir) = sync_tools::receipts_root_dir() {
        let status = sync_tools::receipt_sync_status(&dir)
            .unwrap_or(sync_tools::ReceiptSyncStatus { queued: 0, sent: 0 });
        println!(
            "   Render receipts: {} queued, {} sent",
            status.queued, status.sent
        );
    } else {
        println!("   Render receipts: (unknown)");
    }

    let manifest_path = std::path::PathBuf::from(".vidra").join("asset_manifest.json");
    if manifest_path.exists() {
        match sync_tools::read_asset_manifest(&manifest_path) {
            Ok(m) => {
                let stats = sync_tools::asset_manifest_stats_from_manifest(&m);
                println!(
                    "   Assets: {} total ({} missing, {} hashed, {} remote-url) — manifest updated {}",
                    stats.total,
                    stats.missing,
                    stats.hashed,
                    stats.remote_urls,
                    m.generated_at.to_rfc3339()
                );
            }
            Err(_) => {
                println!(
                    "   Assets: manifest unreadable ({})",
                    manifest_path.display()
                );
            }
        }
    } else {
        println!("   Assets: no manifest (run `vidra sync assets`)");
    }

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Ok(s) = sync_tools::upload_sync_status(&project_root) {
        println!("   Upload queue: {} queued, {} sent", s.queued, s.sent);
    }
    Ok(())
}

fn cmd_sync_assets() -> Result<()> {
    println!("📥 Hydrating missing assets...");

    let config_path = std::path::PathBuf::from("vidra.config.toml");
    if !config_path.exists() {
        anyhow::bail!("vidra.config.toml not found in current directory");
    }
    let config = vidra_core::VidraConfig::load_from_file(&config_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let entry = if std::path::Path::new("main.vidra").exists() {
        std::path::PathBuf::from("main.vidra")
    } else {
        // Fallback: first .vidra file in cwd.
        let mut found: Option<std::path::PathBuf> = None;
        for e in std::fs::read_dir(".").context("failed to read current directory")? {
            let p = e?.path();
            if p.extension().and_then(|x| x.to_str()) == Some("vidra") {
                found = Some(p);
                break;
            }
        }
        found.ok_or_else(|| anyhow::anyhow!("no entry .vidra file found (expected main.vidra)"))?
    };

    let ast = parse_and_resolve_imports(&entry)?;
    let mut project = vidra_lang::Compiler::compile(&ast).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Hydrate remote URLs to local cache paths.
    let remote_report = remote_assets::prepare_project_remote_assets(&mut project, &config)?;
    if remote_report.downloaded > 0 || remote_report.reused_from_cache > 0 {
        println!(
            "   ✓ Remote assets: {} downloaded, {} cached",
            remote_report.downloaded, remote_report.reused_from_cache
        );
    }

    let (manifest, stats) = sync_tools::generate_asset_manifest(&project.id, &project.assets)?;
    let out_path = std::path::PathBuf::from(".vidra").join("asset_manifest.json");
    sync_tools::write_asset_manifest(&out_path, &manifest)?;

    println!("   ✓ Asset manifest: {}", out_path.display());
    println!(
        "   ✓ Assets: {} total ({} missing, {} hashed)",
        stats.total, stats.missing, stats.hashed
    );
    if stats.missing > 0 {
        println!("   ⚠️  Some assets are missing on disk");
    }
    Ok(())
}

fn cmd_jobs_list() -> Result<()> {
    let Some(jobs_root) = jobs_tools::jobs_root_dir() else {
        anyhow::bail!("failed to resolve home dir for ~/.vidra/jobs");
    };

    if let Some(base) = sync_cloud::cloud_base_url_from_env() {
        match jobs_cloud::fetch_jobs_from_cloud(&base)
            .and_then(|jobs| jobs_cloud::enqueue_cloud_jobs(&jobs_root, jobs, None))
        {
            Ok(added) if added > 0 => {
                println!("☁️  Synced {} cloud job(s) into local queue", added)
            }
            Ok(_) => {}
            Err(e) => println!("⚠️  Cloud jobs fetch failed: {:#}", e),
        }
    }

    let queued = jobs_tools::list_queued_jobs(&jobs_root)?;
    println!("☁️  Pending Jobs (local queue):");

    if queued.is_empty() {
        println!("   (none)");
        println!();
        println!("   0 pending jobs.");
        return Ok(());
    }

    for j in &queued {
        println!(
            "   ID: {} | Project: {} | File: {} | Queued: {}",
            j.spec.job_id,
            j.spec
                .project_root
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("(project)"),
            j.spec.vidra_file.display(),
            j.spec.created_at.to_rfc3339()
        );
    }
    println!();
    println!("   {} pending job(s).", queued.len());
    Ok(())
}

fn cmd_jobs_run(all: bool) -> Result<()> {
    let Some(jobs_root) = jobs_tools::jobs_root_dir() else {
        anyhow::bail!("failed to resolve home dir for ~/.vidra/jobs");
    };

    // Best-effort cloud pull before local execution.
    if let Some(base) = sync_cloud::cloud_base_url_from_env() {
        let limit = if all { None } else { Some(1) };
        if let Ok(jobs) = jobs_cloud::fetch_jobs_from_cloud(&base) {
            let _ = jobs_cloud::enqueue_cloud_jobs(&jobs_root, jobs, limit);
        }
    }

    let mut ran_any = false;
    loop {
        let Some(claimed) = jobs_tools::claim_next_job(&jobs_root)? else {
            if !ran_any {
                println!("🚀 No pending jobs.");
            }
            break;
        };
        ran_any = true;

        let job_id = claimed.spec.job_id.clone();
        println!("🚀 Running job {}...", job_id);

        let prev_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        if let Err(e) = std::env::set_current_dir(&claimed.spec.project_root) {
            let msg = format!(
                "failed to set current dir to {}: {}",
                claimed.spec.project_root.display(),
                e
            );
            let _ = jobs_tools::mark_job_failed(&jobs_root, &claimed, &msg);
            anyhow::bail!(msg);
        }

        let started = Instant::now();
        let render_result = cmd_render(
            claimed.spec.vidra_file.clone(),
            claimed.spec.output.clone(),
            claimed.spec.format.clone(),
            claimed.spec.targets.clone(),
            false,
            claimed.spec.data.clone(),
        );

        // Restore cwd even if render fails.
        let _ = std::env::set_current_dir(&prev_dir);

        match render_result {
            Ok(()) => {
                let elapsed = started.elapsed();
                let _ = jobs_tools::mark_job_sent(&jobs_root, &claimed);
                println!("   ✓ Finished in {:.2?} (receipt queued)", elapsed);
            }
            Err(e) => {
                let msg = format!("render failed: {:#}", e);
                let _ = jobs_tools::mark_job_failed(&jobs_root, &claimed, &msg);
                println!("   ✗ Job {} failed", job_id);
                // Keep going for RunAll; fail fast for single-run.
                if !all {
                    return Err(e);
                }
            }
        }

        if !all {
            break;
        }
    }

    Ok(())
}

fn cmd_jobs_watch() -> Result<()> {
    println!("👀 Starting Vidra Job Daemon (local queue)...");
    println!("   Polling for new jobs every 15s...");
    println!("   (Press Ctrl+C to exit)");

    loop {
        // Run all queued jobs, then sleep.
        let _ = cmd_jobs_run(true);
        std::thread::sleep(std::time::Duration::from_secs(15));
    }
}

fn cmd_upload(path: &std::path::Path) -> Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let count = sync_tools::enqueue_upload_path(&project_root, path)?;
    println!("☁️  Queued '{}' for upload...", path.display());
    println!("   ✓ Enqueued {} file(s) (local queue)", count);
    Ok(())
}

fn cmd_assets_list() -> Result<()> {
    println!("📦 Assets:");

    let manifest_path = std::path::PathBuf::from(".vidra").join("asset_manifest.json");
    if manifest_path.exists() {
        if let Ok(m) = sync_tools::read_asset_manifest(&manifest_path) {
            let stats = sync_tools::asset_manifest_stats_from_manifest(&m);
            println!(
                "   Manifest: {} assets ({} missing)",
                stats.total, stats.missing
            );
            for a in m.assets.iter().take(50) {
                let status = if a.exists { "✓" } else { "✗" };
                let size = a
                    .size_bytes
                    .map(|s| format!("{}B", s))
                    .unwrap_or_else(|| "?".to_string());
                println!("   - [{}] {} ({}, {})", status, a.path, a.asset_type, size);
            }
            if m.assets.len() > 50 {
                println!("   … ({} more)", m.assets.len() - 50);
            }
        } else {
            println!("   Manifest: unreadable ({})", manifest_path.display());
        }
    } else {
        println!("   Manifest: none (run `vidra sync assets`)");
    }

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Ok(s) = sync_tools::upload_sync_status(&project_root) {
        println!("   Upload queue: {} queued, {} sent", s.queued, s.sent);
    }
    Ok(())
}

fn cmd_assets_pull(name: &str) -> Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let Some((blob_path, suggested_name)) = sync_tools::resolve_upload_blob(&project_root, name)?
    else {
        anyhow::bail!("asset '{}' not found in local upload queue", name);
    };

    let out_dir = std::path::PathBuf::from("assets");
    std::fs::create_dir_all(&out_dir).context("failed to create assets dir")?;

    let out_path = out_dir.join(suggested_name);
    std::fs::copy(&blob_path, &out_path)
        .with_context(|| format!("failed to copy asset blob to {}", out_path.display()))?;

    println!("📥 Pulled '{}' from local cache", name);
    println!("   ✓ Saved to {}", out_path.display());
    Ok(())
}

fn cmd_search(query: &str) -> Result<()> {
    let q = query.to_lowercase();
    let templates = template_manager::available_templates();
    let matches: Vec<_> = templates
        .into_iter()
        .filter(|t| t.name.contains(&q) || t.description.to_lowercase().contains(&q))
        .collect();

    println!("🔍 Searching local templates for '{}'...", query);
    if matches.is_empty() {
        println!("   No matches.");
        println!("   Tip: run `vidra explore` to see all built-in templates.");
        return Ok(());
    }

    println!("   Found {} result(s):", matches.len());
    for t in matches {
        println!("   - template: '{}' ({})", t.name, t.description);
    }
    Ok(())
}

fn cmd_explore() -> Result<()> {
    let templates = template_manager::available_templates();
    println!("🌟 Templates (built-in):");
    for (i, t) in templates.iter().enumerate() {
        println!("   {}. '{}' — {}", i + 1, t.name, t.description);
    }
    println!();
    println!("   Install one with: vidra add <template>");
    Ok(())
}

fn cmd_licenses() -> Result<()> {
    println!("📜 Project Asset Licenses (local)");

    let manifest_path = std::path::PathBuf::from(".vidra").join("asset_manifest.json");
    if !manifest_path.exists() {
        println!("   No manifest found at {}", manifest_path.display());
        println!("   Run `vidra sync assets` first.");
        return Ok(());
    }

    let licensed = licenses_tools::licenses_from_manifest_path(&manifest_path)?;
    if licensed.is_empty() {
        println!("   (no assets in manifest)");
        return Ok(());
    }

    let mut known = 0usize;
    let mut unknown = 0usize;
    let mut missing = 0usize;

    for a in licensed {
        match a.status {
            licenses_tools::LicenseStatus::Known(s) => {
                known += 1;
                println!("   - [{}] {} — {}", a.asset_type, a.path, s);
            }
            licenses_tools::LicenseStatus::Unknown => {
                unknown += 1;
                println!("   - [{}] {} — (unknown)", a.asset_type, a.path);
            }
            licenses_tools::LicenseStatus::MissingAsset => {
                missing += 1;
                println!("   - [{}] {} — (missing on disk)", a.asset_type, a.path);
            }
        }
    }

    println!();
    println!(
        "   Summary: {} known, {} unknown, {} missing",
        known, unknown, missing
    );
    if unknown > 0 {
        println!(
            "   Tip: add a sidecar file like <asset>.license.txt with the license identifier/text."
        );
    }
    Ok(())
}

fn cmd_brand_create(name: &str) -> Result<()> {
    let path = brand_tools::create_brand_kit(name)?;
    println!("✨ Created brand kit: '{}'", name);
    println!("   Saved locally to {}", path.display());
    Ok(())
}

fn cmd_brand_list() -> Result<()> {
    let kits = brand_tools::list_brand_kits()?;
    println!("🎨 Your Brand Kits:");
    if kits.is_empty() {
        println!("   (none)");
        return Ok(());
    }
    for k in kits {
        println!("   - {} ({})", k.name, k.created_at.to_rfc3339());
    }
    Ok(())
}

fn cmd_brand_apply(name: &str) -> Result<()> {
    if !brand_tools::brand_kit_exists(name)? {
        anyhow::bail!("brand kit '{}' not found (run `vidra brand list`) ", name);
    }
    let path = std::path::PathBuf::from("vidra.config.toml");
    if !path.exists() {
        anyhow::bail!("vidra.config.toml not found in current directory");
    }
    let mut config =
        vidra_core::VidraConfig::load_from_file(&path).map_err(|e| anyhow::anyhow!("{}", e))?;
    config.brand.kit = Some(name.to_string());
    config
        .save_to_file(&path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("✅ Applied brand kit '{}' to project.", name);
    Ok(())
}

fn cmd_workspace_create(name: &str) -> Result<()> {
    let s = workspace_tools::create_workspace(name)?;
    println!("🏢 Created workspace: '{}'", name);
    if let Some(active) = s.active {
        println!("   Active workspace: {}", active);
    }
    Ok(())
}

fn cmd_workspace_list() -> Result<()> {
    let s = workspace_tools::list_workspaces()?;
    println!("🏢 Your Workspaces:");
    if s.workspaces.is_empty() {
        println!("   (none)");
        return Ok(());
    }
    for w in s.workspaces {
        let marker = if s.active.as_deref() == Some(&w.name) {
            ">"
        } else {
            " "
        };
        println!("   {} {}", marker, w.name);
    }
    Ok(())
}

fn cmd_workspace_switch(name: &str) -> Result<()> {
    let s = workspace_tools::switch_workspace(name)?;
    println!("🔄 Switched active workspace to '{}'", name);
    if let Some(active) = s.active {
        println!("   Active workspace: {}", active);
    }
    Ok(())
}

fn cmd_workspace_invite(email: &str) -> Result<()> {
    workspace_tools::invite_to_active_workspace(email)?;
    println!("✉️  Invited {} to current workspace.", email);
    Ok(())
}

fn cmd_publish(path: &PathBuf) -> Result<()> {
    println!("📦 Publishing resource: {}", path.display());
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let (pkg_path, is_tmp) = publish_tools::package_for_publish(path)?;
    let pkg_sha = sha256_file_prefixed(&pkg_path)?;
    let resource_id = publish_tools::resource_id_from_sha256_prefixed(&pkg_sha);

    let enqueued = sync_tools::enqueue_upload_path(&project_root, &pkg_path)?;
    if is_tmp {
        let _ = std::fs::remove_file(&pkg_path);
    }

    println!("   ✓ Packaged: {}", pkg_sha);
    println!("   ✓ Queued for upload: {} file(s)", enqueued);

    if let Some(base) = sync_cloud::cloud_base_url_from_env() {
        let uploaded = sync_cloud::push_uploads_to_cloud(&project_root, &base)?;
        if !uploaded.failures.is_empty() {
            println!("   ⚠️  Upload failures ({}):", uploaded.failures.len());
            for f in &uploaded.failures {
                println!("      - {}", f);
            }
            anyhow::bail!("publish upload completed with failures");
        }
        println!("✅ Resource published! ID: {}", resource_id);
        println!("   URL: {}/commons/resources/{}", base, resource_id);
        return Ok(());
    }

    println!("✅ Resource queued. ID: {}", resource_id);
    println!("   Link: vidra://commons/{}", resource_id);
    println!("   (Set VIDRA_CLOUD_URL then run `vidra sync push` to publish)");
    Ok(())
}

fn cmd_plugin_list() -> Result<()> {
    let plugins = plugin_tools::list_plugins()?;
    println!("🔌 Installed Plugins:");
    if plugins.is_empty() {
        println!("   (none)");
        return Ok(());
    }
    for (i, p) in plugins.iter().enumerate() {
        println!("   [{}] {}  v{}", i + 1, p.name, p.version);
    }
    Ok(())
}

fn cmd_plugin_install(name: &str) -> Result<()> {
    let path = plugin_tools::install_plugin(name, None)?;
    println!("🔌 Installed plugin '{}'", name);
    println!("   Manifest: {}", path.display());
    Ok(())
}

fn cmd_plugin_remove(name: &str) -> Result<()> {
    plugin_tools::remove_plugin(name)?;
    println!("🗑️  Removed plugin '{}'.", name);
    Ok(())
}

fn cmd_plugin_info(name: &str) -> Result<()> {
    let m = plugin_tools::read_plugin_manifest(name)?;
    println!("🔌 Plugin: {}", m.name);
    println!("   Version:     {}", m.version);
    println!("   Installed:   {}", m.installed_at.to_rfc3339());
    Ok(())
}

fn cmd_dashboard() -> Result<()> {
    println!("📊 Render Observability Dashboard (local)");

    let now = chrono::Utc::now();
    let cutoff = now - chrono::Duration::hours(24);

    let mut receipts_last_24h: Vec<crate::receipt::RenderReceipt> = Vec::new();
    if let Some(root) = sync_tools::receipts_root_dir() {
        for dir in [root.clone(), sync_tools::receipts_sent_dir(&root)] {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    if path.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }
                    let Ok(raw) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    let Ok(r) = serde_json::from_str::<crate::receipt::RenderReceipt>(&raw) else {
                        continue;
                    };
                    if r.payload.timestamp >= cutoff {
                        receipts_last_24h.push(r);
                    }
                }
            }
        }
    }

    let render_count = receipts_last_24h.len();
    let avg_ms = if render_count == 0 {
        None
    } else {
        Some(
            receipts_last_24h
                .iter()
                .map(|r| r.payload.render_duration_ms)
                .sum::<u64>()
                / (render_count as u64),
        )
    };

    println!();
    println!("   Renders (last 24h):   {}", render_count);
    if let Some(ms) = avg_ms {
        println!("   Avg render time:      {:.2}s", (ms as f64) / 1000.0);
    } else {
        println!("   Avg render time:      n/a");
    }

    // Local-first queue status
    if let Some(dir) = sync_tools::receipts_root_dir() {
        if let Ok(s) = sync_tools::receipt_sync_status(&dir) {
            println!(
                "   Receipts queue:       {} queued, {} sent",
                s.queued, s.sent
            );
        }
    }
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Ok(s) = sync_tools::upload_sync_status(&project_root) {
        println!(
            "   Upload queue:         {} queued, {} sent",
            s.queued, s.sent
        );
    }
    if let Some(jobs_root) = jobs_tools::jobs_root_dir() {
        if let Ok(j) = jobs_tools::list_queued_jobs(&jobs_root) {
            println!("   Jobs queue:           {} queued", j.len());
        }
    }

    println!();
    if sync_cloud::cloud_base_url_from_env().is_some() {
        println!("   Cloud dashboard: (set) — use `vidra sync status` for endpoint");
    } else {
        println!("   Cloud dashboard: (unset) — set VIDRA_CLOUD_URL to enable uploads");
    }
    Ok(())
}

fn cmd_storyboard(prompt: &str, output: Option<PathBuf>) -> Result<()> {
    let out = output.unwrap_or_else(|| PathBuf::from("storyboard.png"));

    println!("🎨 Generating Storyboard...");
    println!("   Prompt: \"{}\"", prompt);
    storyboard_tools::generate_storyboard_png(prompt, &out)?;
    println!("✅ Storyboard grid saved to '{}'", out.display());
    Ok(())
}

pub mod bench_runner;
pub mod template_manager;
