#[derive(Clone)]
pub struct PythonConfig {
    #[allow(unused)]
    is_slow_runtime: bool,
}

impl PythonConfig {
    #[allow(unused)]
    pub fn read_is_slow_runtime() -> bool {
        true
    }
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            is_slow_runtime: true,
        }
    }
}
