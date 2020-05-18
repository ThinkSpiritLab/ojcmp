use std::path::PathBuf;

#[derive(Debug)]
#[non_exhaustive]
pub enum CompareMode {
    Normal,
    Strict,
    SpjFloat
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
