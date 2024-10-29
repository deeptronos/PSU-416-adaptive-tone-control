# Adaptive Tone Control
## Catherine N 2024

We're getting an FFT from crates.io's [`microfft`](https://docs.rs/microfft/0.6.0/microfft/index.html) crate.

We're getting filters from [`github.com/BartMassey/rbj-eq`](github.com/BartMassey/rbj-eq)

## Assignment


Let's arbitrarily divide audio frequencies into "low" band 0-300Hz, "mid" band 300-2000Hz, and "high" band  2000+ Hz. As demonstrated in class, a "tone control" allows adjusting the volume of the sound in each frequency band separately.

Given some input waveform (some music for example) try using an FFT to measure the sound energy in each of these three bands across a short window, then using three tone filters to adjust the energy in these bands so that the energies of all three bands are roughly equal.

There's a lot of places this idea could take you. How long an FFT window should you use? How fast should you adjust the tone filters? Should you use peak band energy or average band energy? What else might be interesting to do?

Note the relationship to "compression" and "equalization".
