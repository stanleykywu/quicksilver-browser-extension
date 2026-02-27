use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn average_pcm32(pcm: &[f32]) -> f32 {
    if pcm.is_empty() {
        return 0.0;
    }

    let sum: f32 = pcm.iter().copied().sum();
    sum / pcm.len() as f32
}
