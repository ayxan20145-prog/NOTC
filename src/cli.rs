use clap::Parser as ClapParser;

#[derive(ClapParser, Debug)]
#[command(name = "ncc", version, about = "!C compiler")]
pub struct Cli {
    pub input: String,

    #[arg(short = 'S', long)]
    pub assemble: bool,

    #[arg(short = 'c', long)]
    pub compile: bool,

    #[arg(short = 'o', long, default_value = "a.out")]
    pub output: String,
}
