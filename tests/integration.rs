#[cfg(test)]
mod tests {
    use lau_room_acoustics::*;

    fn test_room() -> AcousticRoom {
        AcousticRoom {
            id: "test".into(),
            dimensions: (5.0, 4.0, 3.0),
            absorption: 0.3,
            temperature: 20.0,
        }
    }

    fn big_room() -> AcousticRoom {
        AcousticRoom {
            id: "big".into(),
            dimensions: (10.0, 8.0, 6.0),
            absorption: 0.3,
            temperature: 20.0,
        }
    }

    fn dead_room() -> AcousticRoom {
        AcousticRoom {
            id: "dead".into(),
            dimensions: (5.0, 4.0, 3.0),
            absorption: 0.9,
            temperature: 20.0,
        }
    }

    fn live_room() -> AcousticRoom {
        AcousticRoom {
            id: "live".into(),
            dimensions: (5.0, 4.0, 3.0),
            absorption: 0.05,
            temperature: 20.0,
        }
    }

    // ── AcousticRoom ──

    #[test]
    fn test_speed_of_sound() {
        let room = test_room();
        let expected = 331.3 + 0.606 * 20.0;
        assert!((room.speed_of_sound() - expected).abs() < 0.01);
    }

    #[test]
    fn test_speed_of_sound_increases_with_temperature() {
        let cold = AcousticRoom {
            id: "cold".into(),
            dimensions: (5.0, 4.0, 3.0),
            absorption: 0.3,
            temperature: 0.0,
        };
        let hot = AcousticRoom {
            id: "hot".into(),
            dimensions: (5.0, 4.0, 3.0),
            absorption: 0.3,
            temperature: 40.0,
        };
        assert!(hot.speed_of_sound() > cold.speed_of_sound());
    }

    #[test]
    fn test_volume() {
        let room = test_room();
        assert!((room.volume() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_surface_area() {
        let room = test_room();
        let (w, d, h) = room.dimensions;
        let expected = 2.0 * (w * d + d * h + w * h);
        assert!((room.surface_area() - expected).abs() < 0.01);
    }

    // ── Reverb ──

    #[test]
    fn test_rt60_sabine() {
        let room = test_room();
        let reverb = Reverb { room };
        let v = 60.0;
        let s = 2.0 * (20.0 + 12.0 + 15.0); // 94.0
        let expected = 0.161 * v / (0.3 * s);
        assert!((reverb.rt60() - expected).abs() < 0.001);
    }

    #[test]
    fn test_larger_room_longer_rt60() {
        let small_reverb = Reverb { room: test_room() };
        let big_reverb = Reverb { room: big_room() };
        assert!(big_reverb.rt60() > small_reverb.rt60());
    }

    #[test]
    fn test_higher_absorption_shorter_rt60() {
        let r1 = Reverb { room: test_room() };
        let r2 = Reverb { room: dead_room() };
        assert!(r2.rt60() < r1.rt60());
    }

    #[test]
    fn test_early_decay_time() {
        let reverb = Reverb { room: test_room() };
        assert!((reverb.early_decay_time() - reverb.rt60() * 0.8).abs() < 0.001);
    }

    #[test]
    fn test_clarity_c50() {
        let reverb = Reverb { room: test_room() };
        let c50 = reverb.clarity_c50();
        assert!(c50.is_finite());
    }

    #[test]
    fn test_clarity_c80() {
        let reverb = Reverb { room: test_room() };
        let c80 = reverb.clarity_c80();
        assert!(c80.is_finite());
    }

    #[test]
    fn test_definition_d50() {
        let reverb = Reverb { room: test_room() };
        let d50 = reverb.definition_d50();
        assert!(d50 > 0.0 && d50 < 1.0);
    }

    // ── StandingWave ──

    #[test]
    fn test_axial_modes_basic() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.axial_modes(1);
        assert_eq!(modes.len(), 3);
        let c = test_room().speed_of_sound();
        assert!((modes[0] - c / (2.0 * 5.0)).abs() < 0.01); // f_100
    }

    #[test]
    fn test_axial_mode_f100() {
        let room = test_room();
        let c = room.speed_of_sound();
        let expected = c / (2.0 * 5.0); // f_100
        let sw = StandingWave { room };
        let modes = sw.axial_modes(1);
        assert!(modes.iter().any(|m| (m - expected).abs() < 0.1));
    }

    #[test]
    fn test_axial_mode_f010() {
        let room = test_room();
        let c = room.speed_of_sound();
        let expected = c / (2.0 * 4.0);
        let sw = StandingWave { room };
        let modes = sw.axial_modes(1);
        assert!(modes.iter().any(|m| (m - expected).abs() < 0.1));
    }

    #[test]
    fn test_axial_mode_f001() {
        let room = test_room();
        let c = room.speed_of_sound();
        let expected = c / (2.0 * 3.0);
        let sw = StandingWave { room };
        let modes = sw.axial_modes(1);
        assert!(modes.iter().any(|m| (m - expected).abs() < 0.1));
    }

    #[test]
    fn test_tangential_modes() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.tangential_modes(2);
        assert!(!modes.is_empty());
        // Should have 3 pairs of dimensions, 2*2 each = 12
        assert_eq!(modes.len(), 12);
    }

    #[test]
    fn test_oblique_modes() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.oblique_modes(2);
        assert!(!modes.is_empty());
        assert_eq!(modes.len(), 8); // 2^3
    }

    #[test]
    fn test_all_modes_sorted() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.all_modes(200.0);
        for window in modes.windows(2) {
            assert!(window[0].frequency <= window[1].frequency);
        }
    }

    #[test]
    fn test_all_modes_respect_max_freq() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.all_modes(200.0);
        for m in &modes {
            assert!(m.frequency <= 200.01);
        }
    }

    #[test]
    fn test_mode_density_increases_with_frequency() {
        let sw = StandingWave { room: test_room() };
        let d_low = sw.mode_density(100.0);
        let d_high = sw.mode_density(500.0);
        assert!(d_high > d_low);
    }

    #[test]
    fn test_schroeder_frequency() {
        let sw = StandingWave { room: test_room() };
        let sf = sw.schroeder_frequency();
        assert!(sf > 0.0);
        assert!(sf.is_finite());
    }

    #[test]
    fn test_schroeder_decreases_with_larger_room() {
        let sw_small = StandingWave { room: test_room() };
        let sw_big = StandingWave { room: big_room() };
        assert!(sw_big.schroeder_frequency() < sw_small.schroeder_frequency());
    }

    // ── RoomMode ──

    #[test]
    fn test_room_mode_wavelength() {
        let mode = RoomMode {
            frequency: 100.0,
            mode_type: ModeType::Axial,
            indices: (1, 0, 0),
        };
        let wl = mode.wavelength(343.0);
        assert!((wl - 3.43).abs() < 0.01);
    }

    #[test]
    fn test_room_mode_zero_freq_wavelength() {
        let mode = RoomMode {
            frequency: 0.0,
            mode_type: ModeType::Axial,
            indices: (0, 0, 0),
        };
        assert!(mode.wavelength(343.0).is_infinite());
    }

    // ── ModeType ──

    #[test]
    fn test_mode_type_axial_in_all_modes() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.all_modes(200.0);
        assert!(modes.iter().any(|m| m.mode_type == ModeType::Axial));
    }

    #[test]
    fn test_mode_type_tangential_in_all_modes() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.all_modes(300.0);
        assert!(modes.iter().any(|m| m.mode_type == ModeType::Tangential));
    }

    #[test]
    fn test_mode_type_oblique_in_all_modes() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.all_modes(400.0);
        assert!(modes.iter().any(|m| m.mode_type == ModeType::Oblique));
    }

    // ── Cube: all three axial modes at different frequencies ──

    #[test]
    fn test_cube_different_axial_modes() {
        let cube = AcousticRoom {
            id: "cube".into(),
            dimensions: (4.0, 4.0, 4.0),
            absorption: 0.3,
            temperature: 20.0,
        };
        // All three axial modes should be identical in a cube
        let c = cube.speed_of_sound();
        let sw = StandingWave { room: cube };
        let modes = sw.axial_modes(1);
        assert_eq!(modes.len(), 3);
        // In a cube, all three should be the same frequency
        assert!((modes[0] - modes[1]).abs() < 0.01);
        assert!((modes[1] - modes[2]).abs() < 0.01);
        assert!((modes[0] - c / (2.0 * 4.0)).abs() < 0.01);
    }

    // ── ResonanceProfile ──

    #[test]
    fn test_resonance_profile_is_dead() {
        let room = dead_room();
        let reverb = Reverb { room: room.clone() };
        let rt60 = reverb.rt60();
        let profile = ResonanceProfile {
            modes: vec![],
            rt60,
            schroeder_freq: 100.0,
        };
        assert!(profile.is_dead());
    }

    #[test]
    fn test_resonance_profile_is_live() {
        let room = live_room();
        let reverb = Reverb { room: room.clone() };
        let rt60 = reverb.rt60();
        let profile = ResonanceProfile {
            modes: vec![],
            rt60,
            schroeder_freq: 100.0,
        };
        assert!(profile.is_live());
    }

    #[test]
    fn test_resonance_profile_is_balanced() {
        let room = AcousticRoom {
            id: "balanced".into(),
            dimensions: (8.0, 6.0, 4.0),
            absorption: 0.15,
            temperature: 20.0,
        };
        let reverb = Reverb { room: room.clone() };
        let rt60 = reverb.rt60();
        assert!(rt60 > 0.8 && rt60 < 1.5, "RT60 was {rt60}");
        let profile = ResonanceProfile {
            modes: vec![],
            rt60,
            schroeder_freq: 100.0,
        };
        assert!(profile.is_balanced());
    }

    #[test]
    fn test_frequency_response() {
        let modes = vec![RoomMode {
            frequency: 100.0,
            mode_type: ModeType::Axial,
            indices: (1, 0, 0),
        }];
        let profile = ResonanceProfile {
            modes,
            rt60: 1.0,
            schroeder_freq: 200.0,
        };
        let freqs = vec![50.0, 100.0, 150.0];
        let resp = profile.frequency_response(&freqs);
        assert_eq!(resp.len(), 3);
        assert!(resp[1] > resp[0]); // peak at mode frequency
        assert!(resp[1] > resp[2]);
    }

    #[test]
    fn test_problem_frequencies() {
        let modes = vec![
            RoomMode { frequency: 100.0, mode_type: ModeType::Axial, indices: (1, 0, 0) },
            RoomMode { frequency: 102.0, mode_type: ModeType::Axial, indices: (0, 1, 0) },
        ];
        let profile = ResonanceProfile {
            modes,
            rt60: 1.0,
            schroeder_freq: 200.0,
        };
        let probs = profile.problem_frequencies(10.0);
        assert_eq!(probs.len(), 1);
        assert!((probs[0] - 101.0).abs() < 1.0);
    }

    // ── SignalPath ──

    #[test]
    fn test_direct_distance() {
        let path = SignalPath {
            source: (0.0, 0.0, 0.0),
            listener: (3.0, 4.0, 0.0),
        };
        assert!((path.direct_distance() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_first_reflection_longer_than_direct() {
        let room = test_room();
        let path = SignalPath {
            source: (1.0, 1.0, 1.5),
            listener: (4.0, 3.0, 1.5),
        };
        assert!(path.first_reflection_distance(&room) > path.direct_distance());
    }

    #[test]
    fn test_first_reflection_floor() {
        let room = test_room();
        let path = SignalPath {
            source: (2.5, 2.0, 0.1),
            listener: (2.5, 2.0, 2.9),
        };
        let refl = path.first_reflection_distance(&room);
        assert!(refl < f64::INFINITY);
    }

    #[test]
    fn test_delay_ms() {
        let path = SignalPath {
            source: (0.0, 0.0, 0.0),
            listener: (1.0, 0.0, 0.0),
        };
        let delay = path.delay_ms(343.0, 343.0);
        assert!((delay - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_attenuation_inverse_square() {
        let path = SignalPath {
            source: (0.0, 0.0, 0.0),
            listener: (1.0, 0.0, 0.0),
        };
        let a1 = path.attenuation(1.0);
        let a2 = path.attenuation(2.0);
        // Doubling distance = -6dB
        let ratio_db = 10.0 * (a2 / a1).log10();
        assert!((ratio_db - (-6.02)).abs() < 0.1);
    }

    #[test]
    fn test_attenuation_decreases_with_distance() {
        let path = SignalPath {
            source: (0.0, 0.0, 0.0),
            listener: (1.0, 0.0, 0.0),
        };
        assert!(path.attenuation(2.0) < path.attenuation(1.0));
    }

    // ── ImpulseResponse ──

    #[test]
    fn test_direct_sound_idx() {
        let ir = ImpulseResponse {
            samples: vec![0.0, 0.0, 0.0, 0.5, 0.3, 0.1],
            sample_rate: 44100.0,
        };
        assert_eq!(ir.direct_sound_idx(), 3);
    }

    #[test]
    fn test_energy() {
        let ir = ImpulseResponse {
            samples: vec![1.0, 1.0, 1.0],
            sample_rate: 44100.0,
        };
        assert!((ir.energy() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_is_causal() {
        let ir = ImpulseResponse {
            samples: vec![0.0, 0.0, 0.0, 0.5, 0.3],
            sample_rate: 44100.0,
        };
        assert!(ir.is_causal());
    }

    #[test]
    fn test_causality_all_zeros_is_causal() {
        let ir = ImpulseResponse {
            samples: vec![0.0; 5],
            sample_rate: 44100.0,
        };
        assert!(ir.is_causal());
    }

    #[test]
    fn test_frequency_spectrum() {
        let ir = ImpulseResponse {
            samples: vec![0.0; 16],
            sample_rate: 44100.0,
        };
        let spec = ir.frequency_spectrum();
        assert_eq!(spec.len(), 8);
        assert!(spec.iter().all(|&s| s.abs() < 1e-10));
    }

    #[test]
    fn test_frequency_spectrum_nonzero() {
        let mut samples = vec![0.0; 64];
        // Put a sinusoid at some frequency
        for (i, s) in samples.iter_mut().enumerate().take(64) {
            *s = (2.0 * std::f64::consts::PI * 4.0 * i as f64 / 64.0).sin();
        }
        let ir = ImpulseResponse {
            samples,
            sample_rate: 44100.0,
        };
        let spec = ir.frequency_spectrum();
        assert!(spec[4] > spec[0]); // peak at bin 4
    }

    // ── RoomAcousticsEngine ──

    #[test]
    fn test_engine_full_analysis() {
        let engine = RoomAcousticsEngine::new(test_room());
        let analysis = engine.full_analysis();
        assert_eq!(analysis.room_id, "test");
        assert!(analysis.rt60 > 0.0);
        assert!(analysis.mode_count > 0);
        assert!(analysis.schroeder_freq > 0.0);
    }

    #[test]
    fn test_engine_analysis_summary() {
        let engine = RoomAcousticsEngine::new(test_room());
        let summary = engine.full_analysis().summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("RT60"));
    }

    #[test]
    fn test_engine_compare() {
        let e1 = RoomAcousticsEngine::new(test_room());
        let e2 = RoomAcousticsEngine::new(big_room());
        let diff = e1.compare(&e2);
        assert!(diff.rt60_diff < 0.0); // big room has longer RT60
        assert!(diff.volume_ratio < 1.0); // small/big
    }

    #[test]
    fn test_similar_rooms() {
        let room1 = AcousticRoom {
            id: "r1".into(),
            dimensions: (5.0, 4.0, 3.0),
            absorption: 0.3,
            temperature: 20.0,
        };
        let room2 = AcousticRoom {
            id: "r2".into(),
            dimensions: (5.1, 4.1, 3.1),
            absorption: 0.31,
            temperature: 20.0,
        };
        let e1 = RoomAcousticsEngine::new(room1);
        let e2 = RoomAcousticsEngine::new(room2);
        let diff = e1.compare(&e2);
        assert!(diff.rt60_diff.abs() < 0.3);
    }

    // ── Serde round-trip ──

    #[test]
    fn test_room_serde_roundtrip() {
        let room = test_room();
        let json = serde_json::to_string(&room).unwrap();
        let back: AcousticRoom = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, room.id);
        assert!((back.absorption - room.absorption).abs() < 1e-10);
    }

    #[test]
    fn test_mode_type_serde_roundtrip() {
        let mt = ModeType::Tangential;
        let json = serde_json::to_string(&mt).unwrap();
        let back: ModeType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ModeType::Tangential);
    }

    #[test]
    fn test_room_mode_serde_roundtrip() {
        let mode = RoomMode {
            frequency: 100.0,
            mode_type: ModeType::Oblique,
            indices: (1, 2, 3),
        };
        let json = serde_json::to_string(&mode).unwrap();
        let back: RoomMode = serde_json::from_str(&json).unwrap();
        assert!((back.frequency - mode.frequency).abs() < 1e-10);
        assert_eq!(back.indices, mode.indices);
    }

    #[test]
    fn test_impulse_response_serde_roundtrip() {
        let ir = ImpulseResponse {
            samples: vec![0.0, 0.5, 0.3],
            sample_rate: 44100.0,
        };
        let json = serde_json::to_string(&ir).unwrap();
        let back: ImpulseResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.samples, ir.samples);
    }

    #[test]
    fn test_acoustic_analysis_serde_roundtrip() {
        let analysis = AcousticAnalysis {
            room_id: "test".into(),
            rt60: 1.0,
            mode_count: 10,
            schroeder_freq: 200.0,
            is_dead: false,
            is_live: false,
            is_balanced: true,
            problem_frequencies: vec![100.0],
        };
        let json = serde_json::to_string(&analysis).unwrap();
        let back: AcousticAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(back.room_id, "test");
        assert!(back.is_balanced);
    }

    #[test]
    fn test_engine_serde_roundtrip() {
        let engine = RoomAcousticsEngine::new(test_room());
        let json = serde_json::to_string(&engine).unwrap();
        let back: RoomAcousticsEngine = serde_json::from_str(&json).unwrap();
        assert_eq!(back.room.id, "test");
    }

    // ── FFT peaks at room modes ──

    #[test]
    fn test_fft_peaks_at_room_modes() {
        let room = test_room();
        let c = room.speed_of_sound();
        let f_mode = c / (2.0 * 5.0); // first axial mode
        let sr = 44100.0;
        let n = 4096;
        let mut samples = vec![0.0; n];
        for (i, s) in samples.iter_mut().enumerate() {
            *s = (2.0 * std::f64::consts::PI * f_mode * i as f64 / sr).sin() * 0.5;
        }
        let ir = ImpulseResponse {
            samples,
            sample_rate: sr,
        };
        let spec = ir.frequency_spectrum();
        let bin_resolution = sr / n as f64;
        let target_bin = (f_mode / bin_resolution) as usize;
        if target_bin > 0 && target_bin < spec.len() {
            assert!(spec[target_bin] > spec[1]); // peak should be above DC
        }
    }

    // ── Causality: direct sound before reflections ──

    #[test]
    fn test_causality_direct_before_reflection() {
        let room = test_room();
        let path = SignalPath {
            source: (1.0, 1.0, 1.5),
            listener: (4.0, 3.0, 1.5),
        };
        let c = room.speed_of_sound();
        let direct_delay = path.delay_ms(path.direct_distance(), c);
        let refl_delay = path.delay_ms(path.first_reflection_distance(&room), c);
        assert!(direct_delay < refl_delay);
    }

    // ── Additional theorem verifications ──

    #[test]
    fn test_rt60_positive() {
        let reverb = Reverb { room: test_room() };
        assert!(reverb.rt60() > 0.0);
    }

    #[test]
    fn test_edt_less_than_rt60() {
        let reverb = Reverb { room: test_room() };
        assert!(reverb.early_decay_time() < reverb.rt60());
    }

    #[test]
    fn test_c80_greater_than_c50() {
        let reverb = Reverb { room: test_room() };
        assert!(reverb.clarity_c80() > reverb.clarity_c50());
    }

    #[test]
    fn test_d50_between_zero_and_one() {
        let reverb = Reverb { room: test_room() };
        let d50 = reverb.definition_d50();
        assert!(d50 > 0.0 && d50 < 1.0);
    }

    #[test]
    fn test_mode_density_positive() {
        let sw = StandingWave { room: test_room() };
        assert!(sw.mode_density(100.0) > 0.0);
    }

    #[test]
    fn test_axial_modes_count() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.axial_modes(3);
        assert_eq!(modes.len(), 9); // 3 dimensions * 3 modes each
    }

    #[test]
    fn test_all_modes_has_mixed_types() {
        let sw = StandingWave { room: test_room() };
        let modes = sw.all_modes(300.0);
        let has_axial = modes.iter().any(|m| m.mode_type == ModeType::Axial);
        let has_tangential = modes.iter().any(|m| m.mode_type == ModeType::Tangential);
        let has_oblique = modes.iter().any(|m| m.mode_type == ModeType::Oblique);
        assert!(has_axial);
        assert!(has_tangential);
        assert!(has_oblique);
    }

    #[test]
    fn test_signal_path_same_point() {
        let path = SignalPath {
            source: (1.0, 2.0, 3.0),
            listener: (1.0, 2.0, 3.0),
        };
        assert!((path.direct_distance()).abs() < 1e-10);
    }

    #[test]
    fn test_empty_impulse_response() {
        let ir = ImpulseResponse {
            samples: vec![0.0; 10],
            sample_rate: 44100.0,
        };
        assert_eq!(ir.direct_sound_idx(), 10);
        assert!(ir.is_causal());
        assert!((ir.energy()).abs() < 1e-10);
    }

    #[test]
    fn test_acoustic_diff_volume_ratio() {
        let e1 = RoomAcousticsEngine::new(test_room());
        let e2 = RoomAcousticsEngine::new(big_room());
        let diff = e1.compare(&e2);
        // big room is 10*8*6=480, test room is 5*4*3=60, ratio=0.125
        assert!((diff.volume_ratio - 0.125).abs() < 0.01);
    }
}
