use ojcmp::compare::{CompareMode, CompareTask, Comparer, Comparison};
use ojcmp::comparers::{NormalComparer, SpjFloatComparer, StrictComparer};

use std::path::PathBuf;
use structopt::StructOpt;

fn parse_mode(s: &str) -> Result<CompareMode, &'static str> {
    match s {
        "normal" => Ok(CompareMode::Normal),
        "strict" => Ok(CompareMode::Strict),
        "spj_float" => Ok(CompareMode::SpjFloat),
        _ => Err("Unknown mode"),
    }
}

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(name = "std", short = "s", long, value_name = "path")]
    /// Std file path
    pub std_path: PathBuf,

    #[structopt(name = "user", short = "u", long, value_name = "path")]
    /// User file path. Reads from stdin if it's not given
    pub user_path: Option<PathBuf>,

    #[structopt(name = "all", short = "a", long)]
    /// Reads all bytes of user file even if it's already WA
    pub read_all: bool,

    #[structopt(
        name = "mode",
        short = "m",
        long,
        default_value = "normal",
        parse(try_from_str = parse_mode)
    )]
    /// CompareMode ("normal"|"strict"|"spj_float")
    pub mode: CompareMode,

    #[structopt(name = "backtrace", short = "b", long)]
    /// Prints stack backtrace when fatal error occurs
    pub backtrace: bool,

    #[structopt(name = "eps", long)]
    /// Eps for float comparing
    pub eps: Option<f64>,
}

fn main() {
    let args = Opt::from_args();

    ojcmp::error::set_backtrace(args.backtrace);

    let comparer: Box<dyn Comparer> = match args.mode {
        CompareMode::Normal => Box::new(NormalComparer::new()),
        CompareMode::Strict => Box::new(StrictComparer::new()),
        CompareMode::SpjFloat => Box::new(SpjFloatComparer::new(args.eps.unwrap_or(f64::EPSILON))),
        _ => unreachable!(),
    };

    let task = CompareTask {
        std_path: args.std_path,
        user_path: args.user_path,
        user_read_all: args.read_all,
        mode: args.mode,
    };

    let ans = comparer.exec(&task);

    let output = match ans {
        Comparison::AC => "AC",
        Comparison::WA => "WA",
        Comparison::PE => "PE",
    };

    println!("{}", output);
}
