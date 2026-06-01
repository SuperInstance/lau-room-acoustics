# lau-room-acoustics

**Room acoustics for PLATO** — reverberation time, standing wave modes, resonance profiles, signal paths, and impulse response analysis, all implemented from first principles.

## What This Does

Given a room's physical dimensions (width, depth, height), wall absorption coefficient, and temperature, this library computes:

1. **Reverberation** — RT60 (Sabine equation), early decay time, clarity (C50/C80), definition (D50)
2. **Standing waves** — Axial, tangential, and oblique room modes up to a cutoff frequency
3. **Resonance profiles** — Mode density, Schröder frequency, problem frequency detection
4. **Signal paths** — Direct sound distance, first reflection (image source method), propagation delay, inverse-square attenuation
5. **Impulse responses** — Energy, frequency spectrum (DFT), causality check
6. **Full analysis** — One-stop characterisation: dead/live/balanced, problem spots, comparison between rooms

## Key Idea

Room acoustics reduces to geometry and wave physics. A rectangular room's resonant frequencies are entirely determined by its dimensions and the speed of sound. The Sabine equation ties reverb time to room volume and absorption. This library implements those formulas directly — no simulation, no FEM, just closed-form acoustics.

## Install

```toml
[dependencies]
lau-room-acoustics = "0.1"
```

Or:

```bash
cargo add lau-room-acoustics
```

Requires **Rust 2021 edition**.

## Quick Start

```rust
use lau_room_acoustics::{AcousticRoom, RoomAcousticsEngine};

fn main() {
    // Define a room: 5m × 4m × 3m, moderate absorption, 20°C
    let room = AcousticRoom {
        id: "studio-a".into(),
        dimensions: (5.0, 4.0, 3.0),
        absorption: 0.3,
        temperature: 20.0,
    };

    // Run full analysis
    let engine = RoomAcousticsEngine::new(room);
    let analysis = engine.full_analysis();

    println!("{}", analysis.summary());
    // → Room "studio-a": RT60=0.452s, 42 modes below Schröder freq (128.3 Hz),
    //   character: dead (dry). 3 problem frequencies detected.
}
```

## API Reference

### `AcousticRoom`

The fundamental unit — a rectangular room with physical properties.

```rust
pub struct AcousticRoom {
    pub id: String,
    pub dimensions: (f64, f64, f64), // (width, depth, height) in meters
    pub absorption: f64,             // 0.0 = mirror, 1.0 = anechoic
    pub temperature: f64,            // Celsius
}
```

| Method | Formula |
|--------|---------|
| `speed_of_sound()` | `331.3 + 0.606 × T` m/s |
| `volume()` | `W × D × H` m³ |
| `surface_area()` | `2(WD + DH + WH)` m² |

### `RoomMode` and `ModeType`

A resonant frequency with its modal indices and classification:

| ModeType | Meaning |
|----------|---------|
| `Axial` | One index non-zero (bounces between two parallel walls) |
| `Tangential` | Two indices non-zero (bounces off four walls) |
| `Oblique` | All three indices non-zero (bounces off all six walls) |

`wavelength(speed)` returns λ = c / f.

### `Reverb`

Reverberation calculations using the Sabine equation:

| Method | Description |
|--------|-------------|
| `rt60()` | RT60 = 0.161 × V / (α × S) |
| `early_decay_time()` | EDT ≈ 0.8 × RT60 |
| `clarity_c50()` | C₅₀ in dB (early-to-late energy ratio at 50ms) |
| `clarity_c80()` | C₈₀ in dB (early-to-late energy ratio at 80ms) |
| `definition_d50()` | D₅₀ (fraction of energy in first 50ms) |

### `StandingWave`

Computes resonant modes for the room:

| Method | Description |
|--------|-------------|
| `axial_modes(n)` | First *n* modes for each dimension |
| `tangential_modes(n)` | Modes involving two dimensions |
| `oblique_modes(n)` | Modes involving all three dimensions |
| `all_modes(max_freq)` | All modes up to a frequency, classified by type |
| `mode_density(freq)` | dN/df at a given frequency |
| `schroeder_frequency()` | Crossover from discrete to statistical behaviour |

### `ResonanceProfile`

Aggregates modes into a room characterisation:

| Method | Description |
|--------|-------------|
| `frequency_response(freqs)` | Simulated response with Lorentzian peaks |
| `is_dead()` / `is_live()` / `is_balanced()` | RT60-based character |
| `problem_frequencies(threshold)` | Frequencies where modes cluster within *threshold* Hz |

### `SignalPath`

Direct and reflected sound between a source and listener:

| Method | Description |
|--------|-------------|
| `direct_distance()` | Euclidean distance |
| `first_reflection_distance(room)` | Shortest image-source reflection |
| `delay_ms(distance, speed)` | Propagation delay in milliseconds |
| `attenuation(distance)` | Inverse-square law (relative to 1m) |

### `ImpulseResponse`

Time-domain impulse response with spectral analysis:

| Method | Description |
|--------|-------------|
| `direct_sound_idx()` | Index of first non-zero sample |
| `energy()` | Total energy Σs² |
| `frequency_spectrum()` | DFT magnitude spectrum (O(N²)) |
| `is_causal()` | True if all samples before direct sound are ~zero |

### `RoomAcousticsEngine`

All-in-one engine combining reverb, standing waves, and analysis:

| Method | Description |
|--------|-------------|
| `new(room)` | Create engine |
| `full_analysis()` | `AcousticAnalysis` with RT60, modes, character, problems |
| `compare(other)` | `AcousticDiff` between two rooms |

### `AcousticAnalysis`

```rust
pub struct AcousticAnalysis {
    pub room_id: String,
    pub rt60: f64,
    pub mode_count: usize,
    pub schroeder_freq: f64,
    pub is_dead: bool,
    pub is_live: bool,
    pub is_balanced: bool,
    pub problem_frequencies: Vec<f64>,
}
```

`summary()` returns a human-readable one-liner.

## How It Works

### Speed of Sound

$$c = 331.3 + 0.606 \times T \quad \text{(m/s)}$$

where $T$ is temperature in °C. At 20°C, $c ≈ 343.4$ m/s.

### Room Modes (Standing Waves)

For a rectangular room with dimensions $(L_x, L_y, L_z)$, the resonant frequencies are:

$$f_{n_x, n_y, n_z} = \frac{c}{2} \sqrt{\left(\frac{n_x}{L_x}\right)^2 + \left(\frac{n_y}{L_y}\right)^2 + \left(\frac{n_z}{L_z}\right)^2}$$

- **Axial** ($n = 1$ non-zero): $f_n = \frac{nc}{2L}$ for each dimension
- **Tangential** ($n = 2$ non-zero): involves two dimensions
- **Oblique** ($n = 3$ non-zero): involves all three

### Sabine Reverberation

$$RT_{60} = \frac{0.161 \times V}{\alpha \times S}$$

where $V$ is volume, $\alpha$ is the average absorption coefficient, and $S$ is total surface area.

### Clarity and Definition

Derived from the exponential decay model:

$$C_{50} = 10 \log_{10}\frac{1 - e^{-6.91 \times 0.05 / RT_{60}}}{e^{-6.91 \times 0.05 / RT_{60}}}$$

$$D_{50} = 1 - e^{-6.91 \times 0.05 / RT_{60}}$$

### Schröder Frequency

Crossover between discrete modal behaviour and statistical diffuse field:

$$f_s = 2000 \sqrt{\frac{RT_{60}}{V}}$$

Below $f_s$, individual modes dominate. Above it, the modal density is high enough for statistical treatment.

### Mode Density

$$\frac{dN}{df} \approx \frac{4\pi V f^2}{c^3}$$

### Image Source Method

First reflections use the image source method: reflect the source position across each of the six walls, then measure the distance from the listener to each image. The shortest gives the first reflection distance.

### Inverse Square Law

$$A(d) = \frac{1}{d^2}$$

Relative to the signal at 1 metre.

## Testing

67 integration tests covering reverb calculations, mode classification, resonance profiles, signal paths, impulse responses, full analysis, and room comparison.

```bash
cargo test
```

## License

MIT
