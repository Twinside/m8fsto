use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::path::Component;
use glob::glob;
use m8_files::{reader::*, Instrument};

use crate::types::combine;
use crate::types::FlagBag;
use crate::types::M8FstoErr;

enum Swap {
    Dir { from: String, to: String },
    File { from: String, to: String }
}

impl Swap {
    pub fn try_swap(&self, sample_path: &str) -> Option<String> {
        match self {
            Swap::File { from, to } if sample_path == from =>
                Some(to.clone()),
            Swap::File { from: _, to: _} => None,
            Swap::Dir { from, to } => {
                if !sample_path.starts_with(from) {
                    return None
                }
                let final_path =
                    sample_path.strip_prefix(from).unwrap();
                Some(format!("{}{}", to, final_path))
            }
        }
    }
}

/// Keep track of a modified sample instrument
pub struct SwappedInstruments {
    /// Instrument number in the file
    pub instrument: usize,

    /// Original instrument name
    pub instrument_name: String,

    /// Original sample path (useful to get when
    /// moving a whole sample folder)
    pub original_sample_path: String,

    /// Final sample path after replacement.
    pub new_sample_path: String
}

impl SwappedInstruments {
    fn print(&self) {
        println!(
            " - {} {} \"{}\" -> \"{}\"",
            self.instrument,
            self.instrument_name,
            self.original_sample_path,
            self.new_sample_path)
    }
}

/// Result of song file modification
struct SwappedFile {
    /// Song data ready to be written on disk
    file_data : Vec<u8>,
    /// List of updated instruments
    touched: Vec<SwappedInstruments>
}

fn on_file_blob(flags: &FlagBag, swap: &Swap, path: &Path, data: Vec<u8>) -> Result<Option<SwappedFile>, M8FstoErr> {
    let mut reader = Reader::new(data.clone());
    let mut touched = vec![];
    let mut song = m8_files::Song::read_from_reader(&mut reader)
        .map_err(|e| M8FstoErr::UnparseableM8File {
            path: path.to_path_buf(),
            reason: format!("{:?}", e)
        })?;

    for (instrument, instr) in song.instruments.iter_mut().enumerate() {
        match instr {
            Instrument::Sampler(sampler) => {
                match swap.try_swap(&sampler.sample_path) {
                    None => {}
                    Some(new_path) => {
                        touched.push(SwappedInstruments {
                            instrument,
                            instrument_name: sampler.name.clone(),
                            original_sample_path: sampler.sample_path.clone(),
                            new_sample_path: new_path.clone()
                        });
                        sampler.sample_path = new_path;
                    }
                }
            }
            _ => {}
        }
    }

    if touched.len() == 0 { return Ok(None);}

    if flags.dry_run {
        return Ok(Some(SwappedFile {
            file_data: Vec::new(),
            touched
        }))
    }

    let mut writer =
        m8_files::writer::Writer::new(data);

    song.write(&mut writer)
        .map_err(|reason|
            M8FstoErr::SongSerializationError {
                reason,
                destination: format!("{:?}", path)
            })?;

    Ok(Some(SwappedFile {
        file_data: writer.finish(),
        touched
    }))
}

fn on_dir(flags: &FlagBag, cwd: &Path, swap: &Swap) -> Result<(), M8FstoErr> {
    let mut errors = None;
    let mut matched_not_serializable = vec![];
    let mut to_write= vec![];
    let search_pattern =
        glob(&format!("{}/**/*.m8s", cwd.to_str().unwrap()))
        .expect("Failed to read glob pattern");

    for entry in search_pattern {
        match entry {
            Err(e) => println!("{:?}", e),
            Ok(path) => {
                let try_as_file = fs::read(&path);
                match try_as_file {
                    Err(e) => {
                        errors = combine(errors, M8FstoErr::CannotReadFile {
                            path: path.to_path_buf(),
                            reason: format!("{:?}", e)
                        })
                    }
                    Ok(file_blob) => {
                        match on_file_blob(flags, swap, &path, file_blob) {
                            Ok(None) => {}
                            Err(M8FstoErr::SongSerializationError { reason: _, destination}) =>
                                matched_not_serializable.push(destination),
                            Err(m8err) =>
                                errors = combine(errors, m8err),
                            Ok(Some(swapped)) => {
                                for touched in swapped.touched {
                                    touched.print()
                                }

                                to_write.push((path, swapped.file_data));
                            }
                        }
                    }
                }
            }
        }
    }

    // If we have some file we can't translate, but still want to write
    // the files
    if !flags.dry_run && (matched_not_serializable.len() == 0 || flags.force) {
        for (path, data) in to_write {
            match fs::write(&path, data) {
                Ok(()) => {}
                Err(_) => {
                    errors = combine(errors, 
                        M8FstoErr::SongSerializationError {
                            reason: "Error while writing file".into(),
                            destination: format!("{:?}", path)
                        });
                }
            }
        }
    }

    match errors {
        None => Ok(()),
        Some(errs) => Err(errs)
    }
}


pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}


pub fn move_samples(
    cwd: &Path,
    flags: FlagBag,
    from: String,
    to: String) -> Result<(), M8FstoErr> {

    let cwd = normalize_path(cwd);

    if flags.verbose {
        println!("Using backup at location: {:?}", cwd);
    }

    let from_path = PathBuf::from(from);
    if !from_path.exists() {
        return Err(M8FstoErr::InvalidPath { reason: format!("Folder {:?} doesn't exists", from_path) })
    }

    let to_path = PathBuf::from(to);
    let to_canon =
        if to_path.is_relative() {
            let mut abs = cwd.clone();
            abs.push(to_path);
            abs
        } else {
            to_path
        };

    let from_canon = normalize_path(&from_path);
    
    if flags.verbose {
        println!(" * moving source {:?}", from_canon);
    }

    let rel_from =
        from_canon.strip_prefix(&cwd)
            .map_err(| _| M8FstoErr::InvalidPath { reason: "from relativisation error".into() })?
            .to_str()
            .unwrap()
            .replace("\\","/");

    if flags.verbose {
        println!(" * to {:?}", to_canon);
    }

    let rel_to =
        to_canon.strip_prefix(&cwd)
            .map_err(| _| M8FstoErr::InvalidPath { reason: "destination canonicalization error".into() })?
            .to_str()
            .unwrap()
            .replace("\\","/");

    let move_order =
        if from_path.is_dir() {
            Swap::Dir {
                from: format!("/{}", rel_from),
                to: format!("/{}", rel_to),
            }
        } else if from_path.is_file() {
            Swap::File {
                from: format!("/{}", rel_from),
                to: format!("/{}", rel_to),
            }
        } else {
            return Err(M8FstoErr::CannotReadFile { path: from_path, reason: String::from("Neither file nor directory")})
        };

    match on_dir(&flags, &cwd, &move_order) {
        Ok(()) => {
            std::fs::rename(&from_canon, to_canon)
                .map_err(|_| M8FstoErr::RenameFailure { path: format!("{:?}", from_canon) })
        }
        Err(errs) if flags.force => {
            match std::fs::rename(&from_canon, to_canon) {
                Ok(()) => Err(errs),
                Err(_) => {
                    let path = format!("{:?}", from_canon);
                    Err(errs.combine(M8FstoErr::RenameFailure { path }))
                }
            }
        }
        Err(errs) => Err(errs)
    }
}
