use std::{io::stdout, path::PathBuf};

use clap::{Parser, Subcommand};
use clap_num::maybe_hex;
use types::{FlagBag, M8FstoErr};

use crate::renumber::renumber_element;

mod ls_sample;
mod grep_sample;
mod bundle;
mod prune_bundle;
mod broken_search;
mod types;
mod show_song;
mod move_samples;
mod renumber;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
/// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<M8Commands>
}

/// What do we want to print, prefix with 0x to use hexadecimal notation.
#[derive(Subcommand)]
enum ShowTarget {
    /// Print the whole song view
    Song,

    /// Display various information about the song like version,
    /// number of instruments, number of used samples
    Info,

    /// Display mixer settings
    Mixer,

    /// Display effect settings
    Effects,

    /// Print the content of a chain
    Chain {
        #[clap(value_parser=maybe_hex::<usize>)]
        id: usize
    },

    /// Print the content of a phrase
    Phrase {
        #[clap(value_parser=maybe_hex::<usize>)]
        id: usize
    },

    /// Print the content of a phrase
    Instrument {
        #[clap(value_parser=maybe_hex::<usize>)]
        id: Option<usize>
    },

    /// Print the content of a table
    Table {
        #[clap(value_parser=maybe_hex::<usize>)]
        id: Option<usize>
    },

    /// Print EQ information
    Eq {
        #[clap(value_parser=maybe_hex::<usize>)]
        id: Option<usize>
    },
}

#[derive(Parser)]
struct ShowCommand {
    #[structopt(subcommand)]
    pub show_command: ShowTarget,

    /// File to display
    pub file: String
}

/// What do we want to print, prefix with 0x to use hexadecimal notation.
#[derive(Subcommand)]
enum RenumberTarget {
    /// Print the content of a phrase
    Instrument {
        #[clap(value_parser=maybe_hex::<usize>)]
        from: usize,

        #[clap(value_parser=maybe_hex::<usize>)]
        to: usize,
    },
}

#[derive(Parser)]
struct RenumberCommand {
    #[structopt(subcommand)]
    pub renum_command: RenumberTarget,

    /// File to display
    pub file: String
}

#[derive(Subcommand)]
enum M8Commands {
    /// Show an element of the song
    Show(ShowCommand),

    /// Renumber an element of the M8
    // Renumber(RenumberCommand),

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

    /// Try to find broken sample paths in songs or directories
    BrokenSearch {
        /// When searching direct song, which root do we use?
        root: Option<String>,

        /// Optional paths to process: directories or `.m8s` song files.
        /// If not set, the current working directory is used.
        paths: Vec<String>,
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
        /*
        Some(M8Commands::Renumber(recommand)) => {
            print_errors(renumber::renumber_element(recommand, &mut stdout()));
        } */
        Some(M8Commands::Show(showcmd)) => {
            print_errors(show_song::show_element(showcmd, &mut stdout()));
        }
        Some(M8Commands::LsSample { path }) => {
            print_errors(ls_sample::ls_sample(cwd.as_path(), &path))
        }
        Some(M8Commands::GrepSample { pattern, path }) => {
            print_errors(grep_sample::grep_sample(cwd.as_path(), &pattern, &path))
        }
        Some(M8Commands::BrokenSearch { root, paths }) => {
            let root =
                root.map_or_else(|| cwd.clone(), |e| PathBuf::from(e));
            print_errors(broken_search::process_paths(&root, &paths))
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
