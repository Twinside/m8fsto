use std::{collections::HashSet, fmt::Display, fs, path::PathBuf};

use m8_file_parser::{param_gatherer::{Describable, ParameterGatherer}, reader::Reader, Instrument, Version};

use crate::{types::M8FstoErr, ShowCommand, ShowTarget};

struct AsciiTherer<'a, 'writer> {
    write: &'a mut std::fmt::Formatter<'writer>,
    indent: usize
}

const BLOCKS : [char; 8] = [
    '▏',
    '▎',
    '▌',
    '▍',
    '█',
    '▋',
    '▊',
    '▉'
];

const NAME_LENGTH : usize = 9;

impl<'a, 'writer> ParameterGatherer for AsciiTherer<'a, 'writer> {
    fn hex(self, name: &str, val: u8) -> Self {
        let bar_val = val as usize + 1;

        let full = "▉".repeat(bar_val / 8);
        let middle = BLOCKS[bar_val % 8];
        let after = "░".repeat((256 - bar_val) / 8);

        let _ =
            writeln!(self.write, "{sp:idt$}{name:NAME_LENGTH$}: {val:02X} {full}{middle}{after}",
                sp="",
                idt=self.indent);
        self
    }

    fn bool(self, name: &str, val: bool) -> Self {
        let _ =
            writeln!(self.write, "{sp:idt$}{name:NAME_LENGTH$}: {val}",
                sp="",
                idt=self.indent);
        self
    }

    fn float(self, name: &str, val: f64) -> Self {
        let _ =
            writeln!(self.write, "{sp:idt$}{name:NAME_LENGTH$}: {val}",
                sp="",
                idt= self.indent);
        self
    }

    fn str(self, name: &str, val: &str) -> Self {
        let _ =
            writeln!(self.write, "{sp:idt$}{name:NAME_LENGTH$}: \"{val}\"",
                sp="",
                idt=self.indent);
        self
    }

    fn enumeration(self, name: &str, hex: u8, val: &str) -> Self {
        let _ =
            writeln!(self.write, "{sp:idt$}{name:NAME_LENGTH$}: {hex:X} {val}",
                sp="",
                idt=self.indent);
        self
    }

    fn nest_f<F>(self, name: &str, f: F) -> Self
        where F : FnOnce (Self) -> Self, Self : Sized {

        let _ =
            writeln!(self.write, "{sp:idt$}{name:NAME_LENGTH$}:",
                sp="",
                idt=self.indent);

        let pg = f(Self {
            write: self.write,
            indent: self.indent + 2
        });

        Self {
            write: pg.write,
            indent: self.indent
        }
    }
}

struct ElemDisplay<T> {
    instr: T,
    ver: Version
}

impl<T : Describable> Display for ElemDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = self.instr.describe(AsciiTherer { write: f, indent: 0 }, self.ver);
        Ok(())
    }
}

fn instrument_kind(i: &Instrument) -> &'static str {
    match i {
        Instrument::WavSynth(_) => "WavSynth",
        Instrument::MacroSynth(_) => "MacroSynth",
        Instrument::Sampler(_) => "Sample",
        Instrument::MIDIOut(_) => "MIDIOut",
        Instrument::FMSynth(_) => "FMSynth",
        Instrument::HyperSynth(_) => "HyperSynth",
        Instrument::External(_) => "External",
        Instrument::None => "None"
    }
}

fn show_from_instrument(show: ShowCommand, w: &mut dyn std::io::Write, instr_eq: m8_file_parser::InstrumentWithEq) -> Result<(), M8FstoErr> {
    match show.show_command {
        ShowTarget::Song => Ok(()),
        ShowTarget::Mixer => Ok(()),
        ShowTarget::Effects => Ok(()),
        ShowTarget::Info => {
            writeln!(w, "Version : {}", instr_eq.version).map_err(|_| M8FstoErr::PrintError)?;
            writeln!(w, "Name    : {}", instr_eq.instrument.name().unwrap_or("")).map_err(|_| M8FstoErr::PrintError)?;
            writeln!(w, "Kind    : {}", instrument_kind(&instr_eq.instrument)).map_err(|_| M8FstoErr::PrintError)?;
            Ok(())
        },
        ShowTarget::Chain { id: _ } => Ok(()),
        ShowTarget::Phrase { id: _} => Ok(()),
        ShowTarget::Instrument { id: _ } => {
            write!(w, "{}", ElemDisplay {
                instr: instr_eq.instrument,
                ver: instr_eq.version
            }).map_err(|_| M8FstoErr::PrintError)
        },
        ShowTarget::Table { id: _ } => {
            writeln!(w, "{}", instr_eq.table_view()).map_err(|_| M8FstoErr::PrintError)
        },
        ShowTarget::Eq { id: _ } => {
            match instr_eq.eq {
                None => writeln!(w, "No eq saved in the instrument").map_err(|_| M8FstoErr::PrintError),
                Some(equ) => {
                    write!(w, "{}", ElemDisplay {
                        instr: equ,
                        ver: instr_eq.version
                    }).map_err(|_| M8FstoErr::PrintError)
                }
            }
        },
    }
}

/// Structure used to instantiate Display instance for song info
struct SongInfoDisplay<'a> {
    song: &'a m8_file_parser::Song
}

struct InstrumentCounter {
    pub wavsynth_count : usize,
    pub macrosynth_count : usize,
    pub fm_count : usize,
    pub sampler_count : usize,
    pub midi_count : usize,
    pub external_count : usize,
    pub hypersynth_count: usize,

    pub used_midi_channel : HashSet<u8>
}

impl InstrumentCounter {
    pub fn total(&self) -> usize {
        self.wavsynth_count +
            self.macrosynth_count +
            self.fm_count +
            self.sampler_count +
            self.midi_count +
            self.external_count +
            self.hypersynth_count
    }

    pub fn count(mut self, instr: &m8_file_parser::Instrument) -> Self {
        match instr {
            Instrument::None => (),
            Instrument::WavSynth(_) => self.wavsynth_count += 1,
            Instrument::MacroSynth(_) => self.macrosynth_count += 1,
            Instrument::Sampler(_) => self.sampler_count += 1,
            Instrument::MIDIOut(midiout) => {
                self.midi_count += 1;
                self.used_midi_channel.insert(midiout.channel);
            },
            Instrument::FMSynth(_) => self.fm_count += 1,
            Instrument::HyperSynth(_) => self.hypersynth_count += 1,
            Instrument::External(ext) => {
                self.external_count += 1;
                self.used_midi_channel.insert(ext.channel);
            },
        };

        self
    }
}

impl Default for InstrumentCounter {
    fn default() -> Self {
        Self {
            wavsynth_count: Default::default(),
            macrosynth_count: Default::default(),
            sampler_count: Default::default(),
            midi_count: Default::default(),
            fm_count: Default::default(),
            external_count: Default::default(),
            hypersynth_count: Default::default(),
            used_midi_channel: Default::default()
        }
    }
}

impl Display for InstrumentCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Instruments count    : {}", self.total())?;
        writeln!(f, "            Wavsynth : {}", self.wavsynth_count)?;
        writeln!(f, "          Macrosynth : {}", self.macrosynth_count)?;
        writeln!(f, "             Sampler : {}", self.sampler_count)?;
        writeln!(f, "             FmSynth : {}", self.fm_count)?;
        writeln!(f, "          HyperSynth : {}", self.hypersynth_count)?;
        writeln!(f, "            MIDI out : {}", self.midi_count)?;
        writeln!(f, "           Ext instr : {}", self.external_count)?;

        let midi_vec : Vec<_> = self.used_midi_channel.iter().map(|c| format!("{}", c)).collect();
        write!(f, "  used midi channels : {}", midi_vec.join(", "))?;
        Ok(())
    }
}

impl<'a> Display for SongInfoDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.song;


        let samples : HashSet<_> = s.instruments
            .iter()
            .filter_map(|i|
                match i {
                    Instrument::Sampler(s) => Some(s.sample_path.clone()),
                    _ => None
                })
            .collect();

        writeln!(f, "Version              : {}", s.version)?;
        writeln!(f, "Name                 : {}", s.name)?;

        let instr_count = s.instruments.iter()
            .fold(
                InstrumentCounter::default(),
                |acc, i| acc.count(i));

        writeln!(f, "{}", instr_count)?;

        writeln!(f, "Distinct samples     : {}", samples.len())?;

        let eqs = s.eqs.iter().filter(|t| !t.is_empty()).count();
        writeln!(f, "Non flat EQs         : {}", eqs)?;

        let tables = s.tables.iter().filter(|t| !t.is_empty()).count();
        writeln!(f, "Non empty table      : {}", tables)?;

        let chains = s.chains.iter().filter(|c| !c.is_empty()).count();
        writeln!(f, "Used chains          : {}", chains)?;

        let phrases = s.phrases.iter().filter(|c| !c.is_empty()).count();
        writeln!(f, "Used phrases         : {}", phrases)?;

        Ok(())
    }
}

fn show_from_song(show: ShowCommand, w: &mut dyn std::io::Write, song: m8_file_parser::Song) -> Result<(), M8FstoErr> {
    match show.show_command {
        ShowTarget::Song => {
            writeln!(w, "{}", song.song).map_err(|_| M8FstoErr::PrintError)
        }
        ShowTarget::Effects => {
            write!(w, "{}", ElemDisplay {
                instr: song.effects_settings,
                ver: song.version
            }).map_err(|_| M8FstoErr::PrintError)
        }
        ShowTarget::Mixer => {
            write!(w, "{}", ElemDisplay {
                instr: song.mixer_settings,
                ver: song.version
            }).map_err(|_| M8FstoErr::PrintError)
        }
        ShowTarget::Info => {
            writeln!(w, "{}", SongInfoDisplay { song: &song }).map_err(|_| M8FstoErr::PrintError)
        }
        ShowTarget::Chain { id } => {
            writeln!(w, "{}", song.chains[id]).map_err(|_| M8FstoErr::PrintError)
        }
        ShowTarget::Phrase { id } => {
            writeln!(w, "{}", song.phrase_view(id)).map_err(|_| M8FstoErr::PrintError)
        },
        ShowTarget::Instrument { id: None} => {
            for (ix, instr) in song.instruments.iter().enumerate() {
                match instr {
                    Instrument::None => {}
                    Instrument::FMSynth(fm) => {
                        writeln!(w, "{:02X} FMSynth:{}", ix, fm.name).map_err(|_| M8FstoErr::PrintError)?;
                    },
                    Instrument::WavSynth(ws) => {
                        writeln!(w, "{:02X} Wavsynth:{} (shape: {:?})", ix, ws.name, ws.shape).map_err(|_| M8FstoErr::PrintError)?;
                    }
                    Instrument::MacroSynth(ms) => {
                        writeln!(w, "{:02X} MacroSynth:{} (shape: {:?})", ix, ms.name, ms.shape).map_err(|_| M8FstoErr::PrintError)?;
                    }
                    Instrument::HyperSynth(hs) => {
                        writeln!(w, "{:02X} HyperSynth:{} ", ix, hs.name).map_err(|_| M8FstoErr::PrintError)?;
                    }
                    Instrument::Sampler(smp) => {
                        writeln!(w, "{:02X} Sampler:{} - {}", ix, smp.name, smp.sample_path).map_err(|_| M8FstoErr::PrintError)?;
                    }
                    Instrument::MIDIOut(midi) => {
                        writeln!(w, "{:02X} MIDIOut:{} (chn {}: bnk {} - prg {})", ix, midi.name, midi.channel, midi.bank_select, midi.program_change).map_err(|_| M8FstoErr::PrintError)?;
                    }
                    Instrument::External(ext) => {
                        writeln!(w, "{:02X} External:{} (chn {}: bnk {} - prg {})", ix, ext.name, ext.channel, ext.bank, ext.program).map_err(|_| M8FstoErr::PrintError)?;
                    }
                }
            };
            Ok(())
        },
        ShowTarget::Instrument { id: Some(id)} => {
            write!(w, "{}", ElemDisplay {
                instr: song.instruments[id].clone(),
                ver: song.version
            }).map_err(|_| M8FstoErr::PrintError)
        },
        ShowTarget::Table { id: None } => {
            writeln!(w, "Please select table number").map_err(|_| M8FstoErr::PrintError)
        },
        ShowTarget::Table { id: Some(id) } => {
            writeln!(w, "{}", song.table_view(id)).map_err(|_| M8FstoErr::PrintError)
        },
        ShowTarget::Eq { id: None } => {
            writeln!(w, "Please select eq number").map_err(|_| M8FstoErr::PrintError)
        }
        ShowTarget::Eq { id: Some(id) } => {
            write!(w, "{}", ElemDisplay {
                instr: song.eqs[id].clone(),
                ver: song.version
            }).map_err(|_| M8FstoErr::PrintError)
        },
    }
}

pub fn show_element(show: ShowCommand, w: &mut dyn std::io::Write) -> Result<(), M8FstoErr> {
    let song_path = PathBuf::from(show.file.clone());
    let file_blob = fs::read(song_path.clone())
        .map_err(|e|
            M8FstoErr::CannotReadFile { path: song_path.clone(), reason: format!("{:?}", e) })?;

    let mut reader = Reader::new(file_blob);

    match m8_file_parser::Song::read_from_reader(&mut reader) {
        Ok(song) => show_from_song(show, w, song),
        Err(e) => {
            reader.set_pos(0);
            match m8_file_parser::Instrument::read_from_reader(&mut reader) {
                Ok(instr_eq) => show_from_instrument(show, w, instr_eq),
                Err(ei) => {
                    Err(M8FstoErr::UnparseableM8File {
                        path: song_path,
                        reason: format!("{:?}\n{:?}", e, ei)
                    })
                }
            }
        }
    }
}
