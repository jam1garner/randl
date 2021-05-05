use std::path::PathBuf;
use structopt::StructOpt;

use randl_core::RandlFile;

#[derive(StructOpt)]
struct Args {
    kdl: PathBuf,
    prc: PathBuf,
    prc_out: PathBuf,
}

fn main() {
    let args = Args::from_args();
    let randl = RandlFile::open(args.kdl);
}
