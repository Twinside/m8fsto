use std::{fs, path::PathBuf};
use diffy::{PatchFormatter, create_patch};
use m8_file_parser::reader::Reader;
use m8_file_parser::Song;

use crate::show_song::ElemDisplay;
use crate::types::M8FstoErr;

fn print_patch(w: &mut dyn std::io::Write, title: &str, str_a: &str, str_b: &str) -> Result<(), std::io::Error> {
    writeln!(w, "{}", title)?;

    let patch = create_patch(str_a, str_b);
    let formatter = PatchFormatter::new().with_color();
    writeln!(w, "{}", formatter.fmt_patch(&patch))?;
    Ok(())
}

pub fn diff_song(w: &mut dyn std::io::Write, song_a: &Song, song_b: &Song) -> Result<(), M8FstoErr> {
    for (i, (inst_a, inst_b)) in song_a.instruments.iter().zip(song_b.instruments.iter()).enumerate() {
        if inst_a == inst_b { continue; }

        let instr_a_txt = format!("{}", ElemDisplay {
            instr: inst_a.clone(),
            ver: song_a.version
        });

        let instr_b_txt = format!("{}", ElemDisplay {
            instr: inst_b.clone(),
            ver: song_b.version
        });

        let title= format!("=== Instrument 0x{:02X}", i);
        print_patch(w, &title, &instr_a_txt, &instr_b_txt)
            .map_err(|_| M8FstoErr::PrintError)?;
    }

    for (i, (table_a, table_b)) in song_a.tables.iter().zip(song_b.tables.iter()).enumerate() {
        if table_a == table_b { continue; }

        let table_a_txt = format!("{}", song_a.table_view(i));
        let table_b_txt = format!("{}", song_b.table_view(i));

        let title= format!("=== Table 0x{:02X}", i);
        print_patch(w, &title, &table_a_txt, &table_b_txt)
            .map_err(|_| M8FstoErr::PrintError)?;
    }

    for (i, (eq_a, eq_b)) in song_a.eqs.iter().zip(song_b.eqs.iter()).enumerate() {
        if eq_a == eq_b { continue; }

        let eq_a_txt = format!("{}", ElemDisplay {
            instr: eq_a.clone(),
            ver: song_a.version
        });

        let eq_b_txt = format!("{}", ElemDisplay {
            instr: eq_b.clone(),
            ver: song_b.version
        });

        let title= format!("=== Eq 0x{:02X}", i);
        print_patch(w, &title, &eq_a_txt, &eq_b_txt)
            .map_err(|_| M8FstoErr::PrintError)?;
    }

    for (i, (chain_a, chain_b)) in song_a.chains.iter().zip(song_b.chains.iter()).enumerate() {
        if chain_a == chain_b { continue; }

        let chain_a_txt = format!("{}", chain_a);
        let chain_b_txt = format!("{}", chain_b);

        let title= format!("=== Chain 0x{:02X}", i);
        print_patch(w, &title, &chain_a_txt, &chain_b_txt)
            .map_err(|_| M8FstoErr::PrintError)?;
    }

    for (i, (phrase_a, phrase_b)) in song_a.phrases.iter().zip(song_b.phrases.iter()).enumerate() {
        if phrase_a == phrase_b { continue; }

        let chain_a_txt = format!("{}", song_a.phrase_view(i));
        let chain_b_txt = format!("{}", song_b.phrase_view(i));

        let title= format!("=== Phrase 0x{:02X}", i);
        print_patch(w, &title, &chain_a_txt, &chain_b_txt)
            .map_err(|_| M8FstoErr::PrintError)?;
    }

    if song_a.song != song_b.song {
        let song_a_txt = format!("{}", &song_a.song);
        let song_b_txt = format!("{}", &song_b.song);

        let title= "=== Song";
        print_patch(w, title, &song_a_txt, &song_b_txt)
            .map_err(|_| M8FstoErr::PrintError)?;
    }

    Ok(())
}

pub fn diff(song_a: &PathBuf, song_b: &PathBuf, write: &mut dyn std::io::Write) -> Result<(), M8FstoErr> {

    let file_blob_a = fs::read(song_a)
        .map_err(|e|
            M8FstoErr::CannotReadFile { path: PathBuf::from(song_a), reason: format!("{:?}", e) })?;

    let file_blob_b = fs::read(song_b)
        .map_err(|e|
            M8FstoErr::CannotReadFile { path: PathBuf::from(song_b), reason: format!("{:?}", e) })?;

    let mut reader_a = Reader::new(file_blob_a);
    let song_a = m8_file_parser::Song::read_from_reader(&mut reader_a)
        .map_err(|e| M8FstoErr::UnparseableM8File {
            path: song_a.to_path_buf(),
            reason: format!("{:?}", e)
        })?;

    let mut reader_b = Reader::new(file_blob_b);
    let song_b = m8_file_parser::Song::read_from_reader(&mut reader_b)
        .map_err(|e| M8FstoErr::UnparseableM8File {
            path: song_b.to_path_buf(),
            reason: format!("{:?}", e)
        })?;

    diff_song(write, &song_a, &song_b)?;

    Ok(())
}