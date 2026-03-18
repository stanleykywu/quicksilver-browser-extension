import json
import struct
import wave
from pathlib import Path

import fakepyrint


ROOT = Path(__file__).resolve().parents[2]
ASSETS_DIR = ROOT / "tests" / "assets"


def decode_pcm_frames(raw_frames: bytes, frame_count: int, channels: int, sample_width: int):
    total_samples = frame_count * channels
    if sample_width == 1:
        return [(sample - 128) / 127.0 for sample in raw_frames]
    if sample_width == 2:
        samples = struct.unpack(f"<{total_samples}h", raw_frames)
        return [sample / 32767.0 for sample in samples]
    if sample_width == 3:
        samples = []
        for i in range(0, len(raw_frames), 3):
            sample = int.from_bytes(raw_frames[i:i + 3], "little", signed=False)
            if sample >= 1 << 23:
                sample -= 1 << 24
            samples.append(sample / 8388607.0)
        return samples
    if sample_width == 4:
        samples = struct.unpack(f"<{total_samples}i", raw_frames)
        return [sample / 2147483647.0 for sample in samples]
    raise AssertionError(f"Unsupported PCM sample width {sample_width * 8} bits")


def read_wav_inputs(path: Path):
    with wave.open(str(path), "rb") as wav_file:
        assert wav_file.getcomptype() == "NONE", f"Expected uncompressed PCM WAV: {path}"

        channels = wav_file.getnchannels()
        sample_width = wav_file.getsampwidth()
        sample_rate = wav_file.getframerate()
        frame_count = wav_file.getnframes()
        raw_frames = wav_file.readframes(frame_count)

    samples = decode_pcm_frames(raw_frames, frame_count, channels, sample_width)
    frames = [
        [samples[i + ch] for ch in range(channels)]
        for i in range(0, len(samples), channels)
    ]
    if channels == 1:
        pcm_audio_2d = [[frame[0], frame[0]] for frame in frames]
    else:
        # The bindings expect stereo input, so keep the first two channels.
        pcm_audio_2d = [[frame[0], frame[1]] for frame in frames]
    pcm_audio_1d = [sample for frame in pcm_audio_2d for sample in frame]
    return sample_rate, pcm_audio_1d, pcm_audio_2d


def assert_fakeprint_close(actual, expected, label, tol=1e-6):
    assert isinstance(actual, list), f"{label}: expected list output"
    assert len(actual) == len(expected), (
        f"{label}: length mismatch {len(actual)} != {len(expected)}"
    )
    for idx, (act, exp) in enumerate(zip(actual, expected)):
        assert abs(act - exp) <= tol, (
            f"{label}: mismatch at index {idx}: {act} vs {exp}"
        )


def test_wav(path: Path):
    sample_rate, pcm_audio_1d, pcm_audio_2d = read_wav_inputs(path)

    fp_1d = fakepyrint.compute_fakeprint(pcm_audio_1d, sample_rate)
    fp_2d = fakepyrint.compute_fakeprint_2d(pcm_audio_2d, sample_rate)
    fp_1d_repeat = fakepyrint.compute_fakeprint(pcm_audio_1d, sample_rate)

    assert len(fp_1d) > 0, f"{path.name}: fakeprint should not be empty"
    assert_fakeprint_close(fp_2d, fp_1d, f"{path.name} 2d parity")
    assert_fakeprint_close(fp_1d_repeat, fp_1d, f"{path.name} determinism")

    fp_custom_1d = fakepyrint.compute_fakeprint(
        pcm_audio_1d,
        sample_rate,
        output_sample_rate=44100,
        f_range=(5000.0, 16000.0),
        duration=5,
    )
    fp_custom_2d = fakepyrint.compute_fakeprint_2d(
        pcm_audio_2d,
        sample_rate,
        output_sample_rate=44100,
        f_range=(5000.0, 16000.0),
        duration=5,
    )
    assert len(fp_custom_1d) > 0, f"{path.name}: custom fakeprint should not be empty"
    assert_fakeprint_close(fp_custom_2d, fp_custom_1d, f"{path.name} custom 2d parity")

    return fp_1d


def main():
    ai_fp = test_wav(ASSETS_DIR / "ai.wav")
    human_fp = test_wav(ASSETS_DIR / "human.wav")

    assert_fakeprint_close(
        ai_fp,
        json.loads((ASSETS_DIR / "aifp.json").read_text(encoding="utf-8")),
        "ai.wav fixture parity",
    )
    assert ai_fp != human_fp, "ai.wav and human.wav should not produce identical fakeprints"

    print("Python binding tests passed for ai.wav and human.wav")


if __name__ == "__main__":
    main()
