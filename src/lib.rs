use serde::{Deserialize, Serialize};

// ── AcousticRoom ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticRoom {
    pub id: String,
    pub dimensions: (f64, f64, f64), // (width, depth, height) in meters
    pub absorption: f64,             // 0.0 = perfect reflection, 1.0 = perfect absorption
    pub temperature: f64,            // Celsius
}

impl AcousticRoom {
    pub fn speed_of_sound(&self) -> f64 {
        331.3 + 0.606 * self.temperature
    }

    pub fn volume(&self) -> f64 {
        self.dimensions.0 * self.dimensions.1 * self.dimensions.2
    }

    pub fn surface_area(&self) -> f64 {
        let (w, d, h) = self.dimensions;
        2.0 * (w * d + d * h + w * h)
    }
}

// ── ModeType ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeType {
    Axial,
    Tangential,
    Oblique,
}

// ── RoomMode ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMode {
    pub frequency: f64,
    pub mode_type: ModeType,
    pub indices: (u32, u32, u32),
}

impl RoomMode {
    pub fn wavelength(&self, speed: f64) -> f64 {
        if self.frequency > 0.0 {
            speed / self.frequency
        } else {
            f64::INFINITY
        }
    }
}

// ── Reverb ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reverb {
    pub room: AcousticRoom,
}

impl Reverb {
    /// Sabine equation: RT60 = 0.161 * V / (A * S)
    pub fn rt60(&self) -> f64 {
        let a = self.room.absorption.max(0.001); // avoid division by zero
        0.161 * self.room.volume() / (a * self.room.surface_area())
    }

    pub fn early_decay_time(&self) -> f64 {
        self.rt60() * 0.8
    }

    pub fn clarity_c50(&self) -> f64 {
        let rt60 = self.rt60();
        let early = 1.0 - (-6.91 * 0.05 / rt60).exp();
        let late = (-6.91 * 0.05 / rt60).exp();
        10.0 * (early / late.max(1e-10)).log10()
    }

    pub fn clarity_c80(&self) -> f64 {
        let rt60 = self.rt60();
        let early = 1.0 - (-6.91 * 0.08 / rt60).exp();
        let late = (-6.91 * 0.08 / rt60).exp();
        10.0 * (early / late.max(1e-10)).log10()
    }

    pub fn definition_d50(&self) -> f64 {
        let rt60 = self.rt60();
        1.0 - (-6.91 * 0.05 / rt60).exp()
    }
}

// ── StandingWave ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandingWave {
    pub room: AcousticRoom,
}

impl StandingWave {
    /// Axial modes for a given dimension index (0=x, 1=y, 2=z), up to n modes
    pub fn axial_modes(&self, n: usize) -> Vec<f64> {
        let c = self.room.speed_of_sound();
        let dims = [
            self.room.dimensions.0,
            self.room.dimensions.1,
            self.room.dimensions.2,
        ];
        let mut modes = Vec::new();
        for &dim in &dims {
            for i in 1..=n {
                modes.push((c / 2.0) * (i as f64 / dim));
            }
        }
        modes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        modes
    }

    pub fn tangential_modes(&self, n: usize) -> Vec<f64> {
        let c = self.room.speed_of_sound() / 2.0;
        let dims = [
            self.room.dimensions.0,
            self.room.dimensions.1,
            self.room.dimensions.2,
        ];
        let mut modes = Vec::new();
        for i in 0..3 {
            for j in (i + 1)..3 {
                for ni in 1..=n as u32 {
                    for nj in 1..=n as u32 {
                        let f = c
                            * ((ni as f64 / dims[i]).powi(2)
                                + (nj as f64 / dims[j]).powi(2))
                            .sqrt();
                        modes.push(f);
                    }
                }
            }
        }
        modes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        modes
    }

    pub fn oblique_modes(&self, n: usize) -> Vec<f64> {
        let c = self.room.speed_of_sound() / 2.0;
        let (lx, ly, lz) = self.room.dimensions;
        let mut modes = Vec::new();
        for nx in 1..=n as u32 {
            for ny in 1..=n as u32 {
                for nz in 1..=n as u32 {
                    let f = c
                        * ((nx as f64 / lx).powi(2)
                            + (ny as f64 / ly).powi(2)
                            + (nz as f64 / lz).powi(2))
                        .sqrt();
                    modes.push(f);
                }
            }
        }
        modes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        modes
    }

    pub fn all_modes(&self, max_freq: f64) -> Vec<RoomMode> {
        let c = self.room.speed_of_sound() / 2.0;
        let (lx, ly, lz) = self.room.dimensions;
        let max_n = ((max_freq * 2.0 * lx.min(ly).min(lz) / self.room.speed_of_sound()).ceil()
            as u32)
            .max(20);
        let mut modes = Vec::new();

        for nx in 0..=max_n {
            for ny in 0..=max_n {
                for nz in 0..=max_n {
                    if nx == 0 && ny == 0 && nz == 0 {
                        continue;
                    }
                    let f = c
                        * ((nx as f64 / lx).powi(2)
                            + (ny as f64 / ly).powi(2)
                            + (nz as f64 / lz).powi(2))
                        .sqrt();
                    if f > max_freq {
                        continue;
                    }
                    let nonzero = (nx > 0) as u8 + (ny > 0) as u8 + (nz > 0) as u8;
                    let mode_type = match nonzero {
                        1 => ModeType::Axial,
                        2 => ModeType::Tangential,
                        _ => ModeType::Oblique,
                    };
                    modes.push(RoomMode {
                        frequency: f,
                        mode_type,
                        indices: (nx, ny, nz),
                    });
                }
            }
        }

        modes.sort_by(|a, b| a.frequency.partial_cmp(&b.frequency).unwrap());
        modes
    }

    /// Modes per Hz at a given frequency (approximate)
    pub fn mode_density(&self, freq: f64) -> f64 {
        let v = self.room.volume();
        let c = self.room.speed_of_sound();
        if freq <= 0.0 {
            return 0.0;
        }
        // dN/df ≈ 4πV f² / c³
        4.0 * std::f64::consts::PI * v * freq.powi(2) / c.powi(3)
    }

    /// Schroeder frequency: crossover from discrete to statistical behavior
    pub fn schroeder_frequency(&self) -> f64 {
        let rt60 = Reverb {
            room: self.room.clone(),
        }
        .rt60();
        let v = self.room.volume();
        // f_s = 2000 * sqrt(RT60 / V)  (approximate form)
        // More standard: f_s = c/2 * sqrt(RT60/V) ... but commonly:
        2000.0 * (rt60 / v).sqrt()
    }
}

// ── ResonanceProfile ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceProfile {
    pub modes: Vec<RoomMode>,
    pub rt60: f64,
    pub schroeder_freq: f64,
}

impl ResonanceProfile {
    /// Simulate frequency response (simplified: boost near modes)
    pub fn frequency_response(&self, freqs: &[f64]) -> Vec<f64> {
        freqs
            .iter()
            .map(|&f| {
                let mut response = 0.0;
                for mode in &self.modes {
                    let diff = f - mode.frequency;
                    let bandwidth = 5.0; // Hz
                    response += 1.0 / (1.0 + (diff / bandwidth).powi(2));
                }
                response
            })
            .collect()
    }

    pub fn is_dead(&self) -> bool {
        self.rt60 < 0.3
    }

    pub fn is_live(&self) -> bool {
        self.rt60 > 2.0
    }

    pub fn is_balanced(&self) -> bool {
        self.rt60 > 0.8 && self.rt60 < 1.5
    }

    /// Frequencies where modes cluster too close (within threshold Hz)
    pub fn problem_frequencies(&self, threshold: f64) -> Vec<f64> {
        let mut problems = Vec::new();
        let sorted: Vec<&RoomMode> = {
            let mut m: Vec<&RoomMode> = self.modes.iter().collect();
            m.sort_by(|a, b| a.frequency.partial_cmp(&b.frequency).unwrap());
            m
        };
        for window in sorted.windows(2) {
            if (window[1].frequency - window[0].frequency).abs() < threshold {
                problems.push((window[0].frequency + window[1].frequency) / 2.0);
            }
        }
        problems.dedup_by(|a, b| (*a - *b).abs() < threshold);
        problems
    }
}

// ── SignalPath ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPath {
    pub source: (f64, f64, f64),
    pub listener: (f64, f64, f64),
}

impl SignalPath {
    pub fn direct_distance(&self) -> f64 {
        let dx = self.listener.0 - self.source.0;
        let dy = self.listener.1 - self.source.1;
        let dz = self.listener.2 - self.source.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Shortest first reflection: check all 6 walls
    pub fn first_reflection_distance(&self, room: &AcousticRoom) -> f64 {
        let (sx, sy, sz) = self.source;
        let (lx, ly, lz) = self.listener;
        let (w, d, h) = room.dimensions;
        // Image source method: reflect source across each wall
        let images = [
            (-sx, sy, sz),       // left wall x=0
            (2.0 * w - sx, sy, sz), // right wall x=w
            (sx, -sy, sz),       // front wall y=0
            (sx, 2.0 * d - sy, sz), // back wall y=d
            (sx, sy, -sz),       // floor z=0
            (sx, sy, 2.0 * h - sz), // ceiling z=h
        ];
        images
            .iter()
            .map(|&(ix, iy, iz)| {
                let dx = lx - ix;
                let dy = ly - iy;
                let dz = lz - iz;
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .fold(f64::INFINITY, f64::min)
    }

    pub fn delay_ms(&self, distance: f64, speed: f64) -> f64 {
        if speed <= 0.0 {
            return f64::INFINITY;
        }
        distance / speed * 1000.0
    }

    /// Inverse square law attenuation (relative to 1m reference)
    pub fn attenuation(&self, distance: f64) -> f64 {
        if distance <= 0.0 {
            return 1.0;
        }
        1.0 / (distance * distance)
    }
}

// ── ImpulseResponse ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpulseResponse {
    pub samples: Vec<f64>,
    pub sample_rate: f64,
}

impl ImpulseResponse {
    pub fn direct_sound_idx(&self) -> usize {
        self.samples
            .iter()
            .position(|&s| s.abs() > 1e-10)
            .unwrap_or(self.samples.len())
    }

    pub fn energy(&self) -> f64 {
        self.samples.iter().map(|s| s * s).sum()
    }

    /// Simple DFT magnitude spectrum
    pub fn frequency_spectrum(&self) -> Vec<f64> {
        let n = self.samples.len();
        if n == 0 {
            return Vec::new();
        }
        let half = n / 2;
        let mut spectrum = Vec::with_capacity(half);
        for k in 0..half {
            let mut re = 0.0;
            let mut im = 0.0;
            for (t, s) in self.samples.iter().enumerate() {
                let angle = -2.0 * std::f64::consts::PI * k as f64 * t as f64 / n as f64;
                re += s * angle.cos();
                im += s * angle.sin();
            }
            spectrum.push((re * re + im * im).sqrt() / n as f64);
        }
        spectrum
    }

    pub fn is_causal(&self) -> bool {
        let idx = self.direct_sound_idx();
        self.samples[..idx].iter().all(|&s| s.abs() < 1e-10)
    }
}

// ── AcousticAnalysis ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl AcousticAnalysis {
    pub fn summary(&self) -> String {
        let character = if self.is_dead {
            "dead (dry)"
        } else if self.is_live {
            "live (reverberant)"
        } else if self.is_balanced {
            "balanced"
        } else {
            "moderate"
        };
        format!(
            "Room \"{}\": RT60={:.3}s, {} modes below Schröder freq ({:.1} Hz), character: {}. {} problem frequencies detected.",
            self.room_id,
            self.rt60,
            self.mode_count,
            self.schroeder_freq,
            character,
            self.problem_frequencies.len()
        )
    }
}

// ── AcousticDiff ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticDiff {
    pub rt60_diff: f64,
    pub mode_count_diff: i32,
    pub volume_ratio: f64,
}

impl AcousticDiff {
    pub fn is_similar(&self) -> bool {
        self.rt60_diff.abs() < 0.3 && (self.mode_count_diff.abs() as f64) < self.mode_count_diff.unsigned_abs().max(1) as f64 * 0.2
    }
}

// ── RoomAcousticsEngine ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomAcousticsEngine {
    pub room: AcousticRoom,
    pub reverb: Reverb,
    pub standing: StandingWave,
}

impl RoomAcousticsEngine {
    pub fn new(room: AcousticRoom) -> Self {
        let reverb = Reverb {
            room: room.clone(),
        };
        let standing = StandingWave {
            room: room.clone(),
        };
        Self {
            room,
            reverb,
            standing,
        }
    }

    pub fn full_analysis(&self) -> AcousticAnalysis {
        let rt60 = self.reverb.rt60();
        let modes = self.standing.all_modes(self.standing.schroeder_frequency().max(100.0));
        let schroeder = self.standing.schroeder_frequency();
        let profile = ResonanceProfile {
            modes: modes.clone(),
            rt60,
            schroeder_freq: schroeder,
        };
        AcousticAnalysis {
            room_id: self.room.id.clone(),
            rt60,
            mode_count: modes.len(),
            schroeder_freq: schroeder,
            is_dead: profile.is_dead(),
            is_live: profile.is_live(),
            is_balanced: profile.is_balanced(),
            problem_frequencies: profile.problem_frequencies(10.0),
        }
    }

    pub fn compare(&self, other: &RoomAcousticsEngine) -> AcousticDiff {
        let self_modes = self.standing.all_modes(500.0);
        let other_modes = other.standing.all_modes(500.0);
        AcousticDiff {
            rt60_diff: self.reverb.rt60() - other.reverb.rt60(),
            mode_count_diff: self_modes.len() as i32 - other_modes.len() as i32,
            volume_ratio: self.room.volume() / other.room.volume().max(1e-10),
        }
    }
}

// main is in src/main.rs
