use koicore::{
    parser::{
        StringInputSource,
    },
    Parser, ParserConfig, Command, ParseError,
};

use pyo3::prelude::*;
use pyo3::conversion::IntoPyObjectExt;
use pyo3::exceptions::{PyValueError, PyRuntimeError};

#[pyclass]
pub struct PyParser {
    parser: Parser<StringInputSource>,
}

#[pymethods]
impl PyParser {
    #[new]
    #[pyo3(signature = (source, command_threshold=1))]
    fn new(source: String, command_threshold: usize) -> PyResult<Self> {
        let config = ParserConfig { command_threshold };
        let input = StringInputSource::new(&source);
        Ok(Self {
            parser: Parser::new(input, config)
        })
    }

    fn next_command(&mut self) -> PyResult<Option<PyCommand>> {
        match self.parser.next_command() {
            Ok(Some(cmd)) => Ok(Some(PyCommand { inner: cmd })),
            Ok(None) => Ok(None),
            Err(e) => Err(convert_parse_error(e))
        }
    }

    fn current_line(&self) -> usize {
        self.parser.current_line()
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<PyCommand>> {
        slf.next_command()
    }
}

#[pyclass]
pub struct PyCommand {
    inner: Command,
}

#[pymethods]
impl PyCommand {
    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn params(&self) -> Vec<PyParameter> {
        self.inner.params()
            .iter()
            .map(|p| PyParameter { inner: p.clone() })
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("Command(name='{}', params={})",
                self.inner.name(),
                self.inner.params().len())
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyParameter {
    inner: koicore::Parameter,
}

#[pymethods]
impl PyParameter {
    fn is_basic(&self) -> bool {
        matches!(self.inner, koicore::Parameter::Basic(_))
    }

    fn is_composite(&self) -> bool {
        matches!(self.inner, koicore::Parameter::Composite(_, _))
    }

    fn as_value(&self) -> PyResult<PyValue> {
        match &self.inner {
            koicore::Parameter::Basic(v) => Ok(PyValue { inner: v.clone() }),
            _ => Err(PyValueError::new_err("Not a basic parameter"))
        }
    }

    fn as_composite(&self) -> PyResult<(String, PyObject)> {
        match &self.inner {
            koicore::Parameter::Composite(name, value) => {
                Python::with_gil(|py| {
                    let py_value = convert_composite_value(py, value)?;
                    Ok((name.clone(), py_value))
                })
            }
            _ => Err(PyValueError::new_err("Not a composite parameter"))
        }
    }

    fn __repr__(&self) -> String {
        format!("Parameter({})", self.inner)
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyValue {
    inner: koicore::Value,
}

#[pymethods]
impl PyValue {
    fn as_int(&self) -> PyResult<i64> {
        match self.inner {
            koicore::Value::Int(i) => Ok(i),
            _ => Err(PyValueError::new_err("Not an integer"))
        }
    }

    fn as_float(&self) -> PyResult<f64> {
        match self.inner {
            koicore::Value::Float(f) => Ok(f),
            _ => Err(PyValueError::new_err("Not a float"))
        }
    }

    fn as_string(&self) -> PyResult<String> {
        match &self.inner {
            koicore::Value::String(s) => Ok(s.clone()),
            _ => Err(PyValueError::new_err("Not a string"))
        }
    }

    fn as_literal(&self) -> PyResult<String> {
        match &self.inner {
            koicore::Value::Literal(s) => Ok(s.clone()),
            _ => Err(PyValueError::new_err("Not a literal"))
        }
    }

    fn to_python(&self, py: Python) -> PyObject {
        match &self.inner {
            koicore::Value::Int(i) => i.into_py_any(py).unwrap(),
            koicore::Value::Float(f) => f.into_py_any(py).unwrap(),
            koicore::Value::String(s) => s.into_py_any(py).unwrap(),
            koicore::Value::Literal(s) => s.into_py_any(py).unwrap(),
        }
    }

    fn __repr__(&self) -> String {
        format!("Value({})", self.inner)
    }
}

fn convert_parse_error(error: Box<ParseError>) -> PyErr {
    let message = error.message();
    if let Some((line, col)) = error.position() {
        PyValueError::new_err(format!(
            "Parse error at line {}, column {}: {}",
            line, col, message
        ))
    } else {
        PyValueError::new_err(message)
    }
}

fn convert_composite_value(py: Python, value: &koicore::command::CompositeValue) -> PyResult<PyObject> {
    use koicore::command::CompositeValue;
    match value {
        CompositeValue::Single(v) => {
            Ok(PyValue { inner: v.clone() }.to_python(py))
        }
        CompositeValue::List(values) => {
            let list = values.iter()
                .map(|v| PyValue { inner: v.clone() }.to_python(py))
                .collect::<Vec<_>>();
            Ok(list.into_py_any(py)?)
        }
        CompositeValue::Dict(pairs) => {
            let dict = pyo3::types::PyDict::new(py);
            for (k, v) in pairs {
                dict.set_item(k, PyValue { inner: v.clone() }.to_python(py))?;
            }
            Ok(dict.into_py_any(py)?)
        }
    }
}

#[pymodule]
fn koicore_py(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyParser>()?;
    module.add_class::<PyCommand>()?;
    module.add_class::<PyParameter>()?;
    module.add_class::<PyValue>()?;
    Ok(())
}