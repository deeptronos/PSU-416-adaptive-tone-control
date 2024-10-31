use clap::{Error, Parser, Subcommand};
use hound;
use microfft::{complex::cfft_16, Complex32};
use rodio::{source::Source, Decoder, OutputStream};
use std::convert::TryInto;
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use std::path::PathBuf;
use std::process::ExitCode;

// TODO good types?
const LOW: i16 = 0;
const MID: i16 = 300;
const HIGH: i16 = 2000;

const SIZE: usize = 32768;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    audio_file: Option<PathBuf>,
    #[arg(short, long, default_value_t = false)]
    playback: bool,
}

const spec: hound::WavSpec = hound::WavSpec {
    channels: 1,
    sample_rate: 48000,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

fn read_wave(filename: PathBuf) -> (Vec<Complex32>, usize) {
    let mut reader = hound::WavReader::open(filename).unwrap();
    let n_samples = reader.len() as usize;
    let signal = reader
        .samples::<i32>()
        .map(|x| Complex32::new(x.expect("Failed to map x") as f32, 0f32))
        .collect::<Vec<_>>();
    (signal, n_samples)
}

fn process_wave(mut signal: Vec<Complex32>, n_samples: usize) -> Option<f32> {
    println!("n_samples: {}", n_samples);

    let padded_n = SIZE;

    println!("padded n: {}", padded_n);
    signal.resize(padded_n, Complex32::new(0f32, 0f32));
    let mut samples: [_; SIZE] = signal.try_into().unwrap(); // TODO pad samples to be sized to an even multiple of 32! and DM BART!!
                                                             // .unwrap_or(panic!("Unable to put signal into CFFT32 format samples"));

    let spectrum = microfft::complex::cfft_32768(&mut samples);

    println!("Spectrum: {:?}", spectrum);
    Some(0f32)
}
fn playback(audio_file: &Path) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // Output stream handle
    let file = BufReader::new(File::open(audio_file).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples());
    std::thread::sleep(std::time::Duration::from_secs(5));
}

fn main() -> ExitCode {
    // Parse path to WAV file from CLI
    let cli = Cli::parse();

    let audio_file = cli.audio_file.as_deref().unwrap();

    if cli.playback {
        playback(audio_file);
    }

    let buf = audio_file.to_path_buf();
    if let Some(audio_file) = cli.audio_file.as_deref() {
        println!("Parsed audio file: {}", audio_file.display());
    }

    let (signal, n_samples) = read_wave(buf); // Read WAV file to (Vector of audio signal, Length of audio signal)

    // let samples = s.0;
    // const n_samples = s.1;
    let spectrum = process_wave(signal, n_samples);
    println!("CTEST");
    // process_wave(read_wave(buf));

    // // generate 16 samples of a sine wave at frequency 3
    // let sample_count = 16;
    // let signal_freq = 3.;
    // let sample_interval = 1. / sample_count as f32;
    // let mut samples: Vec<_> = (0..sample_count)
    //     .map(|i| (2. * PI * signal_freq * sample_interval * i as f32).sin())
    //     .collect();

    // // compute the RFFT of the samples
    // let mut samples: [_; 16] = samples.try_into().unwrap();
    // let spectrum = microfft::real::rfft_16(&mut samples);
    // // since the real-valued coefficient at the Nyquist frequency is packed into the
    // // imaginary part of the DC bin, it must be cleared before computing the amplitudes
    // spectrum[0].im = 0.0;

    // // the spectrum has a spike at index `signal_freq`
    // let amplitudes: Vec<_> = spectrum.iter().map(|c| c.norm() as u32).collect(); // microfft "std" feature required for c.norm().
    // assert_eq!(&amplitudes, &[0, 0, 0, 8, 0, 0, 0, 0]);

    ExitCode::SUCCESS
}
