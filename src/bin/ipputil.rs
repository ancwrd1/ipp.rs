extern crate ipp;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::fs::File;
use std::net::{TcpStream, Shutdown};
use std::thread;
use std::io::{Read, Write, copy};
use std::time::Duration;

use ipp::{IppClient, IppAttribute, IppValue, PrintJob, GetPrinterAttributes, IppError};
use ipp::consts::tag::{JOB_ATTRIBUTES_TAG, PRINTER_ATTRIBUTES_TAG};

fn do_socket_print(addr: &str, file: &mut Read) -> Result<(), IppError> {
    let mut stream = TcpStream::connect(addr)?;
    let mut reader = stream.try_clone()?;
    
    let handle = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(size) if size > 0 =>  println!("{}", String::from_utf8_lossy(&buf[0..size])),
                _ =>  break
            }
        }
        let _ = reader.shutdown(Shutdown::Read);
    });

    copy(file, &mut stream)?;
    let _ = stream.shutdown(Shutdown::Write);
    let _ = handle.join();
    Ok(())
}

const PJL_PREFIX: &'static str = "\x1b%-12345X@PJL INFO ";

fn do_socket_status(addr: &str, attrs: &[String]) -> Result<(), IppError> {
    let mut stream = TcpStream::connect(addr)?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(10000)));
    let mut buf = [0u8; 4096];

    let def_attrs = [String::from("ID"), String::from("STATUS")];

    for pjl in if attrs.len() > 0 { attrs } else { &def_attrs[..] } {
        stream.write((PJL_PREFIX.to_string() + pjl + "\n\x1b%-12345X").as_bytes())?;
        loop {
            match stream.read(&mut buf) {
                Ok(size) if size > 0 => {
                    let s = String::from_utf8_lossy(&buf[0..size]).to_string();
                    println!("{}", s.trim());
                    if s.ends_with('\x0c') { break }
                }
                _ =>  break
            }
        }
    }
    Ok(())
}

fn do_print(args: &[String]) -> Result<(), IppError> {
    let mut f = File::open(&args[3])?;

    if args[2].starts_with("socket://") {
        let mut addr = args[2][9..].to_string();
        if !addr.contains(':') { addr += ":9100" }
        return do_socket_print(&addr, &mut f);
    }

    let client = IppClient::new(&args[2]);

    let mut operation = PrintJob::new(
        &mut f,
        &env::var("USER").unwrap_or_else(|_| String::new()),
        Some(&args[1])
    );

    for arg in &args[4..] {
        let mut kv = arg.split('=');
        let (k, v) = (kv.next().unwrap_or_else(|| ""), kv.next().unwrap_or_else(|| ""));

        if k.is_empty() || v.is_empty() { continue }

        let value = if let Ok(iv) = v.parse::<i32>() {
            IppValue::Integer(iv)
        } else if v == "true" || v == "false" {
            IppValue::Boolean(v == "true")
        } else {
            IppValue::Keyword(v.to_string())
        };

        operation.add_attribute(IppAttribute::new(k, value));
    }

    let attrs = client.send(&mut operation)?;

    if let Some(group) = attrs.get_group(JOB_ATTRIBUTES_TAG) {
        for v in group.values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

fn do_status(args: &[String]) -> Result<(), IppError> {
    if args[2].starts_with("socket://") {
        let mut addr = args[2][9..].to_string();
        if !addr.contains(':') { addr += ":9100" }
        return do_socket_status(&addr, &args[3..]);
    }

    let client = IppClient::new(&args[2]);
    let mut operation = GetPrinterAttributes::with_attributes(&args[3..]);

    let attrs = client.send(&mut operation)?;

    if let Some(group) = attrs.get_group(PRINTER_ATTRIBUTES_TAG) {
        for v in group.values() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}

fn usage(prog: &str) {
    println!("Usage: {} status uri [attr...]", prog);
    println!("       {} print uri filename [attr=value]", prog);
    println!("\nSupported uri schemes: http, socket");
}

pub fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        usage(&args[0]);
        exit(1);
    }

    match &args[1][..] {
        "status" => {
            if let Err(err) = do_status(&args) {
                println!("{:?}", err);
                exit(2);
            }
            
        }
        "print" => {
            if args.len() < 4 {
                usage(&args[0]);
                exit(1);
            }
            if let Err(err) = do_print(&args) {
                println!("{:?}", err);
                exit(2);
            }
        }
        _ => {
            println!("ERROR: invalid operation, expected one of 'status' or 'print");
            exit(1);
        }
    }
}
