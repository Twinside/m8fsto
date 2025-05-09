use std::{fmt::Display, path::PathBuf};

/// Some standard flags used acrross various commands.
#[derive(Clone, Copy)]
pub struct FlagBag {
    /// We do not want to write anything, only performing
    /// a test run/scan of a given command
    pub dry_run: bool,

    /// Print more information to understand errors
    pub verbose: bool,

    /// Perform writing/updates even if some errors
    /// have been detected
    pub force: bool,

}

#[derive(Debug, Clone)]
pub enum M8FstoErr {
    UnparseableM8File { path: PathBuf, reason: String },
    InvalidSearchPattern { pattern: String },
    CannotReadFile { path: PathBuf, reason: String },
    SampleCopyError { path: PathBuf, to: PathBuf, reason: String },
    SongSerializationError { destination: String, reason: String },
    MissingSample { instr: usize, path: PathBuf },
    MultiErrs { inner: Vec<M8FstoErr> },
    FolderCreationError { path: PathBuf, reason: String },
    SampleInBundleNotRelative {
        sample_path: String,
        instrument: usize
    },
    FileRemovalFailure { path: PathBuf, reason: String },
    InvalidPath { reason: String },
    RenameFailure { path: String }
}

impl M8FstoErr {
    /// Combine multiple errors together, maintaining a canonical
    /// form.
    pub fn combine(self, other: M8FstoErr) -> Self {
        match self {
            M8FstoErr::MultiErrs { mut inner} => {
                match other {
                    M8FstoErr::MultiErrs { inner: mut other_inner} =>
                        inner.append(&mut other_inner),
                    other => inner.push(other)
                }

                Self::MultiErrs { inner }
            }
            _ => {
                Self::MultiErrs { inner: vec![self, other] }
            }
        }
    }
}

pub fn combine(err: Option<M8FstoErr>, other: M8FstoErr) -> Option<M8FstoErr> {
    match err {
        None => Some(other),
        Some(org) => Some(org.combine(other))
    }
}

impl Display for M8FstoErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            M8FstoErr::UnparseableM8File { path, reason } => {
                writeln!(f, "Can't parse M8 file '{:?}' : {}", path.as_path(), reason)
            }
            M8FstoErr::InvalidSearchPattern { pattern } => {
                writeln!(f, "Invalid search pattern '{}'", pattern)
            },
            M8FstoErr::CannotReadFile { path, reason } => {
                writeln!(f, "Cannot read file '{:?}' : {}", path, reason)
            },
            M8FstoErr::MultiErrs { inner } => {
                for i in inner.iter() {
                    i.fmt(f)?
                }
                Ok(())
            }
            M8FstoErr::MissingSample { instr, path } => {
                writeln!(f, "Missing sample '{:?}' for instrument {:02X}", path, instr)
            }
            M8FstoErr::SampleCopyError { path, to , reason } => {
                writeln!(f, "Cannot copy file '{:?}' to '{:?}' : {}", path, to, reason)
            }
            M8FstoErr::SongSerializationError { destination, reason } => {
                writeln!(f, "Error while writing song \"{}\": {}", destination, reason)
            }
            M8FstoErr::SampleInBundleNotRelative { sample_path, instrument } => {
                writeln!(f, "The M8 song has non-relative sample \"{}\" for instrument {:02X}", sample_path, instrument)
            }
            M8FstoErr::FolderCreationError { path, reason} => {
                writeln!(f, "Cannot create folder '{:?}' for bundling : {}", path, reason)
            }
            M8FstoErr::FileRemovalFailure { path, reason} => {
                writeln!(f, "Cannot remove file {:?} : '{}'", path, reason)
            }
            M8FstoErr::InvalidPath { reason }=> {
                writeln!(f, "Invalid path {}", reason)
            }
            M8FstoErr::RenameFailure { path } => {
                writeln!(f, "Cannot rename file or folder \"{:?}\"", path)
            }
        }
    }
}