use std::path::PathBuf;
use structopt::StructOpt;
use owo_colors::OwoColorize;

use randl_core::RandlFile;
use randl_core::prc;

#[derive(StructOpt)]
struct Args {
    #[structopt(help = "the KDL file to use as a randomization template")]
    kdl: PathBuf,
    #[structopt(help = "the PRC file to read from disk to apply the template to")]
    prc: PathBuf,
    #[structopt(help = "the file to write the resulting prc to")]
    prc_out: PathBuf,

    #[structopt(
        short, long,
        help = "The file node to apply to the given prc file"
    )]
    file: Option<String>,
}

fn main() {
    let args = Args::from_args();
    let randl = match RandlFile::open(args.kdl) {
        Ok(randl) => randl,
        Err(e) => {
            println!("{} {}", "Parse Error:".bright_red(), e.bright_red());
            return
        }
    };

    let idx = match args.file {
        Some(file) => match randl.entries.iter().position(|entry| entry.prc_name == file) {
            Some(x) => x,
            None => {
                println!("{}", "Warning: file not found, defaulting to first `file` node.".bright_yellow());
                0
            }
        }
        None => 0,
    };

    let mut prc = prc::open(&args.prc).unwrap();
    if let Err(e) = randl.entries[idx].apply(&mut prc, &randl.sets) {
        println!("{} {}", "Eval Error:".bright_red(), e.bright_red());
        return
    }

    prc::save(&args.prc_out, &prc).unwrap();
}