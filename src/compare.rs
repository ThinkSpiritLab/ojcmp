use std::path::PathBuf;

#[derive(Debug)]
pub enum CompareMode {
    Normal,
    Strict,
}

pub struct CompareTask {
    pub std_path: PathBuf,
    pub user_path: Option<PathBuf>,
    pub user_read_all: bool,
    pub mode: CompareMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    AC,
    WA,
    PE,
}

pub trait Comparer {
    fn exec(&self, task: &CompareTask) -> Comparison;
}
