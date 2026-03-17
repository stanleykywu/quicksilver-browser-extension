import argparse
import os
import random
from concurrent.futures import ProcessPoolExecutor, as_completed

import fakepyrint
import numpy as np
import torchaudio
from tqdm.auto import tqdm


def process_file(audio_fp, crop=False, crop_length=30):
    audio_raw, sr = torchaudio.load(audio_fp, channels_first=False)
    audio_raw = audio_raw.numpy()

    if audio_raw.shape[1] == 1:
        audio_raw = np.repeat(audio_raw, 2, axis=1)

    # randomly crop
    if crop:
        segment_length = int(crop_length * sr)  # crop_length seconds in samples
        total_length = audio_raw.shape[0]

        if total_length > segment_length:
            start = random.randint(0, total_length - segment_length)
            audio_raw = audio_raw[start : start + segment_length]

    fakeprint = fakepyrint.compute_fakeprint_2d(audio_raw, sr)
    return np.array(fakeprint)


def main(args):
    audio_fps = ...  # TODO: list of audio fps to compute fakeprints for

    results_dict = {}
    with ProcessPoolExecutor(max_workers=os.cpu_count()) as executor:
        futures = [
            executor.submit(process_file, fp, args.crop, args.crop_length)
            for fp in audio_fps
        ]

        for future in tqdm(as_completed(futures), total=len(futures)):
            computed_fakeprint = future.result()
            results_dict[...] = computed_fakeprint

    np.save(args.output_fp, results_dict)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output_fp", type=str, required=True)
    parser.add_argument("--crop", action="store_true")
    parser.add_argument("--crop_length", type=int, default=30)
    args = parser.parse_args()
    main(args)
