use std::convert::TryInto;
use std::f32::consts::PI;

fn main() {
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
    let amplitudes: Vec<_> = spectrum.iter().map(|c| c.norm() as u32).collect();
    assert_eq!(&amplitudes, &[0, 0, 0, 8, 0, 0, 0, 0]);
}
