use clap::{Parser, Subcommand};
use jot::{issues, plan, scan};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "0jot")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan a repo and write the create/close plan as actions.json.
    Plan {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        issues: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Plan { repo, issues: issues_path, out } => {
            let found = scan::scan(&repo);
            let existing =
                issues::parse(&fs::read_to_string(&issues_path).expect("read issues.json"));
            let actions = plan::plan(&found, &existing);
            fs::write(&out, serde_json::to_string_pretty(&actions).unwrap())
                .expect("write actions.json");
        }
    }
}
