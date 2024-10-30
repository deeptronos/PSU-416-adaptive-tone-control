use clap::{Error, Parser, Subcommand};
use rodio::{source::Source, Decoder, OutputStream};
use std::convert::TryInto;
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitCode;

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

fn play_sample() {}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let audio_file = cli.audio_file.as_deref().unwrap();

    // if let Some(audio_file) = cli.audio_file.as_deref() {
    //     println!("Parsed audio file: {}", audio_file.display());
    // }

    let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // Output stream handle
    let file = BufReader::new(File::open(audio_file).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples());
    std::thread::sleep(std::time::Duration::from_secs(5));

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
