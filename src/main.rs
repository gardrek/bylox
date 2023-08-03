fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = {
        use clap::Parser;
        bylox::arg::ArgStruct::parse()
    };

    match args.script {
        Some(file) => bylox::run_file(file)?,
        None => {
            use std::io::BufRead;
            use std::io::Write;

            let mut stdout = std::io::BufWriter::new(std::io::stdout());
            let mut stdin = std::io::BufReader::new(std::io::stdin());

            let mut vm = bylox::vm::Vm::new(Box::default());

            loop {
                write!(stdout, "> ")?;
                stdout.flush()?;
                let mut input = String::new();
                stdin.read_line(&mut input)?;

                let chunk = bylox::compile(&input)?;

                vm.interpret(Box::new(chunk))?;
            }
        }
    }

    Ok(())
}
