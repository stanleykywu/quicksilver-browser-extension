import argparse
import glob
import os
import random
import traceback
from concurrent.futures import ProcessPoolExecutor, as_completed

import fakepyrint
import numpy as np
import torchaudio
from loguru import logger
from tqdm.auto import tqdm


def process_file(audio_fp, crop=False, crop_length=30, min_length=None):
    try:
        audio_raw, sr = torchaudio.load(audio_fp, channels_first=False)
        audio_raw = audio_raw.numpy()

        if audio_raw.shape[1] == 1:
            audio_raw = np.repeat(audio_raw, 2, axis=1)

        # randomly crop
        if crop:
            desired_length = int(crop_length * sr)  # crop_length seconds in samples
            total_length = audio_raw.shape[0]

            # decide if too short
            if min_length:
                min_segment_length = int(min_length * sr)

                if total_length < min_segment_length:
                    return None

            if total_length >= desired_length:
                start = random.randint(0, total_length - desired_length)
                audio_raw = audio_raw[start : start + desired_length]

        fakeprint = fakepyrint.compute_fakeprint_2d(audio_raw, sr)
        return audio_fp, np.array(fakeprint)
    except Exception:
        logger.error(f"Error processing {audio_fp}: {traceback.format_exc()}")
        return None


def main(args):
    audio_fps = glob.glob(
        args.input_audio_glob
    )  # list of audio fps to compute fakeprints for

    if args.sample is not None:
        audio_fps = random.sample(audio_fps, args.sample)

    results_dict = {}
    with ProcessPoolExecutor(max_workers=args.num_workers) as executor:
        futures = [
            executor.submit(
                process_file, fp, args.crop, args.crop_length, args.min_length
            )
            for fp in audio_fps
        ]

        for future in tqdm(as_completed(futures), total=len(futures)):
            result = future.result()

            # if function returns None, skip
            if not result:
                continue

            audio_fp, computed_fakeprint = result
            results_dict[audio_fp] = computed_fakeprint

    np.save(args.output_fp, results_dict)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--input_audio_glob", type=str, required=True)
    parser.add_argument("--num_workers", type=int, default=os.cpu_count())
    parser.add_argument("--sample", type=int, default=None)
    parser.add_argument("--output_fp", type=str, required=True)
    parser.add_argument("--crop", action="store_true")
    parser.add_argument("--min_length", type=int, default=None)
    parser.add_argument("--crop_length", type=int, default=30)
    args = parser.parse_args()
    main(args)
