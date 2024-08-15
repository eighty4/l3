#[derive(Clone)]
pub struct PythonDeets {
    #[allow(unused)]
    is_slow_runtime: bool,
}

impl PythonDeets {
    fn read_is_slow_runtime() -> bool {
        true
    }
}

impl Default for PythonDeets {
    fn default() -> Self {
        Self {
            is_slow_runtime: true,
        }
    }
}
