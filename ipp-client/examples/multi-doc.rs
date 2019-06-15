use std::{env, process::exit};

use futures::Future;

use ipp_client::{IppClientBuilder, IppError};
use ipp_proto::{
    attribute::{JOB_ID, OPERATIONS_SUPPORTED},
    ipp::{DelimiterTag, Operation},
    IppOperationBuilder, IppReadStream, IppValue,
};

fn supports_multi_doc(v: &IppValue) -> bool {
    if let IppValue::Enum(ref v) = v {
        *v == Operation::CreateJob as i32 || *v == Operation::SendDocument as i32
    } else {
        false
    }
}

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} uri filename [filename...]", args[0]);
        exit(1);
    }

    let uri = args[1].clone();

    let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let client = IppClientBuilder::new(&uri).build();

    // check if printer supports create/send operations
    let get_op = IppOperationBuilder::get_printer_attributes()
        .attribute(OPERATIONS_SUPPORTED)
        .build();
    let printer_attrs = runtime.block_on(client.send(get_op)).unwrap();
    let ops_attr = printer_attrs
        .get(DelimiterTag::PrinterAttributes, OPERATIONS_SUPPORTED)
        .unwrap();

    if !ops_attr.value().into_iter().any(supports_multi_doc) {
        println!("ERROR: target printer does not support create/send operations");
        exit(2);
    }

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let create_op = IppOperationBuilder::create_job().job_name("multi-doc").build();
    let attrs = runtime.block_on(client.send(create_op)).unwrap();
    let job_id = match *attrs.get(DelimiterTag::JobAttributes, JOB_ID).unwrap().value() {
        IppValue::Integer(id) => id,
        _ => panic!("invalid value"),
    };
    println!("job id: {}", job_id);

    let pool = tokio_threadpool::ThreadPool::new();

    for (i, item) in args.iter().enumerate().skip(2) {
        let client = IppClientBuilder::new(&uri).build();

        let last = i >= (args.len() - 1);
        println!("Sending {}, last: {}", item, last);

        let fut = pool
            .spawn_handle(tokio::fs::File::open(item.to_owned()))
            .map_err(IppError::from)
            .and_then(move |f| {
                let send_op = IppOperationBuilder::send_document(job_id, IppReadStream::new(Box::new(f)))
                    .user_name(&env::var("USER").unwrap_or_else(|_| String::new()))
                    .last(last)
                    .build();

                client.send(send_op).and_then(|send_attrs| {
                    for v in send_attrs.job_attributes().unwrap().values() {
                        println!("{}: {}", v.name(), v.value());
                    }
                    Ok(())
                })
            });

        runtime.block_on(fut).unwrap();
    }
}
