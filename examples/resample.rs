use hound;

fn main() {
    let input_path = std::env::args()
        .nth(1)
        .expect("Usage: cargo run --example resample -- <input.wav>");
    println!("Testing resampling on file: {}", input_path);
    let mut reader = hound::WavReader::open(&input_path).expect("Failed to open WAV file");
    let spec = reader.spec();
    let samples = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect::<Vec<f32>>();
    println!(
        "Original sample rate: {}, number of samples: {}",
        spec.sample_rate,
        samples.len()
    );
    let audio_slice = ai_music_browser_detector::core::fakeprint::open_audio_slice(&samples);
    let resampled = ai_music_browser_detector::core::fakeprint::resample_audio(
        &audio_slice,
        spec.sample_rate,
        44100,
    );
    assert_eq!(resampled.shape()[1], spec.channels as usize); // should have the same number of channels
    let expected = (samples.len() / spec.channels as usize) * 44100 / spec.sample_rate as usize;
    assert!((resampled.shape()[0] as isize - expected as isize).abs() <= 1);
    println!("Resampled to {}Hz shape: {:?}", 44100, resampled.shape());

    println!("Saving resampled audio to a WAV file for manual inspection...");
    // save resampled audio to a wav file for manual inspection
    let temp_file = format!(
        "{}-resampled-{}hz.wav",
        &input_path.trim_end_matches(".wav"),
        44100
    );
    let new_spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer =
        hound::WavWriter::create(&temp_file, new_spec).expect("Failed to create WAV writer");
    for frame in resampled.rows() {
        for &sample in frame {
            let s = sample.clamp(-1.0, 1.0);
            let pcm = (s * 32767.0) as i16;
            writer.write_sample(pcm).expect("Failed to write sample");
        }
    }
    writer.finalize().expect("Failed to finalize WAV file");
    println!("Resampled audio saved to {}", &temp_file);
}
