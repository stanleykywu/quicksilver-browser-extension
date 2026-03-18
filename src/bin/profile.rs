#[cfg(not(feature = "web"))]
use ai_music_browser_detector::core;
#[cfg(feature = "web")]
use ai_music_browser_detector::web;
use hound;

fn main() {
    let input_path = std::env::args().nth(1).expect(
        format!(
            "Usage: {} <input.wav>",
            std::env::args()
                .nth(0)
                .unwrap_or_else(|| "cargo run --bin profile".into())
        )
        .as_str(),
    );
    let mut reader = hound::WavReader::open(&input_path).expect("Failed to open WAV file");
    let spec = reader.spec();
    assert!(
        spec.channels == 2,
        "must have 2 channels, got {} instead",
        spec.channels
    );
    assert!(
        spec.bits_per_sample == 16,
        "must be 16-bit audio, got {} bits per sample instead",
        spec.bits_per_sample
    );
    assert!(
        spec.sample_format == hound::SampleFormat::Int,
        "must be PCM audio, got {:?} instead",
        spec.sample_format
    );
    let samples = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect::<Vec<f32>>();

    #[cfg(not(feature = "web"))]
    {
        let fakeprint = core::compute_fakeprint(&samples, spec.sample_rate, None, None, None);
        assert!(!fakeprint.is_empty(), "Fakeprint should not be empty");
    }
    #[cfg(feature = "web")]
    {
        let prob = web::run_inference(&samples, spec.sample_rate).expect("Inference failed");
        assert!(prob >= 0.0); // use so that the compiler doesn't optimize away the inference code
    }
}
