use pyo3_file::PyFileLikeObject;
use pyo3::types::PyString;

use pyo3::prelude::*;
use std::io::Read;
use std::fs::File;

/// Represents either a path `File` or a file-like object `FileLike`
#[derive(Debug)]
enum FileOrFileLike {
    File(String),
    FileLike(PyFileLikeObject),
}

impl FileOrFileLike {
    pub fn from_pyobject(path_or_file_like: PyObject) -> PyResult<FileOrFileLike> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // is a path
        if let Ok(string_ref) = path_or_file_like.cast_as::<PyString>(py) {
            return Ok(FileOrFileLike::File(
                string_ref.to_string_lossy().to_string(),
            ));
        }

        // is a file-like
        match PyFileLikeObject::with_requirements(path_or_file_like, true, false, true) {
            Ok(f) => Ok(FileOrFileLike::FileLike(f)),
            Err(e) => Err(e)
        }
    }
}

/// Opens a file or file-like, and reads it to string.
pub fn read_path_or_file_like(
    path_or_file_like: PyObject,
) -> PyResult<String> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    match FileOrFileLike::from_pyobject(path_or_file_like) {
        Ok(f) => match f {
            FileOrFileLike::File(s) => {
                println!("It's a file! - path {}", s);
                let mut f = File::open(s)?;
                let mut string = String::new();

                let read = f.read_to_string(&mut string);
                Ok(string)
            }
            FileOrFileLike::FileLike(mut f) => {
                println!("Its a file-like object");
                let mut string = String::new();

                let read = f.read_to_string(&mut string);
                Ok(string)
            }
        },
        Err(e) => Err(e),
    }
}
