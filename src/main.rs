use clap::Parser;

#[derive(Parser, Debug)]
#[command(author,version)]
struct Args {
    #[arg(short, long)]
    ip: Option<String>,
    #[arg(short, long)]
    port: String,
    #[arg(short, long)]
    config: Option<String>,
}
fn main() {
    let args = Args::parse();

    println!("{:?}", args);
}
