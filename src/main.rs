use std::path::PathBuf;

use clap::{Parser, Subcommand};
use types::{FlagBag, M8FstoErr};

mod ls_sample;
mod grep_sample;
mod bundle;
mod prune_bundle;
mod broken_search;
mod types;
mod move_samples;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
/// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<M8Commands>
}

#[derive(Subcommand)]
enum M8Commands {
    /// List samples used in M8 song file
    LsSample {
        /// Optional path/folder
        path: Option<String>
    },

    /// Try to find songs that are using a given sample
    GrepSample {
        /// Pattern to search, representing a sample file path using
        /// glob patterns
        pattern : String,

        /// In which folder to search
        path : Option<String>
    },

    /// Bundle a song, avoiding sample duplication
    Bundle {
        /// Specific song only
        song : String,

        /// Root folder for the sample path.
        root : Option<String>,

        /// Where to write the bundled song, by default
        /// will be in the root directory "Bundle" subfolder.
        out_folder: Option<String>
    },

    /// Given a bundled song, remove all local samples
    /// that are not used within the bundled song.
    PruneBundle {
        /// If set to true, it will list the sample to be removed
        /// and don't do anything. Should be used first.
        #[arg(short, long)]
        dry_run : bool,

        /// Specific song only
        song : String
    },

    /// Try to find broken sample path in songs from a given root
    BrokenSearch {
        /// Optional root folder for the sample path, if not
        /// set, current working directory is used.
        root : Option<String>
    },

    /// Move a sample or sample folder and update songs referencing
    /// them.
    Mv {
        /// If set, it will list the sample to be moved
        /// and the list of modified songs & instruments
        #[arg(short, long)]
        dry_run : bool,

        /// If set, it will file will be written even if some
        /// songs cannot be rewritten (like in 3.x format)
        #[arg(short, long)]
        force : bool,

        /// Optional root folder for the sample path, if not
        /// set, current working directory is used.
        #[arg(short, long)]
        root: Option<String>,

        /// Source folder or sample
        from: String,

        /// Destination
        to: String
    }
}

fn print_errors(r : Result<(), M8FstoErr>) {
    match r {
        Ok(()) => {}
        Err(e) => eprintln!("{}", e)
    }
}

fn main() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap();

    match cli.command {
        None => { println!("Please use a command") }
        Some(M8Commands::LsSample { path }) => {
            print_errors(ls_sample::ls_sample(cwd.as_path(), &path))
        }
        Some(M8Commands::GrepSample { pattern, path }) => {
            print_errors(grep_sample::grep_sample(cwd.as_path(), &pattern, &path))
        }
        Some(M8Commands::BrokenSearch { root }) => {
            let root = root
                .map_or_else(
                    || cwd.as_path().to_path_buf(),
                     |f| PathBuf::from(f));

            print_errors(broken_search::find_broken_sample(root.as_path()))
        }
        Some(M8Commands::Bundle { song, root, out_folder }) => {
            let root =
                root.map_or_else(|| cwd.clone(), |e| PathBuf::from(e));

            print_errors(bundle::bundle_song(root.as_path(), &song, &out_folder))
        }
        Some(M8Commands::PruneBundle { dry_run, song}) => {
            let flags = FlagBag {
                dry_run,
                force: false,
                verbose: false
            };

            print_errors(prune_bundle::prune_bundle(flags, &song))
        },
        Some(M8Commands::Mv { root, force, dry_run, from, to }) => {
            let root = root
                .map_or_else(
                    || cwd.as_path().to_path_buf(),
                     |f| PathBuf::from(f));

            let flags = FlagBag {
                dry_run,
                force,
                verbose: false
            };

            print_errors(move_samples::move_samples(&root, flags, from, to));
        }
    }
}
