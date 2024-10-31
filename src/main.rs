use clap::{Error, Parser, Subcommand};
use hound;
use microfft::{complex::cfft_32, Complex32};
use num::complex::Complex;
use rodio::{source::Source, Decoder, OutputStream};
use std::convert::TryInto;
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitCode;

// TODO good types?
const LOW: i16 = 0;
const MID: i16 = 300;
const HIGH: i16 = 2000;

const BUFFER_SIZE: i16 = 2048;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    audio_file: Option<PathBuf>,
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
// fn process_wave<const N: usize>(signal: Vec<Complex32>) -> Option<f32> {
//     // let samples = (0..n_samples). // Turn iterator returns into a samples object??
//     // init new FFT to n_samples size

//     // let signal = reader
//     //     .samples::<i32>()
//     //     .map(|x| Complex32::new(x.expect("Failed to map x") as f32, 0f32))
//     //     .collect::<Vec<_>>();
//     // let samples : [Complex32; n_samples]= signal.try_into().unwrap_or_else(|v: Vec<T>| panic!("Expected a vec"))
//     // let mut samples = &signal[..];
//     let mut samples: [_; n_samples] = signal.try_into().unwrap();
//     let spectrum = microfft::complex::cfft_32(&mut samples);
//     println!("Signal: {:?}", signal);
//     println!("Samples: {:?}", signal);
//     Some(0f32)

//     // let mut spectrum = signal.clone();
//     // let max_peak = spectrum
//     //     .iter()
//     //     .take(n_samples / 2)
//     //     .enumerate()
//     //     .max_by_key(|&(_, freq)| freq.norm() as u32);

//     // if let Some((i, _)) = max_peak {
//     //     let bin = 44100f32 / n_samples as f32;
//     //     Some(i as f32 * bin)
//     // } else {
//     //     None
//     // }
// }

fn process_wave(signal: Vec<Complex32>, n_samples: usize) -> Option<f32> {
    let mut samples: [_; 32] = signal.try_into().unwrap();

    let spectrum = microfft::complex::cfft_32(&mut samples);
    // let mut samples = signal;
    // let spectrum = unsafe{
    //     let ptr = samples.as_mut_ptr()
    //     let slice = std::slice::from_raw_parts_mut(ptr, n_samples);
    //     microfft::complex::cfft_32(slice)
    // }
    // let mut samples: [Complex32; n_samples] = signal.try_into().unwrap();
    // let spectrum = microfft::complex::cfft_32(&mut samples);
    // println!("Samples: {:?}", samples);
    println!("Spectrum: {:?}", spectrum);
    Some(0f32)
}

// fn play_sample() {}

fn main() -> ExitCode {
    // Parse path to WAV file from CLI
    let cli = Cli::parse();
    let audio_file = cli.audio_file.as_deref().unwrap();
    let buf = audio_file.to_path_buf();

    // let s: (Vec<Complex32>, usize) = read_wave(buf); // Read WAV file to (Vector of audio signal, Length of audio signal)

    let (signal, n_samples) = read_wave(buf);

    // let samples = s.0;
    // const n_samples = s.1;
    let spectrum = process_wave(signal, n_samples);
    // process_wave(read_wave(buf));

    // if let Some(audio_file) = cli.audio_file.as_deref() {
    //     println!("Parsed audio file: {}", audio_file.display());
    // }

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // Output stream handle
    // let file = BufReader::new(File::open(audio_file).unwrap());
    // let source = Decoder::new(file).unwrap();
    // stream_handle.play_raw(source.convert_samples());
    // std::thread::sleep(std::time::Duration::from_secs(5));

    // generate 16 samples of a sine wave at frequency 3
    let sample_count = 16;
    let signal_freq = 3.;
    let sample_interval = 1. / sample_count as f32;
    let mut samples: Vec<_> = (0..sample_count)
        .map(|i| (2. * PI * signal_freq * sample_interval * i as f32).sin())
        .collect();

    // compute the RFFT of the samples
    let mut samples: [_; 16] = samples.try_into().unwrap();
    let spectrum = microfft::real::rfft_16(&mut samples);
    // since the real-valued coefficient at the Nyquist frequency is packed into the
    // imaginary part of the DC bin, it must be cleared before computing the amplitudes
    spectrum[0].im = 0.0;

    // the spectrum has a spike at index `signal_freq`
    let amplitudes: Vec<_> = spectrum.iter().map(|c| c.norm() as u32).collect(); // microfft "std" feature required for c.norm().
    assert_eq!(&amplitudes, &[0, 0, 0, 8, 0, 0, 0, 0]);

    ExitCode::SUCCESS
}
