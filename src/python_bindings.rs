use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;

use crate::error::RunomeError;
use crate::tokenizer::{Token as RustToken, TokenizeResult, Tokenizer as RustTokenizer};

/// Python wrapper for RunomeError
impl From<RunomeError> for PyErr {
    fn from(err: RunomeError) -> PyErr {
        PyException::new_err(format!("{:?}", err))
    }
}

/// Python Token class - mirrors Janome Token exactly
#[pyclass(name = "Token")]
#[derive(Clone)]
pub struct PyToken {
    inner: RustToken,
}

#[pymethods]
impl PyToken {
    /// surface property
    #[getter]
    fn surface(&self) -> String {
        self.inner.surface().to_string()
    }

    /// part_of_speech property
    #[getter]
    fn part_of_speech(&self) -> String {
        self.inner.part_of_speech().to_string()
    }

    /// infl_type property
    #[getter]
    fn infl_type(&self) -> String {
        self.inner.infl_type().to_string()
    }

    /// infl_form property
    #[getter]
    fn infl_form(&self) -> String {
        self.inner.infl_form().to_string()
    }

    /// base_form property
    #[getter]
    fn base_form(&self) -> String {
        self.inner.base_form().to_string()
    }

    /// reading property
    #[getter]
    fn reading(&self) -> String {
        self.inner.reading().to_string()
    }

    /// phonetic property
    #[getter]
    fn phonetic(&self) -> String {
        self.inner.phonetic().to_string()
    }

    /// node_type property
    #[getter]
    fn node_type(&self) -> String {
        format!("{:?}", self.inner.node_type())
    }

    /// String representation matching Janome format
    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    /// Debug representation
    fn __repr__(&self) -> String {
        format!(
            "Token(surface='{}', part_of_speech='{}')",
            self.inner.surface(),
            self.inner.part_of_speech()
        )
    }
}

impl PyToken {
    fn from_rust_token(token: RustToken) -> Self {
        PyToken { inner: token }
    }
}

/// Python iterator for tokenization results
#[pyclass(name = "TokenIterator")]
pub struct PyTokenIterator {
    results: Vec<TokenizeResult>,
    index: usize,
}

#[pymethods]
impl PyTokenIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<PyObject>> {
        if self.index >= self.results.len() {
            return Ok(None);
        }

        let result = &self.results[self.index];
        self.index += 1;

        Python::with_gil(|py| {
            match result {
                TokenizeResult::Token(token) => {
                    // Return PyToken object - Rust tokenizer decided this should be a token
                    Ok(Some(PyToken::from_rust_token(token.clone()).into_py_any(py)?))
                }
                TokenizeResult::Surface(surface) => {
                    // Return surface string - Rust tokenizer decided this should be wakati mode
                    Ok(Some(surface.clone().into_py_any(py)?))
                }
            }
        })
    }
}

/// Python Tokenizer class - mirrors Janome Tokenizer exactly
#[pyclass(name = "Tokenizer")]
pub struct PyTokenizer {
    inner: RustTokenizer,
}

#[pymethods]
impl PyTokenizer {
    /// Initialize Tokenizer with Janome-compatible parameters
    ///
    /// Args:
    ///     udic (str): User dictionary file path (default: '')
    ///     max_unknown_length (int): Maximum unknown word length (default: 1024)
    ///     wakati (bool): Wakati mode flag (default: False)
    #[new]
    #[pyo3(signature = (udic = "", max_unknown_length = 1024, wakati = false))]
    fn new(udic: &str, max_unknown_length: usize, wakati: bool) -> PyResult<Self> {
        // For now, ignore user dictionary parameter
        // TODO: Implement user dictionary loading
        if !udic.is_empty() {
            return Err(PyException::new_err("User dictionary not yet implemented"));
        }

        let tokenizer = RustTokenizer::new(Some(max_unknown_length), Some(wakati))
            .map_err(|e| PyException::new_err(format!("Failed to create tokenizer: {:?}", e)))?;

        Ok(PyTokenizer { inner: tokenizer })
    }

    /// Get version info to verify we're using the right code
    fn get_version_info(&self) -> String {
        format!("wakati_fix_v1_tokenizer_wakati_{}", self.inner.wakati())
    }

    /// Tokenize text with Janome-compatible parameters
    ///
    /// Args:
    ///     text (str): Input text to tokenize
    ///     wakati (bool): Override wakati mode (default: None)
    ///     baseform_unk (bool): Set base form for unknown words (default: True)
    ///
    /// Returns:
    ///     Iterator yielding Token objects (wakati=False) or strings (wakati=True)
    #[pyo3(signature = (text, wakati = None, baseform_unk = true))]
    fn tokenize(
        &self,
        text: &str,
        wakati: Option<bool>,
        baseform_unk: bool,
    ) -> PyResult<PyTokenIterator> {
        // Let the Rust tokenizer handle wakati precedence
        let results: Result<Vec<_>, _> = self
            .inner
            .tokenize(text, wakati, Some(baseform_unk))
            .collect();

        let token_results =
            results.map_err(|e| PyException::new_err(format!("Tokenization failed: {:?}", e)))?;

        Ok(PyTokenIterator {
            results: token_results,
            index: 0,
        })
    }

}

/// Python module definition
#[pymodule]
fn runome(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyToken>()?;
    m.add_class::<PyTokenizer>()?;
    m.add_class::<PyTokenIterator>()?;
    Ok(())
}
