use clap::{Error, Parser, Subcommand};
use hound;
use microfft::{complex::cfft_16, Complex32};
// use rodio::cpal::traits::{HostTrait,DeviceTrait};
use rodio::cpal;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{source::Source, Decoder, OutputStream};
use std::cell::RefCell;
use std::cmp::max;
use std::convert::TryInto;
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use std::path::PathBuf;
use std::process::ExitCode;

use audio_visualizer::dynamic::live_input::{list_input_devs, AudioDevAndCfg};
use audio_visualizer::dynamic::window_top_btm::{open_window_connect_audio, TransformFn};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencyValue};

use rodio::*;

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

///
/// Returns: a cpal::Device for the Default output device.
fn list_host_output_devices() -> cpal::Device {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        println!("# Device: {}", dev_name);
    }
    println!("---");
    println!(
        "# Default output device: {}",
        cpal::default_host()
            .default_output_device()
            .unwrap()
            .name()
            .unwrap()
    );

    cpal::default_host().default_output_device().unwrap()
}

fn get_output_stream(device_name: &str) -> (OutputStream, OutputStreamHandle) {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    let (mut _stream, mut stream_handle) = OutputStream::try_default().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        if dev_name == device_name {
            println!("Device found: {}", dev_name);
            (_stream, stream_handle) = OutputStream::try_from_device(&dev).unwrap();
        }
    }
    return (_stream, stream_handle);
}

// fn enumerate_output_devices() -> Vec<(String, cpal::Device)>{
//     list cpal::default_host().default_output_device().unwrap();
//     let name = device.name().unwrap();
// }

fn visualize_audio_device() {
    // Contains the data for the spectrum to be visualized. It contains ordered pairs of
    // `(frequency, frequency_value)`. During each iteration, the frequency value gets
    // combined with `max(old_value * smoothing_factor, new_value)`.
    let visualize_spectrum: RefCell<Vec<(f64, f64)>> = RefCell::new(vec![(0.0, 0.0); 1024]);

    let device: Option<Device> = Some(list_host_output_devices()); // Get the default output device.
    println!(
        "device: {}",
        device
            .as_ref()
            .expect("Unable to get device.")
            .name()
            .unwrap()
    );

    let closure = |i: rodio::cpal::DefaultStreamConfigError| -> SupportedStreamConfig {
        println!("Error: {}", i);
        panic!("Closure failed")
    };

    let cfg = AudioDevAndCfg::new(
        Some(device.clone().unwrap()),
        Some(match device {
            Some(dev) => dev.default_output_config().unwrap_or_else(closure).into(),
            None => panic!("Failed to get device for cfg"),
        }),
    );

    // Closure that captures `visualize_spectrum`.
    let to_spectrum_fn = move |audio: &[f32], sampling_rate| {
        let skip_elements = audio.len() - 2048;
        // spectrum analysis only of the latest 46ms
        let relevant_samples = &audio[skip_elements..skip_elements + 2048];

        // do FFT
        let hann_window = hann_window(relevant_samples);
        let latest_spectrum = samples_fft_to_spectrum(
            &hann_window,
            sampling_rate as u32,
            FrequencyLimit::All,
            Some(&divide_by_N),
        )
        .unwrap();

        // now smoothen the spectrum; old values are decreased a bit and replaced,
        // if the new value is higher
        latest_spectrum
            .data()
            .iter()
            .zip(visualize_spectrum.borrow_mut().iter_mut())
            .for_each(|((fr_new, fr_val_new), (fr_old, fr_val_old))| {
                // actually only required in very first iteration
                *fr_old = fr_new.val() as f64;
                let old_val = *fr_val_old * 0.84;
                let max = max(
                    *fr_val_new * 5000.0_f32.into(),
                    FrequencyValue::from(old_val as f32),
                );
                *fr_val_old = max.val() as f64;
            });

        visualize_spectrum.borrow().clone()
    };

    let tsf = TransformFn::Complex(&to_spectrum_fn);
    open_window_connect_audio(
        "Live Spectrum View",
        None,
        None,
        Some(0.0..22050.0),
        Some(0.0..500.0),
        "x_axis",
        "y_axis",
        cfg,
        tsf,
    )
}

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

    let spectrum = microfft::complex::cfft_32768(&mut samples); // TODO why do large (>32768) samples NOT cause unwrap to fail...

    // println!("Spectrum: {:?}", spectrum);
    Some(0f32)
}

fn playback(audio_file: &Path) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // Output stream handle
                                                                         // OutputStream::x
    let file = BufReader::new(File::open(audio_file).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples());

    std::thread::sleep(std::time::Duration::from_secs(5));
}

fn main() -> ExitCode {
    list_host_output_devices();

    // Acquire handles on default output stream
    let device: rodio::Device = cpal::default_host().default_output_device().unwrap();
    let name = device.name().unwrap();
    let (_stream, stream_handle) = get_output_stream(&name);

    // Parse path to WAV file from CLI
    let cli = Cli::parse();
    let file: Option<PathBuf> = cli.audio_file;

    let audio_file: &Path = match file.as_deref() {
        None => panic!("Error: Nothing to parse."),
        Some(f) => f,
    };

    let viz = visualize_audio_device();
    if cli.playback {
        playback(audio_file);
    }

    return ExitCode::SUCCESS;

    let buf = audio_file.to_path_buf();

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
