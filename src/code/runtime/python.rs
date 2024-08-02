#[derive(Clone)]
pub struct PythonDeets {
    #[allow(unused)]
    is_slow_runtime: bool,
}

impl PythonDeets {
    pub fn read_details() -> Result<Self, anyhow::Error> {
        let is_slow_runtime = Self::read_is_slow_runtime();
        debug_assert!(is_slow_runtime);
        Ok(Self { is_slow_runtime })
    }
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
