//! The load model's inputs match their sources.

use cubesat_cad::loads::{FS, classroom_g, frequency, gevs_peak_g, gevs_psd};

#[test]
fn gevs_profile_breakpoints() {
    assert!((gevs_psd(20.0) - 0.026).abs() < 1e-9);
    assert!((gevs_psd(50.0) - 0.1625).abs() < 0.01);
    assert!((gevs_psd(400.0) - 0.16).abs() < 1e-9);
    assert!((gevs_psd(2000.0) - 0.0256).abs() < 0.001);
}

#[test]
fn gevs_profile_is_14_1_grms() {
    // Integrate the PSD from 20 to 2000 Hz in small steps.
    let (mut f, mut area) = (20.0_f64, 0.0);
    while f < 2000.0 {
        let step = 0.1;
        area += gevs_psd(f + step / 2.0) * step;
        f += step;
    }
    let grms = area.sqrt();
    assert!((grms - 14.1).abs() < 0.2, "{grms:.2} Grms");
}

#[test]
fn miles_response_and_frequency() {
    // 100 Hz, Q 10, 0.16 g2/Hz: sqrt(pi/2 * 100 * 10 * 0.16) = 15.85 Grms, 3 sigma 47.6 g.
    assert!((gevs_peak_g(100.0) - 47.55).abs() < 0.1);
    // 1 kN/mm on 1 kg: sqrt(1e6)/(2 pi) = 159.2 Hz.
    assert!((frequency(1000.0, 1.0) - 159.15).abs() < 0.1);
}

#[test]
fn factors_follow_their_sources() {
    assert!((FS - 2.0).abs() < 1e-9);
    assert!((classroom_g() - 50.0).abs() < 1e-9);
}
