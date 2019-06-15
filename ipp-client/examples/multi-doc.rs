use std::{env, fs::File, io::BufReader, process::exit};

use ipp_client::IppClientBuilder;
use ipp_proto::{
    attribute::{JOB_ID, OPERATIONS_SUPPORTED},
    ipp::{DelimiterTag, Operation},
    IppOperationBuilder, IppValue,
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

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let client = IppClientBuilder::new(&args[1]).build();

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

    for (i, item) in args.iter().enumerate().skip(2) {
        let last = i >= (args.len() - 1);
        println!("Sending {}, last: {}", item, last);
        let f = File::open(&item).unwrap();

        let send_op = IppOperationBuilder::send_document(job_id, Box::new(BufReader::new(f)))
            .user_name(&env::var("USER").unwrap_or_else(|_| String::new()))
            .last(last)
            .build();

        let send_attrs = runtime.block_on(client.send(send_op)).unwrap();
        for v in send_attrs.job_attributes().unwrap().values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
}
