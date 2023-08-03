use clap::Parser;

#[derive(Parser)]
#[command(name = "bylox")]
//#[command(author = "Nonymous A. <admin@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Lox interpreter", long_about = None)]
pub struct ArgStruct {
    /// The script to run
    pub script: Option<std::path::PathBuf>,
}
