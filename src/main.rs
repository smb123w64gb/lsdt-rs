use clap::Parser;
use std::path::PathBuf;

mod ls;
mod rf;

mod extract;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    ls: PathBuf,
    dt: PathBuf,
    dt1: Option<PathBuf>,
    #[clap(short, long, default_value = "Out")]
    out_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    extract::extract(args.ls, args.dt, args.dt1, args.out_dir);
}
