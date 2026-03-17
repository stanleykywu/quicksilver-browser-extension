use crate::core::compute_fakeprint;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction(name = "compute_fakeprint")]
#[pyo3(signature = (pcm_audio, input_sample_rate, output_sample_rate=None, f_range=None))]
fn py_compute_fakeprint(
    pcm_audio: Vec<f32>,
    input_sample_rate: u32,
    output_sample_rate: Option<u32>,
    f_range: Option<(f32, f32)>,
) -> PyResult<Vec<f32>> {
    if pcm_audio.is_empty() {
        return Err(PyValueError::new_err("pcm_audio is empty"));
    }

    Ok(compute_fakeprint(&pcm_audio, input_sample_rate, output_sample_rate, f_range).to_vec())
}

pub fn register_python_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(py_compute_fakeprint, module)?)?;
    Ok(())
}

#[pymodule]
pub fn fakepyrint(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    register_python_module(module)?;
    Ok(())
}
