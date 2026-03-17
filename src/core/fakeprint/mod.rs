use scirs2_core::ndarray::{Array1, Array2, Array3, s};
use scirs2_signal::resampling::resample;

mod stft;
#[allow(unused_imports)]
use stft::{N_FFT, get_stft};
mod curve;
use curve::{DEFAULT_F_RANGE, curve_profile};

// TODO: save audio slice as a wav to confirm it's the right audio, create e2e test with browser extension

const NUM_CHANNELS: usize = 2;
const DEFAULT_SAMPLE_RATE: u32 = 44100; // hz
const DURATION: u32 = 30; // seconds
const NORMALIZE_MAX_DB: f32 = 5.0; // dB

/// Open an audio slice for processing, given the raw
/// PCM float 32 data and the sample rate.
/// Returns a 2d array of shape [time, channels] for further processing.
pub fn open_audio_slice(pcm_audio: &[f32]) -> Array2<f32> {
    let n_samples = pcm_audio.len() / NUM_CHANNELS;
    // Convert to a 2d ndarray for processing
    Array2::from_shape_vec((n_samples, NUM_CHANNELS), pcm_audio.to_vec())
        .expect("Failed to convert PCM audio to 2D array") // returns shape [time, channels]
}
/// Resample an audio slice with shape [time, channels] to the target sample rate, if needed.
pub fn resample_audio(audio_slice: &Array2<f32>, input_rate: u32, output_rate: u32) -> Array2<f32> {
    if audio_slice.shape()[1] != NUM_CHANNELS {
        panic!(
            "Expected audio slice to have {} channels, but got {}",
            NUM_CHANNELS,
            audio_slice.shape()[1]
        );
    }

    if input_rate == output_rate {
        return audio_slice.clone();
    }

    let mut resampled_channels = Vec::with_capacity(NUM_CHANNELS);
    let audio_slice_t = audio_slice.t(); // transpose to shape [channels, time]
    for channel in audio_slice_t.outer_iter() {
        // resample() expects f64 input, so we need to upcast the audio data before resampling
        let upcasted_channel: Vec<f64> = channel.iter().map(|&x| x as f64).collect();
        let resampled_channel: Vec<f32> = resample(
            &upcasted_channel,
            input_rate as f64,
            output_rate as f64,
            None,
        )
        .expect("Failed to resample audio")
        .iter()
        .map(|&x| x as f32)
        .collect();
        resampled_channels.push(resampled_channel);
    }
    let n_samples = resampled_channels[0].len();
    // convert back to 2d array
    Array2::from_shape_vec(
        (NUM_CHANNELS, n_samples),
        resampled_channels.into_iter().flatten().collect(),
    )
    .expect("Failed to convert resampled audio to 2D array")
    .reversed_axes() // return shape [time, channels]
}

pub fn spectrogram(
    pcm_audio: &[f32],
    input_sample_rate: u32,
    output_sample_rate: Option<u32>,
    max_duration: Option<u32>,
) -> Array3<f32> {
    let output_sample_rate = output_sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);
    let max_duration = max_duration.unwrap_or(DURATION);

    let audio_slice = if output_sample_rate != input_sample_rate {
        let audio_slice = open_audio_slice(pcm_audio);
        resample_audio(&audio_slice, input_sample_rate, output_sample_rate)
    } else {
        open_audio_slice(pcm_audio)
    };

    // get only the first x seconds of audio for spectrogram computation
    let n_samples = (output_sample_rate * max_duration) as usize;
    // if the audio is shorter than max_duration, we will just get the spectrogram of the whole audio
    let slice_end = audio_slice.shape()[0].min(n_samples);
    let audio_slice = audio_slice.slice(s![..slice_end, ..]).to_owned();

    get_stft(&audio_slice)
}

pub fn max_normalize(x: &Array1<f32>, max_db: Option<f32>) -> Array1<f32> {
    let max_db = max_db.unwrap_or(NORMALIZE_MAX_DB);
    let x = x.clamp(0.0, max_db);
    let max_val = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    x / (1e-6 + max_val)
}

pub fn fakeprint(
    stft: &Array3<f32>,
    f_range: Option<(f32, f32)>,
    sample_rate: Option<u32>,
) -> Array1<f32> {
    let sample_rate = sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);
    let f_range = f_range.unwrap_or(DEFAULT_F_RANGE);
    let (chs, n_bins, n_frames) = stft.dim();
    let mut fp = Array1::<f32>::zeros(n_bins);
    for bin in 0..n_bins {
        let mut sum = 0.0;
        for frame in 0..n_frames {
            for ch in 0..chs {
                sum += stft[[ch, bin, frame]];
            }
        }
        fp[bin] = sum / (chs * n_frames) as f32;
    }

    let x_real = Array1::linspace(0.0, (sample_rate as f32) / 2.0, fp.len());
    let (_, fp_curve) = curve_profile(&x_real, &fp, Some(f_range), None);
    max_normalize(&fp_curve, None)
}

pub fn compute_fakeprint(
    pcm_audio: &[f32],
    input_sample_rate: u32,
    output_sample_rate: Option<u32>,
    f_range: Option<(f32, f32)>,
) -> Array1<f32> {
    if pcm_audio.is_empty() {
        panic!("pcm_audio is empty");
    }
    if pcm_audio.len() / NUM_CHANNELS < N_FFT {
        panic!(
            "pcm_audio is too short: expected at least {} samples for {} channels, but got {} samples",
            N_FFT,
            NUM_CHANNELS,
            pcm_audio.len() / NUM_CHANNELS
        );
    }
    let spectro = spectrogram(pcm_audio, input_sample_rate, output_sample_rate, None);
    fakeprint(&spectro, f_range, output_sample_rate)
}

#[cfg(test)]
mod tests {

    use super::*;

    use hound;

    fn test_wav(file_path: &str) -> Result<(Vec<f32>, Vec<f32>), hound::Error> {
        let mut reader = hound::WavReader::open(file_path).expect("Failed to open WAV file");
        let spec = reader.spec();
        let samples = reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect::<Vec<f32>>();
        let audio_slice = open_audio_slice(&samples);
        let temp_file = format!("{}_temp.wav", file_path);
        let mut writer =
            hound::WavWriter::create(&temp_file, spec).expect("Failed to create WAV writer");
        for frame in audio_slice.rows() {
            for &sample in frame {
                let s = sample.clamp(-1.0, 1.0);
                let pcm = (s * 32767.0) as i16;
                writer.write_sample(pcm)?;
            }
        }
        writer.finalize()?;

        let mut recon_reader =
            hound::WavReader::open(&temp_file).expect("Failed to open reconstructed WAV file");
        let recon_samples = recon_reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect::<Vec<f32>>();
        // delete the test output file
        std::fs::remove_file(&temp_file).expect("Failed to delete test output WAV file");
        Ok((samples, recon_samples))
    }

    #[test]
    fn open_audio_slice1() {
        let pcm_audio = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]; // 3 samples of stereo audio
        let audio_slice = open_audio_slice(&pcm_audio);
        assert_eq!(audio_slice.shape(), &[3, 2]);
        let left_expected = vec![0.0, 0.2, 0.4];
        let right_expected = vec![0.1, 0.3, 0.5];
        assert_eq!(audio_slice.column(0).to_vec(), left_expected);
        assert_eq!(audio_slice.column(1).to_vec(), right_expected);
    }
    #[test]
    fn open_audio_slice2() {
        let mut pcm_audio = Vec::with_capacity(200); // Empty audio
        for i in 0..200 {
            pcm_audio.push(i as f32 * 0.1);
        }
        let audio_slice = open_audio_slice(&pcm_audio);
        assert_eq!(audio_slice.shape(), &[100, 2]);
        let left_expected = pcm_audio.iter().step_by(2).cloned().collect::<Vec<f32>>();
        let right_expected = pcm_audio
            .iter()
            .skip(1)
            .step_by(2)
            .cloned()
            .collect::<Vec<f32>>();
        assert_eq!(audio_slice.column(0).to_vec(), left_expected);
        assert_eq!(audio_slice.column(1).to_vec(), right_expected);
    }

    #[test]
    fn check_reconstruction1() {
        // skip test if the file doesn't exist
        if !std::path::Path::new("tests/test1-48000hz.wav").exists() {
            eprintln!("Skipping test_check_reconstruction1 since test WAV file doesn't exist");
            return;
        }
        let (orig_samples, recon_samples) = test_wav("tests/test1-48000hz.wav").unwrap();
        assert_eq!(recon_samples.len(), orig_samples.len());
        for (recon, orig) in recon_samples.iter().zip(orig_samples.iter()) {
            assert!(
                (recon - orig).abs() < 1e-5,
                "Reconstructed sample differs from original: recon={}, orig={}",
                recon,
                orig
            );
        }
    }

    #[test]
    fn test_downsampling() {
        let pcm_audio = vec![0.0, 0.1, 0.2, 0.3]; // 2 samples of stereo audio
        let audio_slice = open_audio_slice(&pcm_audio);
        let resampled = resample_audio(&audio_slice, 44100, 22050);
        assert_eq!(resampled.shape(), &[1, 2]); // should have 1 sample after downsampling
    }
    #[test]
    fn test_upsampling() {
        let pcm_audio = vec![0.0, 0.1, 0.2, 0.3]; // 2 samples of stereo audio
        let audio_slice = open_audio_slice(&pcm_audio);
        let resampled = resample_audio(&audio_slice, 22050, 44100);
        assert_eq!(resampled.shape(), &[4, 2]); // should have 4 samples after upsampling
    }

    #[test]
    fn test_e2e_no_errors() {
        let pcm_audio = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]; // 3 samples of stereo audio
        // repeat N_FFT times to ensure we have enough samples for the spectrogram
        let pcm_audio = pcm_audio
            .into_iter()
            .cycle()
            .take(2 * NUM_CHANNELS * N_FFT)
            .collect::<Vec<f32>>();
        let fakeprint = compute_fakeprint(&pcm_audio, 44100, None, None);
        assert_eq!(fakeprint.len(), 4087); // should have 4087 frequency bins for N_FFT=16384
    }
}
