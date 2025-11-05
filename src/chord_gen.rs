use m8_file_parser::writer::Writer;
use m8_file_parser::{ AHDEnv, FMWave, FmAlgo, HyperSynth, Instrument, InstrumentWithEq, LfoShape, LimitType, Mod, Operator, SynthParams, Table, Version, LFO };
use m8_file_parser::FMSynth;

use crate::types::M8FstoErr;

#[derive(Clone)]
struct Chord {
    name: String,
    offsets: Vec<usize>
}

impl Chord {
    pub fn make(n : &str, offsets: Vec<usize>) -> Chord {
        Chord {
            name: String::from(n),
            offsets
        }
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    fn make_mod_op(&self) -> Operator {
        Operator {
            shape: FMWave::SIN,
            ratio: 0x01,
            ratio_fine: 0x00,
            level: 0x80,
            feedback: 0x00,
            retrigger: 0x00,
            mod_a: 0x00,
            mod_b: 0x00,
        }
    }

    fn make_op(&self, ix: usize) -> Operator {
        let ofs = self.offsets[ix];
        let freq : f64 = (2 as f64).powf((ofs as f64) / 12.0);
        let fine = (freq - ((freq as i64) as f64)) * 100.0;

        Operator {
            shape: FMWave::SAW,
            ratio: freq as u8,
            ratio_fine: fine as u8,
            level: 0x80,
            feedback: 0x00,
            retrigger: 0x00,
            mod_a: 0x00,
            mod_b: 0x00,
        }
    }

    /// encode all the inversions as chords inside the hypersynth.
    pub fn as_hypersynth(&self) -> HyperSynth {
        let mut chords : [m8_file_parser::Chord; 0x10] = Default::default();
        let mut default_chord : [u8; 7] = Default::default();

        for (i, ofs) in self.offsets.iter().enumerate() {
            default_chord[i + 1] = *ofs as u8;
        }

        let mut ofs = self.offsets.clone();
        let mut mutate_cursor = 8;

        for i in 0 .. ofs.len() {
            if mutate_cursor >= ofs.len() {
                mutate_cursor = 0;
            } else {
                ofs[mutate_cursor] += 12; // octave up!
                mutate_cursor += 1;
            }

            let mut mask = 0xC0;
            let max = (chords[i].offsets.len() / ofs.len()) * ofs.len();
            for write_cursor in 0 .. max {
                chords[i].offsets[write_cursor] = ofs[write_cursor % ofs.len()] as u8;
                mask = mask | (1 << write_cursor);
            }

            chords[i].mask = mask
        }

        HyperSynth {
            number: 0,
            name: self.name.clone(),
            transpose: true,
            table_tick: 1,
            synth_params: Self::make_synth_param(),
            scale: 0x00,
            default_chord,
            shift: 0x80,
            swarm: 0,
            width: 0,
            subosc: 0x80,
            chords
        }
    }

    fn make_synth_param() -> SynthParams {
        let ahd = AHDEnv {
            dest: 0,
            amount: 0xFF,
            attack: 0,
            hold: 0,
            decay: 0x80,
        };

        let lfo = LFO {
            shape: LfoShape::TRI,
            dest: 0,
            trigger_mode: m8_file_parser::LfoTriggerMode::FREE,
            freq: 0x10,
            amount: 0xFF,
            retrigger: 0,
        };

        SynthParams {
            volume: 0x0,
            pitch: 0,
            fine_tune: 0x80,
            filter_type: 0,
            filter_cutoff: 0xFF,
            filter_res: 0x0,
            amp: 0,
            limit: LimitType(0),
            mixer_pan: 0x80,
            mixer_dry: 0xC0,
            mixer_mfx: 0,
            mixer_delay: 0,
            mixer_reverb: 0x00,
            associated_eq: 0x80,
            mods: [
                Mod::AHDEnv(ahd.clone()),
                Mod::AHDEnv(ahd),
                Mod::LFO(lfo.clone()),
                Mod::LFO(lfo),
            ],
        }
    }

    pub fn as_fm(&self, inversion: u8) -> FMSynth {
        let mk_op = |i| {
            if self.offsets.len() > i {
                self.make_op(i)
            } else {
                self.make_mod_op()
            }
        };

        let operators = [
            mk_op(3),
            mk_op(2),
            mk_op(1),
            mk_op(0)
        ];

        let algo =
            match self.offsets.len() {
                4 => FmAlgo(0xB),
                3 => FmAlgo(0x8),
                2 => FmAlgo(0x7),
                _ => FmAlgo(0x0),
            };

        FMSynth { 
            number: 0,
            name: 
                if inversion > 0 {
                    format!("{}_INV{}", self.name, inversion)
                } else {
                    self.name.clone()
                },
            transpose: true,
            table_tick: 1,
            synth_params: Self::make_synth_param(),
            algo,
            operators,
            mod1: 0,
            mod2: 0,
            mod3: 0,
            mod4: 0
        }
    }
}

fn build_chords() -> Vec<Chord> {
    vec![
        Chord::make("MAJ", vec![0, 4, 7]),
        Chord::make("MAJ6", vec![0, 4, 7, 9]),
        Chord::make("DOM7", vec![0, 4, 7, 10]),
        Chord::make("MAJ7", vec![0, 4, 7, 11]),
        Chord::make("AUG", vec![0, 4, 8]),
        Chord::make("AUG7", vec![0, 4, 8, 10]),
        Chord::make("MIN", vec![0, 3, 7]),
        Chord::make("MIN6", vec![0, 3, 7, 9]),
        Chord::make("MIN7", vec![0, 3, 7, 10]),
        Chord::make("MINMAJ7", vec![0, 3, 7, 11]),
        Chord::make("DIM", vec![0, 3, 6]),
        Chord::make("DIM7", vec![0, 3, 6, 9]),
        Chord::make("HDIM7", vec![0, 3, 6, 10]),
        Chord::make("POW", vec![0, 7]),
        Chord::make("POW_AUG", vec![0, 7, 12]),
    ]
}

fn wrap(synth: Instrument) -> InstrumentWithEq {
    let ver = Version {
        major: 4, minor: 2, patch: 0
    };

    InstrumentWithEq {
        instrument: synth,
        table: Table::default_ver(ver),
        eq: None,
        version: ver,
    }
}

pub fn generate() -> Result<(), M8FstoErr> {

    for chord in &build_chords() {
        let folder_prefix = format!("FM_CHORDS/{}", &chord.name);

        // one folder per chord
        std::fs::create_dir_all(&folder_prefix).map_err(|err|
            M8FstoErr::FolderCreationError {
                path: (&chord.name).into(),
                reason: format!("{}", err)
        })?;

        let instrument =
            wrap(Instrument::FMSynth(chord.as_fm(0)));

        let mut w = Writer::new_instrument_writer(false);
        let instr_name = format!("{}/{}.m8i", &folder_prefix, &chord.name);
        instrument.write(&mut w);

        std::fs::write(&instr_name, &w.finish())
            .map_err(|err|
                M8FstoErr::SongSerializationError {
                    destination: instr_name.clone(),
                    reason: format!("{}", err)
                })?;

        let mut cchord = chord.clone();
        for inversion in 0 .. chord.len() - 1 {
            let instr_name = format!("{}/{}_INV{}.m8i", &folder_prefix, &chord.name, inversion);
            cchord.offsets[inversion] += 12;
            let as_fm = cchord.as_fm((inversion + 1) as u8);
            let inv_instr = wrap(Instrument::FMSynth(as_fm));
            let mut inv_w = Writer::new_instrument_writer(false);
            inv_instr.write(&mut inv_w);
            std::fs::write(&instr_name, &inv_w.finish())
                .map_err(|err|
                    M8FstoErr::SongSerializationError {
                        destination: instr_name.clone(),
                        reason: format!("{}", err)
                    })?;
        }

        let hs_chord =
            wrap(Instrument::HyperSynth(chord.as_hypersynth()));
        let hs_instr_name = format!("{}/{}_HS.m8i", &folder_prefix, &chord.name);

        let mut hs_w = Writer::new_instrument_writer(false);
        hs_chord.write(&mut hs_w);
        std::fs::write(&hs_instr_name, &hs_w.finish())
            .map_err(|err|
                M8FstoErr::SongSerializationError {
                    destination: hs_instr_name.clone(),
                    reason: format!("{}", err)
                })?;
    }

    Ok(())
}
