extern crate ippparse;
extern crate log;

use operation::PrintJob;
use std::io::Read;
use operation::GetPrinterAttributes;
use operation::CreateJob;
use ippparse::IppAttribute;
use operation::SendDocument;

pub mod operation;
pub mod request;

pub struct IppOperationBuilder;

impl IppOperationBuilder {
    pub fn print_job(reader: Box<Read>) -> PrintJobBuilder {
        PrintJobBuilder::new(reader)
    }

    pub fn get_printer_attributes() -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder::new()
    }

    pub fn create_job() -> CreateJobBuilder {
        CreateJobBuilder::new()
    }

    pub fn send_document(job_id: i32, reader: Box<Read>) -> SendDocumentBuilder {
        SendDocumentBuilder::new(job_id, reader)
    }
}

pub struct PrintJobBuilder {
    reader: Box<Read>,
    user_name: Option<String>,
    job_title: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl PrintJobBuilder {
    fn new(reader: Box<Read>) -> PrintJobBuilder {
        PrintJobBuilder {
            reader,
            user_name: None,
            job_title: None,
            attributes: Vec::new()
        }
    }
    pub fn user_name(mut self, user_name: &str) -> Self {
        self.user_name = Some(user_name.to_owned());
        self
    }

    pub fn job_title(mut self, job_title: &str) -> Self {
        self.job_title = Some(job_title.to_owned());
        self
    }

    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn build(self) -> PrintJob {
        let mut op = PrintJob::new(
            self.reader,
            &self.user_name.unwrap_or_else(|| String::new()),
            self.job_title.as_ref(),
        );
        for attr in self.attributes {
            op.add_attribute(attr);
        }
        op
    }
}

pub struct GetPrinterAttributesBuilder {
    attributes: Vec<String>,
}

impl GetPrinterAttributesBuilder {
    fn new() -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder {
            attributes: Vec::new(),
        }
    }

    pub fn attribute(mut self, attribute: &str) -> Self {
        self.attributes.push(attribute.to_owned());
        self
    }

    pub fn attributes<T>(mut self, attributes: &[T]) -> Self
    where
        T: AsRef<str>,
    {
        self.attributes.extend(attributes.iter().map(|s| s.as_ref().to_string()));
        self
    }

    pub fn build(self) -> GetPrinterAttributes {
        GetPrinterAttributes::with_attributes(&self.attributes)
    }
}

pub struct CreateJobBuilder {
    job_name: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl CreateJobBuilder {
    fn new() -> CreateJobBuilder {
        CreateJobBuilder {
            job_name: None,
            attributes: Vec::new(),
        }
    }

    pub fn job_name(mut self, job_name: &str) -> Self {
        self.job_name = Some(job_name.to_owned());
        self
    }

    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn build(self) -> CreateJob {
        let mut op = CreateJob::new(self.job_name.as_ref());
        for attr in self.attributes {
            op.add_attribute(attr);
        }
        op
    }
}

pub struct SendDocumentBuilder {
    job_id: i32,
    reader: Box<Read>,
    user_name: Option<String>,
    is_last: bool,
}

impl SendDocumentBuilder {
    fn new(job_id: i32, reader: Box<Read>) -> SendDocumentBuilder {
        SendDocumentBuilder {
            job_id,
            reader,
            user_name: None,
            is_last: true,
        }
    }

    pub fn user_name(mut self, user_name: &str) -> Self {
        self.user_name = Some(user_name.to_owned());
        self
    }

    pub fn last(mut self, last: bool) -> Self {
        self.is_last = last;
        self
    }

    pub fn build(self) -> SendDocument {
        SendDocument::new(
            self.job_id,
            self.reader,
            &self.user_name.unwrap_or_else(|| String::new()),
            self.is_last
        )
    }
}
