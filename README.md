# m8fsto

Command line helper to manage M8 files

Available command :

 * `ls-sample` : to list samples used in a song file along with its instrument number
 * `grep-sample` : to find song using a specific sample
 * `broken-search` : Find songs with missing samples
 * `bundle` : Bundling a song (without a M8)
 * `prune-bundle` : Remove unused extra sample from a bundled song.
 * `mv`: move a file or folder, rewriting all song files using the moved samples
   to point to the new location.
 * `show`: display part of m8 song files in the console.

## Examples

### help

```
> m8fsto help
```

Display inline help

To get help for a specific sub command, you can use

```
> m8fsto help grep-sample
```

### ls-sample

```
> m8fsto ls-sample 'C:\Users\twins\tracks\M8 backup\Songs\UNFINISHED\AMCHORD.m8s'

C:\Users\twins\tracks\M8 backup\Songs\UNFINISHED\AMCHORD.m8s
  00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
  01 SNARECKK : /Samples/Drums/Hits/TR909/SD/ST0T0S7.wav
  02 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
  03 : /Samples/Drums/Hits/TR909/CH/HHCD4.wav
  04 : /Samples/Drums/Hits/TR909/CR/RIDED6.wav
```

This example will list the samples used in a specific M8 song.
The number is the instrument number, with the optional instrument
name, followed by the sample path.


```
m8fsto ls-sample 'C:\Users\twins\tracks\M8 backup/**/*.m8s'
```

Will display the list of all samples used in songs present in a backup folder
or SD card.

### show

Show command can display the content of a m8 song or instrument file.

```
> m8fsto show '..\Songs\DONE\2025\07_JULY\FANFARE.m8s' phrase 0x66
PHRASE 66

  N   V  I  FX1   FX2   FX3
0 C-6 64 60 ---   ---   FM316
1 --- -- -- ---   ---   ---
2 C-4 64 60 ---   ---   FM31a
3 --- -- -- ---   ---   ---
4 C-4 64 60 DEL02 ---   FM31b
5 --- -- -- ---   ---   ---
6 C-4 64 60 ---   ---   FM326
7 --- -- -- ---   ---   ---
8 C-6 64 60 DEL01 ---   FM316
9 --- -- -- ---   ---   ---
a C-6 64 60 ---   ---   FM316
b C-5 64 60 DEL01 ---   FM319
c C-5 74 60 ---   ---   FM316
d C-4 74 60 DEL02 ---   FM31b
e C-6 64 60 ---   ---   FM31d
f --- -- -- ---   ---   ---
```

Please be sure to use 0x prefix for ID to use hexadecimal notation.
The followng elements can be printed:

 * `info`: display generali information/statistics about the song
 * `song`: display the song view, with all the chains
 * `chain`: display the phrase list of a specific chain (don't forget `0x` prefix)
 * `phrase`: display a specific phrase (again, `0x`)
 * `instrument`: display an instrument configuration (without table, but with modulators)
 * `table`: display a table (you can reuse instrument number to display an instrument table)
 * `eq`: display an equalizer configuration (no plotting)

The generic `info` is here to get high level overview:

```
> m8fsto show '..\Songs\DONE\2025\07_JULY\FANFARE.m8s' info
Version              : 6.0.1
Name                 : FANFARE
Instruments count    : 13
            Wavsynth : 0
          Macrosynth : 0
             Sampler : 3
             FmSynth : 9
          HyperSynth : 0
            MIDI out : 1
           Ext instr : 0
  used midi channels : 10
Distinct samples     : 3
Non flat EQs         : 8
Non empty table      : 10
Used chains          : 46
Used phrases         : 82
```

### grep-sample

A reverse proposition from ls-sample, we have a sample, but we want to find
the song using them, so we grep-sample to search through the songs.

```
> m8fsto grep-sample '*/BT7AADA.wav' 'C:\Users\twins\tracks\M8 backup\Songs\**/*.m8s'

C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\04 APRIL\DADRO.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\06 JUNE\VIAKE.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\DABASSTE.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\FSTUB.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\FSTUB.m8s:01 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\GNARTTERY.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\HYJUNGLE.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\IDEABOX.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\IDEABOX.m8s:01 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\11-NOV\SADUB.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\12-DEC\DABASSTE.m8s:00 909KICKK : /Samples/Drums/Hits/TR909/BD/BT7AADA.wav
```

Here we want to search for the usage of a 909 kick sample through all the songs.
The pattern AND the folder use glob matching. We get the song file, the instrument
number and name, along with the full matched sample.

```
> m8fsto grep-sample '**/SFM/*' 'C:\Users\twins\tracks\M8 backup\Songs\**/*.m8s'

C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\GNARTTERY.m8s:01 909KICKK : /Samples/Packs/SFM/essen/Drums/02. Kits/CR78 From Mars/02. Kit 2/Conga CR78 01.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\GNARTTERY.m8s:10  : /Samples/Packs/SFM/essen/Drums/02. Kits/DMX From Mars/08. 612 Echo/SD DMX 612 Echo 11.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\GNARTTERY.m8s:11  : /Samples/Packs/SFM/essen/Drums/02. Kits/CR78 From Mars/02. Kit 2/OH CR78 06.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\IDEABOX.m8s:02  : /Samples/Packs/SFM/909/Individual Hits/06. Hi Hat/01. CH/02. Color/CH 909 Color 02.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\IDEABOX.m8s:10  : /Samples/Packs/SFM/909/Individual Hits/05. Hand Clap/02. Color/Clap 909 Color 02.wav
C:\Users\twins\tracks\M8 backup\Songs\BOF\2024\10-OCT\IDEABOX.m8s:60  : /Samples/Packs/SFM/essen/Synths/Dr Sample From Mars/Bass/Goon Bass Dr Sample 13 C1.wav
```

This command for the usage of any sample within `SFM` folder

### broken-search

Broken search will list songs using sample that has been moved or
removed, and can no longer be found in your backup or SD card.

```
m8fsto broken-search 'C:\Users\twins\tracks\M8 backup'
```

will search for all of the broken songs present in a M8 sd
card backup (or directly on the SD card if you want).

### bundle

Allow to create a song bundle using only SD card data or M8 backup
file.

```
> m8fsto bundle '\M8 backup\Songs\DONE\2025\01_JAN\IDEABOX2.m8s' '\M8 backup' './rebundle'
```

This will create a bundle for the song `IDEABOX2.m8s` with the backup being rooted at `/M8 backup`
and will make the bundle in the folder `./rebundle` (the folder must exists!). A subfolder with
the song name will be created. You can list the samples on the bundled song after

```
> m8fsto ls-sample .\rebundle\IDEABOX2\IDEABOX2.m8s

.\rebundle\IDEABOX2\IDEABOX2.m8s
  00 909KICKK : Samples/0_BT7AADA.wav
  01 909KICKK : Samples/0_BT7AADA.wav
  10 : Samples/16_Clap 909 Color 02.wav
  61 : Samples/97_Minor 7 Dr Sample 01 Cm7.wav
  62 : Samples/98_Massive Poly Voyetra C1 02.wav
  64 : Samples/100_Awaiting Raptor Tape Fragments.wav
  65 : Samples/100_Awaiting Raptor Tape Fragments.wav
  66 : Samples/102_80s Threat Wasp C2.wav
```

You can see that the sample path are now relative.

### Prune bundle

After multiple rebundling of the same song, unused sample can linger
in the Sample sub-folder, this command is here to prune all these
unused samples. You can first try a dry run of the pruning, to avoid
bad surprises:

```
> m8fsto prune-bundle --dry-run 'C:\Users\twins\tracks\M8 backup\Bundles\REREVERB\REREVERB.m8s'
Extra samples to be removed:
 * '"C:\\Users\\twins\\tracks\\M8 backup\\Bundles\\REREVERB\\Samples\\004_LSDJ.wav"'
```

This will only list the files to be removed, you can then launch the realy cleaning
without the `--dry-run` flag:

```
> m8fsto prune-bundle --dry-run 'C:\Users\twins\tracks\M8 backup\Bundles\REREVERB\REREVERB.m8s'
Removing '"C:\\Users\\twins\\tracks\\M8 backup\\Bundles\\REREVERB\\Samples\\004_LSDJ.wav"'
```

### mv


You can move a folder within a backup, the option `--dry-run` will show all
songs that should be rewritten:

```
> m8fsto mv --dry-run --root "$HOME/tracks/M8 backup" "$HOME/tracks/M8 backup/Samples/Packs/Elektron" "$HOME/tracks/M8 backup/Samples/Packs/Elk"
```

> *This features only works on songs written with M8 firmware version 4 and above, previous firmware are not supported.*


if you have song made using older firmware you can use the --force flag to force the action to be performed, otherwize nothing will be applied.

```
> m8fsto mv --force --root "$HOME/tracks/M8 backup" "$HOME/tracks/M8 backup/Samples/Packs/Elektron" "$HOME/tracks/M8 backup/Samples/Packs/Elk"
```

You can also move single files:

```
> m8fsto mv --force --root "$HOME/tracks/M8 backup" "$HOME/tracks/M8 backup/Samples/Drums/Hits/TR909/BD/BT7AADA.wav" "$HOME/tracks/M8 backup/Samples/MY_HH_10.wav"
```

## Garbage printed after the command

Every problematic element is logged, and written on stderr
after the command has been executed. You can redirect stderr
to `/dev/null` or an equivalent on your platform of choice.
