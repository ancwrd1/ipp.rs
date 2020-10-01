use std::{env, error::Error, fs, process::exit};

use ipp::prelude::*;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [key=value ...]", args[0]);
        exit(1);
    }

    let reader = futures::io::AllowStdIo::new(fs::File::open(&args[2])?);

    let mut builder = IppOperationBuilder::print_job(reader)
        .user_name(&env::var("USER").unwrap_or_else(|_| String::new()))
        .job_title(&args[1]);

    for arg in &args[3..] {
        let mut kv = arg.split('=');
        let (k, v) = (kv.next().unwrap(), kv.next().unwrap());

        builder = builder.attribute(IppAttribute::new(k, v.parse()?));
    }

    let operation = builder.build();

    let client = IppClientBuilder::new(args[1].parse()?).build();

    let attrs = futures::executor::block_on(client.send(operation))?;

    for v in attrs.groups_of(DelimiterTag::JobAttributes)[0].attributes().values() {
        println!("{}: {}", v.name(), v.value());
    }

    Ok(())
}
