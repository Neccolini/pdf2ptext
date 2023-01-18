mod pdf_reader;

use clap::Parser;
use std::path::Path;

#[derive(Parser)]
#[command(name = "pdf2ptext")]
#[command(author = "Neccolini")]
#[command(version = "0.0")]
#[command(about = "Extract plain text from pdf file", long_about = None)]
struct Cli {
    #[arg(value_name="FILE")]
    pdf: String,
}
fn main() {
    let cli = Cli::parse();

    let path = Path::new(&cli.pdf);
    let s = pdf_reader::read_pdf(path);
    println!("{}", s);
}
