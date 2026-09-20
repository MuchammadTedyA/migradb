use clap::{Parser, Subcommand};
use migration_engine::{diff_drawdb, generate_models, InMemoryTracker, MigratorBuilder, TargetLanguage};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "migradb")]
#[command(author = "Muchammad Tedy")]
#[command(version = "0.1.1")]
#[command(about = "High-performance SQL database migration and model generation engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new migrations directory
    Init {
        /// Path to the migrations directory
        #[arg(short, long, default_value = "./migrations")]
        dir: PathBuf,
    },

    /// Create a new timestamped SQL migration file
    Create {
        /// Name of the migration (e.g. add_users_table)
        name: String,

        /// Path to the migrations directory
        #[arg(short, long, default_value = "./migrations")]
        dir: PathBuf,
    },

    /// Show migration status (applied vs pending)
    Status {
        /// Path to the migrations directory
        #[arg(short, long, default_value = "./migrations")]
        dir: PathBuf,
    },

    /// Generate entity models or SQL DDL from a DrawDB schema
    Generate {
        /// Path to DrawDB JSON schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Target language (go, node, python, csharp, rust, sql)
        #[arg(short, long)]
        lang: String,

        /// Output directory for generated code
        #[arg(short, long, default_value = "./models")]
        out: PathBuf,

        /// Optional package name, namespace, or SQL dialect (e.g. postgres, mysql, sqlite)
        #[arg(short, long)]
        pkg: Option<String>,
    },

    /// Calculate schema diff against snapshot and generate an incremental SQL migration
    Diff {
        /// Path to DrawDB JSON schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Migration description name (e.g. add_user_avatar)
        #[arg(short, long, default_value = "sync_drawdb")]
        name: String,

        /// Path to the migrations directory
        #[arg(short, long, default_value = "./migrations")]
        dir: PathBuf,

        /// Path to the schema directory for snapshots
        #[arg(long, default_value = "./schema")]
        schema_dir: PathBuf,

        /// SQL dialect: postgres (default), mysql, or sqlite
        #[arg(long, default_value = "postgres")]
        dialect: String,

        /// Force generating a full baseline schema dump even if a snapshot exists
        #[arg(long)]
        full: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { dir } => {
            if let Err(e) = handle_init(&dir) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Create { name, dir } => {
            if let Err(e) = handle_create(&name, &dir) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Status { dir } => {
            if let Err(e) = handle_status(&dir) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Generate {
            schema,
            lang,
            out,
            pkg,
        } => {
            if let Err(e) = handle_generate(&schema, &lang, &out, pkg.as_deref()) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Diff {
            schema,
            name,
            dir,
            schema_dir,
            dialect,
            full,
        } => {
            if let Err(e) = handle_diff(&schema, &name, &dir, &schema_dir, &dialect, full) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn handle_init(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
        println!("Created directory: {}", dir.display());
    } else {
        println!("Directory already exists: {}", dir.display());
    }

    let initial_file = dir.join("20260101000000_initial_schema.sql");
    if !initial_file.exists() {
        let content = "-- Initial schema migration\n-- Up\nCREATE TABLE IF NOT EXISTS users (\n    id SERIAL PRIMARY KEY,\n    name VARCHAR(255) NOT NULL,\n    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP\n);\n\n-- Down\nDROP TABLE IF EXISTS users;\n";
        fs::write(&initial_file, content)?;
        println!("Created sample migration: {}", initial_file.display());
    }

    println!("\nInitialized MigraDB in {}", dir.display());
    Ok(())
}

fn handle_create(name: &str, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tracker = Arc::new(InMemoryTracker::new());
    let migrator = MigratorBuilder::new()
        .migrations_dir(dir.to_path_buf())
        .build(tracker);

    let template = format!("-- Migration: {}\n-- Up\n\n-- Down\n", name);
    let path = migrator.create_migration(name, &template)?;

    println!("Created migration: {}", path.display());
    Ok(())
}

fn handle_status(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        println!("Migrations directory `{}` does not exist.", dir.display());
        return Ok(());
    }

    let tracker = Arc::new(InMemoryTracker::new());
    let migrator = MigratorBuilder::new()
        .migrations_dir(dir.to_path_buf())
        .build(tracker);

    let status_list = migrator.status()?;

    println!("\nMigration Status ({}):", dir.display());
    println!("{:-<70}", "");
    println!("{:<12} {:<18} {}", "STATUS", "VERSION", "NAME");
    println!("{:-<70}", "");

    if status_list.is_empty() {
        println!("(No migration files found in {})", dir.display());
    } else {
        for s in status_list {
            let label = if s.applied { "[APPLIED]" } else { "[PENDING]" };
            println!("{:<12} {:<18} {}", label, s.version, s.name);
        }
    }
    println!("{:-<70}\n", "");

    Ok(())
}

fn handle_generate(
    schema: &Path,
    lang_str: &str,
    out: &Path,
    pkg: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let target_lang = TargetLanguage::parse(lang_str)?;
    let written = generate_models(schema, target_lang, out, pkg)?;

    println!(
        "\nSuccessfully generated {} files in `{}` (target: {}):",
        written.len(),
        out.display(),
        lang_str
    );
    for file in &written {
        println!("  - {}", file.display());
    }
    println!();
    Ok(())
}

fn handle_diff(
    schema: &Path,
    name: &str,
    dir: &Path,
    schema_dir: &Path,
    dialect: &str,
    full: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let res = diff_drawdb(schema, dir, schema_dir, name, Some(dialect), full)?;

    if res.is_empty {
        println!("\n{}", res.diff_summary);
        println!("No migration file was created.\n");
        return Ok(());
    }

    println!("\n{}\n", res.diff_summary);
    if let Some(p) = res.migration_path {
        println!("Created incremental migration ({} dialect):", dialect);
        println!("  - {}", p);
    }
    println!("Updated schema snapshots:");
    println!("  - {}/drawdb_snapshot.json", schema_dir.display());
    println!("  - {}/.schema_snapshot.json\n", dir.display());

    Ok(())
}

