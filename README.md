# Quicksilver Browser Extension

This is a browser extension that you can use on Google Chrome or Microsoft Edge, to quickly check if music you are listening to is AI-generated. This browser extension works by directly analyzing music audio from any online streaming source (YouTube, Spotify, Apple Music, ..., etc.) for artifacts music commonly found in AI-generated music. In other words, if your browser can play it, Quicksilver can directly classify the audio without you needing to download it or upload it anywhere. If you are looking for our Mac app, please look [here](https://github.com/stanleykywu/quicksilver-macos).

## Downloading

You can download Quicksilver into the browser from the official Google Chrome or Microsoft Edge extension stores:
* [Google Chrome](https://chromewebstore.google.com/detail/quicksilver/ikahnkjmdjoikhpcbpelagokjnkmpjdm)
* [Microsoft Edge](https://microsoftedge.microsoft.com/addons/detail/quicksilver/hacidakkfmemkmkjlkleegmlajbnfoga)

## Developing Quicksilver Browser Extension

This repo contains code for the Quicksilver browser extension, using Rust + WASM. Our code is composed of three modules:

1. Core Rust libraries for computing an audio sample's _fakeprint_, which are the features used to train our model, found under `src/core`
2. Python bindings for exporting the fakeprint computation as the Python package `fakepyrint`, found under `src/python`. These bindings are used for training the logistic regression used for inference in Python.
3. WebAssembly bindings for running end-to-end inference with the latest model, found under `src/web`.

Additionally, the `chromium/` folder contains the publicly available web extension. Note that by default, the WASM bindings are not included in the `chromium/` folder. To install the bindings, run `./scripts/build.sh web` from the root directory (see [Building Quicksilver](#building-quicksilver)).

We have provided a set of shell scripts to make it easy to contribute to Quicksilver. These scripts assume you have [Rust](https://rust-lang.org/tools/install/) (along with Cargo) and [uv](https://docs.astral.sh/uv/getting-started/installation/) already installed on your computer. uv is not needed if you don't plan on installing the Python bindings.

### Building Quicksilver

To build the core fakeprint libraries, run `./scripts/build.sh core`. 
To build the web exported inference package, run `./scripts/build.sh web`.
To build the Python bindings for model training, run `./scripts/build.sh python`
To build everything, run `./scripts/build.sh all`.

To install the browser extension, you can follow the guides for your respective browser. See guides for [Chrome](https://developer.chrome.com/docs/extensions/get-started/tutorial/hello-world#load-unpacked) and [Edge](https://learn.microsoft.com/en-us/microsoft-edge/extensions/getting-started/extension-sideloading). When choosing a file directory to load, choose `./chromium` as the directory.  

### Testing Quicksilver

To run unit tests for the core fakeprint modules, run `./tests/scripts/core.sh`.
To also run unit tests for the web module (including Chromedriver integration tests), run `./tests/scripts/web.sh`. 
To run unit tests for Fakepyrint, run `./tests/scripts/python.sh`.

We also provide two additional utilities for testing. 
  
`./tests/scripts/resample.sh` calls the standalone Rust executable in `src/bin/resample.rs`. Since resampling is a lossy operation, the easiest way to verify the resampling operation done in fakeprint computation is to listen to it. This provides a convenient script to produce resampled versions of input .WAV files for manual inspection.

`./tests/scripts/profile.sh` provides a script for performance profiling. The profiler uses [samply](https://crates.io/crates/samply) which conveniently requires zero external dependencies. If samply is not installed, the script will automatically install it. The profiled executable is `src/bin/profile.rs`. To profile the performance of the end-to-end inference, run `./tests/scripts/profile.sh web path/to/input.wav`. To profile just the fakeprint computation, run `./tests/scripts/profile.sh core path/to/input.wav`

### Fakepyrint

To compute the fakeprint from Python, we have provided convenient Python bindings around the core libraries, available as the locally installed package `fakepyrint` (install via `./scripts/build.sh python`). The package features two functions: `fakepyrint.compute_fakeprint` which computes the fakeprint of a flattened PCM audio array (an audio source with `M` channels and `N` samples should be interleaved as `[S_1_CH_1, ..., S_1_CH_M, ..., S_N_CH_1, S_N, CH_M]`) and `fakepyrint.compute_fakeprint_2d`, which computes the fakeprint of a PCM audio array of the form `[N;M]` where `N` is the audio samples and `M` is the channels. Example usage of the library can be found in `examples/fakeprint.py`.

### Building Custom Models

We provide a set of models that we trained ourselves in the `model/` folder. The web package utilizes the most recent of these models for inference. These models are trained using `sklearn.linear_model.LogisticRegression`, using Fakepyrint for feature extraction. The model is then converted to CBOR to be interoperable with Rust. 

If you would like to train your own model, we provide a serialization utility under `./scripts/serialize-model.py` that takes a pickle of a `sklearn.linear_model.LogisticRegression` and saves it as a CBOR. The path of `MODEL_BYTES` in `src/web/model.rs` can then be modified to point to the path of your custom model. To make it easier to generate quickly generate fakeprints from real audio sources for model training/testing, we also provide `./scripts/script-generate-fakeprints-multiprocessing.py`.
