#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {
        RuntimeError::new(
            file!(), line!(), column!(), format_args!($($arg)*).to_string()
        )
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    file: &'static str,
    line: u32,
    column: u32,
    message: String,
}

impl RuntimeError {
    #[inline]
    pub fn new(file: &'static str, line: u32, column: u32, message: String) -> Self {
        Self { file, line, column, message }
    }

    #[inline]
    pub fn what(&self) -> &str {
        &self.message
    }

    #[inline]
    pub fn debug_info(&self) -> String {
        format!("[{}::{}::{}]>>{}", self.file, self.line, self.column, self.message)
    }
}
