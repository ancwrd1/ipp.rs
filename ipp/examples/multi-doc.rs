use std::{env, error::Error, process::exit};

use ipp::{
    client::{IppClientBuilder, IppError},
    proto::{
        attribute::{JOB_ID, OPERATIONS_SUPPORTED},
        ipp::{DelimiterTag, Operation},
        IppOperationBuilder, IppValue,
    },
};

fn supports_multi_doc(v: &IppValue) -> bool {
    v.as_enum()
        .map(|v| *v == Operation::CreateJob as i32 || *v == Operation::SendDocument as i32)
        .unwrap_or(false)
}

pub fn main() -> Result<(), Box<dyn Error>> {
    async_std::task::block_on(async {
        env_logger::init();

        let args: Vec<_> = env::args().collect();

        if args.len() < 3 {
            println!("Usage: {} uri filename [filename...]", args[0]);
            exit(1);
        }

        let uri = args[1].clone();

        let client = IppClientBuilder::new(&uri).build();

        // check if printer supports create/send operations
        let get_op = IppOperationBuilder::get_printer_attributes()
            .attribute(OPERATIONS_SUPPORTED)
            .build();
        let printer_attrs = client.send(get_op).await?;

        let ops_attr = printer_attrs
            .groups_of(DelimiterTag::PrinterAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(OPERATIONS_SUPPORTED))
            .ok_or(IppError::MissingAttribute)?;

        if !ops_attr.value().into_iter().any(supports_multi_doc) {
            println!("ERROR: target printer does not support create/send operations");
            exit(2);
        }

        let create_op = IppOperationBuilder::create_job().job_name("multi-doc").build();
        let attrs = client.send(create_op).await?;
        let job_id = *attrs
            .groups_of(DelimiterTag::JobAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(JOB_ID))
            .and_then(|attr| attr.value().as_integer())
            .ok_or(IppError::MissingAttribute)?;

        println!("job id: {}", job_id);

        for (i, item) in args.iter().enumerate().skip(2) {
            let client = IppClientBuilder::new(&uri).build();

            let last = i >= (args.len() - 1);
            println!("Sending {}, last: {}", item, last);

            let reader = async_std::fs::File::open(item.to_owned()).await?;

            let send_op = IppOperationBuilder::send_document(job_id, reader)
                .user_name(&env::var("USER").unwrap_or_else(|_| String::new()))
                .last(last)
                .build();

            let attrs = client.send(send_op).await?;
            for v in attrs.groups_of(DelimiterTag::JobAttributes)[0].attributes().values() {
                println!("{}: {}", v.name(), v.value());
            }
        }

        Ok(())
    })
}
