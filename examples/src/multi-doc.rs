use std::{env, error::Error, fs, process::exit};

use ipp::prelude::*;

fn supports_multi_doc(v: &IppValue) -> bool {
    v.as_enum()
        .map(|v| *v == Operation::CreateJob as i32 || *v == Operation::SendDocument as i32)
        .unwrap_or(false)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [filename...]", args[0]);
        exit(1);
    }

    let uri: Uri = args[1].parse()?;
    let client = IppClient::new(uri.clone());

    // check if printer supports create/send operations
    let get_op = IppOperationBuilder::get_printer_attributes(uri.clone())
        .attribute(IppAttribute::OPERATIONS_SUPPORTED)
        .build()?;

    let response = client.send(get_op)?;

    let ops_attr = response
        .attributes()
        .groups_of(DelimiterTag::PrinterAttributes)
        .next()
        .and_then(|g| g.attributes().get(IppAttribute::OPERATIONS_SUPPORTED))
        .ok_or(IppError::MissingAttribute)?;

    if !ops_attr.value().into_iter().any(supports_multi_doc) {
        println!("ERROR: target printer does not support create/send operations");
        exit(2);
    }

    let create_op = IppOperationBuilder::create_job(uri.clone())
        .job_name("multi-doc")
        .build()?;
    let response = client.send(create_op)?;
    let job_id = *response
        .attributes()
        .groups_of(DelimiterTag::JobAttributes)
        .next()
        .and_then(|g| g.attributes().get(IppAttribute::JOB_ID))
        .and_then(|attr| attr.value().as_integer())
        .ok_or(IppError::MissingAttribute)?;

    println!("job id: {job_id}");

    for (i, item) in args.iter().enumerate().skip(2) {
        let client = IppClient::new(uri.clone());

        let last = i >= (args.len() - 1);
        println!("Sending {item}, last: {last}");

        let payload = IppPayload::new(fs::File::open(item)?);

        let send_op = IppOperationBuilder::send_document(uri.clone(), job_id, payload)
            .user_name(env::var("USER").unwrap_or_else(|_| String::new()))
            .last(last)
            .build()?;

        let response = client.send(send_op)?;
        println!("IPP status code: {}", response.header().status_code());

        let attrs = response
            .attributes()
            .groups_of(DelimiterTag::JobAttributes)
            .flat_map(|g| g.attributes().values());

        for attr in attrs {
            println!("{}: {}", attr.name(), attr.value());
        }
    }

    Ok(())
}
