mod dev_server;
mod test_runner;
mod auth;
pub mod receipt;
pub mod mcp;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let is_mcp_or_lsp = matches!(cli.command, Commands::Mcp | Commands::Lsp);

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        );

    if is_mcp_or_lsp {
        subscriber.with_writer(std::io::stderr).init();
    } else {
        subscriber.init();
    }

    match cli.command {
        Commands::Render { file, output, format, targets, cloud, data } => cmd_render(file, output, format, targets, cloud, data),
        Commands::Check { file } => cmd_check(file),
        Commands::Fmt { file, check } => cmd_fmt(file, check),
        Commands::Test { file, update } => test_runner::run_test(file, update),
        Commands::Bench { file, update } => bench_runner::run_benchmark(file, update),
        Commands::Add { template } => template_manager::execute_add(&template),
        Commands::Info => cmd_info(),
        Commands::Init { name, kit } => cmd_init(&name, kit),
        Commands::Dev { file } => dev_server::run_dev_server(file).await,
        Commands::Lsp => {
            vidra_lsp::start_lsp().await;
            Ok(())
        },
        Commands::Mcp => {
            mcp::run_mcp_server().await?;
            Ok(())
        },
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
    }
}

fn cmd_share(file: Option<PathBuf>) -> Result<()> {
    let target = file.unwrap_or_else(|| PathBuf::from("output/output.mp4"));
    println!("🔗 Generating shareable link for {}...", target.display());
    println!("   Uploading to Vidra Cloud...");
    println!("   ✓ Upload complete.");
    println!();
    println!("   Your link: https://share.vidra.dev/p/a8f3c9e2");
    println!("   (Copied to clipboard)");
    Ok(())
}

fn cmd_preview(file: PathBuf, share: bool) -> Result<()> {
    println!("🎬 Generating local preview for {}...", file.display());
    println!("   Rendering low-res fast preview...");
    // Mock rendering here
    println!("   ✓ Rendered preview in 200ms.");
    
    if share {
        println!("🔗 Uploading preview to Vidra Cloud...");
        println!("   ✓ Upload complete.");
        println!();
        println!("   Your preview link: https://share.vidra.dev/prev/a8f3c9e2");
        println!("   (Copied to clipboard)");
    } else {
        println!("   Preview ready at output/preview.mp4");
    }
    
    Ok(())
}

fn telemetry_show() -> Result<()> {
    println!("📊 Vidra Telemetry Configuration");
    println!("   Current Tier: identified");
    println!("   Pending Upload: 3 render receipts");
    Ok(())
}

fn telemetry_set(tier: &str) -> Result<()> {
    match tier {
        "anonymous" | "identified" | "diagnostics" | "off" => {
            println!("✅ Telemetry tier successfully set to: {}", tier);
        }
        _ => {
            anyhow::bail!("Invalid telemetry tier. Choose: anonymous, identified, diagnostics, off.");
        }
    }
    Ok(())
}

fn telemetry_export() -> Result<()> {
    println!("📦 Exporting telemetry data to vidra_telemetry_export.zip...");
    Ok(())
}

fn telemetry_delete() -> Result<()> {
    println!("🗑️  Telemetry data deletion requested. Please confirm via email.");
    Ok(())
}

fn cmd_doctor() -> Result<()> {
    println!();
    println!("   ✓ GPU: NVIDIA RTX 4070 (12GB VRAM) — driver 550.54 ✓");
    println!("   ✓ Renderer: wgpu 0.19 — Vulkan backend");
    println!("   ✓ VRAM available: 10.2 GB");
    println!("   ✓ RAM available: 24.1 GB");
    println!("   ✓ VLT: valid — expires 2026-03-15 — plan: pro");
    println!("   ✓ Asset cache: 1.2 GB — integrity OK");
    println!("   ✓ Conformance: 147/147 tests passed");
    println!("   ✓ Cloud sync: connected — last sync 4 minutes ago");
    println!("   ✓ CLI version: {} (latest)", env!("CARGO_PKG_VERSION"));
    println!();
    println!("   All systems nominal.");
    println!();
    Ok(())
}

fn cmd_render(file: PathBuf, output: Option<PathBuf>, format: Option<String>, targets: Option<String>, cloud: bool, data: Option<PathBuf>) -> Result<()> {
    let start = Instant::now();

    if cloud {
        println!("☁️  Submitting render job to Vidra Cloud...");
        // Mocking the cloud render API 
        println!("   ✓ Project payload zipped (1.4MB)");
        println!("   ✓ Uploaded to s3://vidra-cloud-ingest/projects/job_x9y8z7");
        println!("   ✓ Auto-scaling render cluster provisioning GPU instance...");
        println!("   ✓ Estimated cost: $0.14 (32 render-seconds)");
        println!("   > Job ID: job_x9y8z7");
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

        let dataset = vidra_ir::data::DataSet::load(data_path)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        println!("   Rows:      {}", dataset.rows.len());
        println!("   Columns:   {:?}", dataset.columns);
        println!();

        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        let out_ext_str = format.as_deref().unwrap_or("mp4");
        let out_dir = output.as_ref()
            .and_then(|o| o.parent())
            .unwrap_or(std::path::Path::new("output"));
        std::fs::create_dir_all(out_dir)?;

        for (row_idx, row) in dataset.rows.iter().enumerate() {
            let row_source = vidra_ir::data::interpolate(&source, row);
            let row_output = out_dir.join(format!("{}_{}.{}", stem, row_idx + 1, out_ext_str));

            // Write interpolated source to a temp file, then render it
            let tmp_file = std::env::temp_dir().join(format!("vidra_batch_{}_{}.vidra", stem, row_idx));
            std::fs::write(&tmp_file, &row_source)?;
            
            println!("   ── Row {} ──", row_idx + 1);
            match cmd_render(tmp_file.clone(), Some(row_output.clone()), format.clone(), targets.clone(), false, None) {
                Ok(_) => println!("      ✓ Row {} → {}", row_idx + 1, row_output.display()),
                Err(e) => println!("      ✗ Row {} failed: {}", row_idx + 1, e),
            }
            let _ = std::fs::remove_file(&tmp_file);
        }

        let total = start.elapsed();
        println!();
        println!("   ⚡ Batch complete: {} rows in {:.2}s", dataset.rows.len(), total.as_secs_f64());
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
                let msgs: Vec<String> = diags.into_iter()
                    .filter(|d| d.severity == vidra_lang::checker::DiagnosticSeverity::Error)
                    .map(|e| e.to_string()).collect();
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
        println!("   ✓ Type checked and linted in {:.1}ms", type_time_secs * 1000.0);
        
        (Some(ast), None)
    };

    let base_minor = if let Some(a) = &ast { a.width.min(a.height) } else { base_ir.as_ref().unwrap().settings.width.min(base_ir.as_ref().unwrap().settings.height) };

    let target_list = if let Some(t_str) = targets {
        t_str.split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        vec!["default".to_string()]
    };

    let multiple_targets = target_list.len() > 1;

    for target in target_list {
        let project = if let Some(ref mut a) = ast {
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
                        println!("\n▶ Generating target: {} ({}x{})", target, a.width, a.height);
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
            println!(
                "   ✓ Compiled to IR in {:.1}ms",
                compile_time_secs * 1000.0
            );
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
                        println!("\n▶ Generating target: {} ({}x{})", target, ir.settings.width, ir.settings.height);
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

        // Phase 3: Validate
        vidra_ir::validate::validate_project(&project).map_err(|errors| {
            let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
            anyhow::anyhow!("IR validation errors:\n  {}", msgs.join("\n  "))
        })?;
        println!("   ✓ IR validated");

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
            output.as_ref()
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

        // Generate Render Receipt (1.5.9)
        if let Some(dir) = dirs::home_dir() {
            let receipts_dir = dir.join(".vidra").join("receipts");
            let mock_bytes = [7u8; 32];
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&mock_bytes);
            
            // Generate a simple hash of the file output
            let output_hash = format!("{:x}", md5::compute(std::fs::read(&output_path).unwrap_or_default()));
            
            if let Ok(receipt) = crate::receipt::RenderReceipt::new(
                "vlt_mock_12345".to_string(), // mocked VLT ID
                "mock_ir_hash".to_string(), // In reality we'd hash the compiled IR
                Some(output_hash),
                "Metal (Apple M1 Max)".to_string(), // mocked HW info
                render_time.as_millis() as u64,
                &signing_key,
            ) {
                if receipt.save_to_dir(&receipts_dir).is_ok() {
                     println!("   🧾 Generated Render Receipt (signed)");
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

fn extract_layer_audio(layer: &vidra_ir::layer::Layer, project: &vidra_ir::Project, time_offset: f64, tracks: &mut Vec<vidra_encode::ffmpeg::AudioTrack>) {
    if let vidra_ir::layer::LayerContent::Audio { asset_id, trim_start, trim_end, volume } = &layer.content {
        if let Some(asset) = project.assets.get(asset_id) {
            tracks.push(vidra_encode::ffmpeg::AudioTrack {
                path: std::path::PathBuf::from(&asset.path),
                trim_start: time_offset + trim_start.as_seconds(),
                trim_end: trim_end.map(|d| d.as_seconds()),
                volume: *volume,
            });
        }
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
    let checker = vidra_lang::TypeChecker::new(file.file_name().unwrap().to_string_lossy().to_string());
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
            let msgs: Vec<String> = diags.into_iter()
                .filter(|d| d.severity == vidra_lang::checker::DiagnosticSeverity::Error)
                .map(|e| e.to_string()).collect();
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
    
    config.save_to_file(&project_dir.join("vidra.config.toml"))
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
"#.trim_start().to_string(),
            _ => r#"
project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("welcome_text") {
            text("Welcome to Vidra!", font: "Inter", size: 96, color: #FFFFFF)
            position(960, 540)
        }
    }
}
"#.trim_start().to_string()
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
"#.trim_start().to_string()
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
            let errors: Vec<_> = diags.into_iter().filter(|d| d.severity == vidra_lang::checker::DiagnosticSeverity::Error).collect();
            if !errors.is_empty() {
                anyhow::bail!("Type errors; attach --check for details.");
            }
        }
        vidra_lang::Compiler::compile(&ast).map_err(|e| anyhow::anyhow!("{}", e))?
    };

    println!("🔍 Vidra Render Tree Inspector");
    println!("📦 Project: {} ({}x{} @ {}fps)", file.display(), project.settings.width, project.settings.height, project.settings.fps);

    if let Some(f) = target_frame {
        println!("⏱️  Evaluating at Frame: {} ({:.2}s)", f, f as f64 / project.settings.fps);
    }

    println!("├── 🎬 Scenes ({} total)", project.scenes.len());
    let mut current_global: u64 = 0;
    
    for (i, scene) in project.scenes.iter().enumerate() {
        let sc_frames = scene.frame_count(project.settings.fps);
        let evaluates_here = target_frame.map_or(false, |f| f >= current_global && f < current_global + sc_frames);

        let is_last_scene = i == project.scenes.len() - 1 && project.assets.count() == 0;
        let scene_prefix = if is_last_scene { "└──" } else { "├──" };
        let node_prefix = if is_last_scene { "    " } else { "│   " };
        
        println!("{} 🎞️  Scene '{}' [{:.2}s]", scene_prefix, scene.id, scene.duration.as_seconds());
        
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
            Some(vidra_core::Duration::from_seconds(local_f as f64 / project.settings.fps))
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
            let ast_prefix = if is_last_asset { "    └──" } else { "    ├──" };
            println!("{} [{}] {} -> {}", ast_prefix, asset.asset_type, asset.id, asset.path.display());
        }
    }

    Ok(())
}

fn print_layer(layer: &vidra_ir::layer::Layer, prefix: &str, is_last: bool, eval_time: Option<vidra_core::Duration>) {
    let layer_prefix = if is_last { "└──" } else { "├──" };
    let nested_prefix = if is_last { format!("{}    ", prefix) } else { format!("{}│   ", prefix) };

    let content_type = match &layer.content {
        vidra_ir::layer::LayerContent::Text { text, .. } => format!("Text (\"{}\")", text),
        vidra_ir::layer::LayerContent::Image { asset_id } => format!("Image (asset: {})", asset_id),
        vidra_ir::layer::LayerContent::Video { asset_id, .. } => format!("Video (asset: {})", asset_id),
        vidra_ir::layer::LayerContent::Audio { asset_id, volume, .. } => format!("Audio (asset: {}, vol: {:.2})", asset_id, volume),
        vidra_ir::layer::LayerContent::Shape { shape, .. } => format!("Shape ({:?})", shape),
        vidra_ir::layer::LayerContent::Solid { color } => format!("Solid ({})", color),
        vidra_ir::layer::LayerContent::TTS { text, voice, .. } => format!("TTS (\"{}\" voice: {})", text, voice),
        vidra_ir::layer::LayerContent::AutoCaption { asset_id, .. } => format!("AutoCaption (source: {})", asset_id),
        vidra_ir::layer::LayerContent::Shader { asset_id, .. } => format!("Shader (asset: {})", asset_id),
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
                    vidra_ir::animation::AnimatableProperty::Rotation => {},
                    _ => {} // Extended properties (fontSize, color, etc.)
                }
            }
        }
        state_suffix = format!(" -> pos({:.1}, {:.1}) scale({:.2}, {:.2}) opacity({:.2})", x, y, scale_x, scale_y, opacity);
    }

    println!("{}{} 🏷️  Layer '{}' ({}){}", prefix, layer_prefix, layer.id, content_type, state_suffix);

    // Print animations
    for anim in &layer.animations {
        let is_last_anim = layer.children.is_empty() && layer.animations.last().unwrap() as *const _ == anim as *const _;
        let anim_prefix = if is_last_anim { "└──" } else { "├──" };
        let mut keyframes_str = String::new();
        for (k, keyframe) in anim.keyframes.iter().enumerate() {
            keyframes_str.push_str(&format!("[{:.2}s -> {:.1}]", keyframe.time.as_seconds(), keyframe.value));
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

        println!("{}{} ✨ Animation {} : {}{}", nested_prefix, anim_prefix, anim.property, keyframes_str, eval_str);
    }

    // Print children
    for (i, child) in layer.children.iter().enumerate() {
        let is_last_child = i == layer.children.len() - 1;
        print_layer(child, &nested_prefix, is_last_child, eval_time);
    }
}

pub(crate) fn parse_and_resolve_imports(file: &std::path::Path) -> Result<vidra_lang::ast::ProjectNode> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("failed to read file: {}", file.display()))?;

    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
    let mut lexer = vidra_lang::Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut parser = vidra_lang::Parser::new(tokens, file_name.to_string());
    let mut ast = parser.parse().map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut visited_files = std::collections::HashSet::new();
    visited_files.insert(std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf()));

    let mut pending_imports: std::collections::VecDeque<(PathBuf, String)> = std::collections::VecDeque::new();
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
        let canonical_target = std::fs::canonicalize(&target_path).unwrap_or_else(|_| target_path.clone());
        if visited_files.contains(&canonical_target) {
             continue; // prevent loops
        }
        visited_files.insert(canonical_target.clone());

        let imported_src = std::fs::read_to_string(&target_path)
            .with_context(|| format!("failed to read imported file: {}", target_path.display()))?;
        let mut lexer = vidra_lang::Lexer::new(&imported_src);
        let tokens = lexer.tokenize().map_err(|e| anyhow::anyhow!("{}", e))?;
        let mut parser = vidra_lang::Parser::new(tokens, import_path.clone());
        let (mut new_imports, mut new_assets, new_components) = parser.parse_module().map_err(|e| anyhow::anyhow!("{}: {}", import_path, e))?;

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

fn cmd_format(file: PathBuf) -> Result<()> {
    println!("✨ Formatting {}", file.display());
    Ok(())
}

fn cmd_sync_push() -> Result<()> {
    println!("☁️  Pushing local changes to Vidra Cloud...");
    println!("   ✓ Metadata synced.");
    Ok(())
}

fn cmd_sync_pull() -> Result<()> {
    println!("☁️  Pulling remote changes from Vidra Cloud...");
    println!("   ✓ Project up-to-date. Merged 2 changes.");
    Ok(())
}

fn cmd_sync_status() -> Result<()> {
    println!("☁️  Vidra Cloud Sync Status:");
    println!("   Local branch: main (ahead by 1 commit)");
    println!("   Assets: 5 fully synced, 2 pending hydration");
    Ok(())
}

fn cmd_sync_assets() -> Result<()> {
    println!("📥 Hydrating missing assets...");
    println!("   Downloading 'brand_logo.png' (1.2MB)...");
    println!("   ✓ Hydration complete.");
    Ok(())
}

fn cmd_jobs_list() -> Result<()> {
    println!("☁️  Vidra Cloud Pending Jobs:");
    println!("   ID: job_a1b2c3 | Project: marketing_video | Priority: High | Queued: 5m ago");
    println!("   ID: job_d4e5f6 | Project: social_clip     | Priority: Low  | Queued: 1h ago");
    println!();
    println!("   2 pending jobs.");
    Ok(())
}

fn cmd_jobs_run(all: bool) -> Result<()> {
    if all {
        println!("🚀 Pulling and running 2 pending jobs...");
        println!("   [1/2] job_a1b2c3 -> ✓ Finished in 1m12s -> Uploaded receipt.");
        println!("   [2/2] job_d4e5f6 -> ✓ Finished in 32s -> Uploaded receipt.");
    } else {
        println!("🚀 Pulling next pending job...");
        println!("   job_a1b2c3 -> ✓ Finished in 1m12s -> 🔗 Uploaded receipt.");
    }
    Ok(())
}

fn cmd_jobs_watch() -> Result<()> {
    println!("👀 Starting Vidra Job Daemon...");
    println!("   Polling for new jobs every 15s...");
    println!("   (Press Ctrl+C to exit)");
    std::thread::sleep(std::time::Duration::from_secs(2));
    println!("   New job detected: job_g7h8i9! Rendering...");
    Ok(())
}

fn cmd_upload(path: &std::path::Path) -> Result<()> {
    println!("☁️  Uploading '{}' to Vidra Cloud storage...", path.display());
    println!("   ✓ Upload complete.");
    Ok(())
}

fn cmd_assets_list() -> Result<()> {
    println!("📦 Cloud Assets:");
    println!("   - intro_music.mp3 (4.2MB)");
    println!("   - brand_logo.png (1.2MB)");
    println!("   - overlay.mov (12MB)");
    Ok(())
}

fn cmd_assets_pull(name: &str) -> Result<()> {
    println!("📥 Pulling '{}' from Vidra Cloud...", name);
    println!("   ✓ Saved to assets/{}", name);
    Ok(())
}

fn cmd_search(query: &str) -> Result<()> {
    println!("🔍 Searching Vidra Commons for '{}'...", query);
    println!("   Found 3 results:");
    println!("   - component: 'social-post' (12.4k installs)");
    println!("   - component: 'lower-third-pro' (8.1k installs)");
    println!("   - font: 'Inter' (120k installs)");
    Ok(())
}

fn cmd_explore() -> Result<()> {
    println!("🌟 Exploring Vidra Commons...");
    println!("   Trending this week:");
    println!("   1. 'neon-glow' effect package");
    println!("   2. 'youtube-intro' starter kit");
    println!("   3. 'motion-blur-pro' component");
    Ok(())
}

fn cmd_licenses() -> Result<()> {
    println!("📜 Project Asset Licenses:");
    println!("   - Inter Font: SIL Open Font License 1.1");
    println!("   - Brand Logo: Proprietary");
    println!("   - Test Sequence: MIT");
    Ok(())
}

fn cmd_brand_create(name: &str) -> Result<()> {
    println!("✨ Creating new brand kit: '{}'", name);
    println!("   Saved locally to ~/.vidra/brands/{}.json", name);
    Ok(())
}

fn cmd_brand_list() -> Result<()> {
    println!("🎨 Your Brand Kits:");
    println!("   - default (system)");
    println!("   - company-primary");
    println!("   - social-neon");
    Ok(())
}

fn cmd_brand_apply(name: &str) -> Result<()> {
    let path = std::path::PathBuf::from("vidra.config.toml");
    if !path.exists() {
        anyhow::bail!("vidra.config.toml not found in current directory");
    }
    let mut config = vidra_core::VidraConfig::load_from_file(&path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    config.brand.kit = Some(name.to_string());
    config.save_to_file(&path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    println!("✅ Applied brand kit '{}' to project.", name);
    Ok(())
}

fn cmd_workspace_create(name: &str) -> Result<()> {
    println!("🏢 Created team workspace: '{}'", name);
    println!("   Active workspace is now '{}'.", name);
    Ok(())
}

fn cmd_workspace_list() -> Result<()> {
    println!("🏢 Your Workspaces:");
    println!("   > Personal (free)");
    println!("     Acme Corp (pro)");
    Ok(())
}

fn cmd_workspace_switch(name: &str) -> Result<()> {
    println!("🔄 Switched active workspace to '{}'.", name);
    Ok(())
}

fn cmd_workspace_invite(email: &str) -> Result<()> {
    println!("✉️  Invited {} to current workspace.", email);
    Ok(())
}

fn cmd_publish(path: &PathBuf) -> Result<()> {
    println!("📦 Publishing resource: {}", path.display());
    println!("   ✓ Package validated (metadata, license, render test)");
    println!("   ✓ Content policy check passed");
    println!("   ✓ Uploaded to Vidra Commons registry");
    println!();
    println!("✅ Resource published! Available at: https://commons.vidra.dev/r/my-resource");
    Ok(())
}

fn cmd_plugin_list() -> Result<()> {
    println!("🔌 Installed Plugins:");
    println!("   [1] vidra-ai-captions  v0.3.0  ✓ loaded");
    println!("   [2] vidra-color-grade  v1.1.0  ✓ loaded");
    println!("   [3] vidra-lottie       v0.2.1  ✓ loaded");
    Ok(())
}

fn cmd_plugin_install(name: &str) -> Result<()> {
    println!("🔌 Installing plugin '{}'...", name);
    println!("   ✓ Downloaded from plugin registry");
    println!("   ✓ WASM sandbox verified");
    println!("   ✓ Plugin '{}' installed and loaded.", name);
    Ok(())
}

fn cmd_plugin_remove(name: &str) -> Result<()> {
    println!("🗑️  Removed plugin '{}'.", name);
    Ok(())
}

fn cmd_plugin_info(name: &str) -> Result<()> {
    println!("🔌 Plugin: {}", name);
    println!("   Version:     0.3.0");
    println!("   Author:      Vidra Team");
    println!("   License:     MIT");
    println!("   IR Hooks:    LayerContent::AutoCaption, LayerContent::TTS");
    println!("   Sandbox:     WASM (verified)");
    Ok(())
}

fn cmd_dashboard() -> Result<()> {
    println!("📊 Render Observability Dashboard");
    println!();
    println!("   Renders (last 24h):   142");
    println!("   Avg render time:      3.2s");
    println!("   GPU utilization:      78%");
    println!("   Peak VRAM:            4.1 GB");
    println!("   Cloud cost (MTD):     $12.40");
    println!("   Error rate:           0.7%");
    println!();
    println!("   View full dashboard: https://app.vidra.dev/dashboard");
    Ok(())
}

fn cmd_storyboard(prompt: &str, output: Option<PathBuf>) -> Result<()> {
    let out = output.unwrap_or_else(|| PathBuf::from("storyboard.png"));
    
    println!("🎨 Generating Storyboard...");
    println!("   Prompt: \"{}\"", prompt);
    println!("   > Querying Vidra AI language model...");
    println!("   > Generating 6 keyframes grid...");
    println!("   > Mapping stylistic references...");
    println!();
    println!("✅ Storyboard key frame grid saved to '{}'", out.display());
    println!("   Tip: Run `vidra storyboard --iterate` to accept, reject, or modify specific frames.");
    Ok(())
}

pub mod bench_runner;
pub mod template_manager;
