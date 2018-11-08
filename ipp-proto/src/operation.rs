//!
//! High-level IPP operation abstractions
//!
use std::io::Read;

use ippparse::{attribute::*, ipp::*, IppValue};
use request::IppRequestResponse;

/// Trait which represents a single IPP operation
pub trait IppOperation {
    /// Convert this operation to IPP request which is ready for sending
    fn into_ipp_request(self, uri: &str) -> IppRequestResponse;
}

/// IPP operation Print-Job
pub struct PrintJob {
    reader: Box<Read>,
    user_name: String,
    job_name: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl PrintJob {
    /// Create Print-Job operation
    ///
    /// * `reader` - [std::io::Read](https://doc.rust-lang.org/stable/std/io/trait.Read.html) reference which points to the data to be printed<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `job_name` - optional job name (job-name)<br/>
    pub fn new<T>(reader: Box<Read>, user_name: T, job_name: Option<T>) -> PrintJob
    where
        T: AsRef<str>,
    {
        PrintJob {
            reader,
            user_name: user_name.as_ref().to_string(),
            job_name: job_name.map(|v| v.as_ref().to_string()),
            attributes: Vec::new(),
        }
    }

    /// Set extra job attribute for this operation, for example `colormodel=grayscale`
    pub fn add_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }
}

impl IppOperation for PrintJob {
    fn into_ipp_request(self, uri: &str) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(Operation::PrintJob, uri);

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                REQUESTING_USER_NAME,
                IppValue::NameWithoutLanguage(self.user_name.clone()),
            ),
        );

        if let Some(ref job_name) = self.job_name {
            retval.set_attribute(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(JOB_NAME, IppValue::NameWithoutLanguage(job_name.clone())),
            )
        }

        for attr in &self.attributes {
            retval.set_attribute(DelimiterTag::JobAttributes, attr.clone());
        }
        retval.set_payload(self.reader);
        retval
    }
}

/// IPP operation Get-Printer-Attributes
#[derive(Default)]
pub struct GetPrinterAttributes {
    attributes: Vec<String>,
}

impl GetPrinterAttributes {
    /// Create Get-Printer-Attributes operation
    ///
    pub fn new() -> GetPrinterAttributes {
        GetPrinterAttributes::default()
    }

    /// Set attributes to request from the printer
    pub fn with_attributes<T>(attributes: &[T]) -> GetPrinterAttributes
    where
        T: AsRef<str>,
    {
        GetPrinterAttributes {
            attributes: attributes.iter().map(|a| a.as_ref().to_string()).collect(),
        }
    }
}

impl IppOperation for GetPrinterAttributes {
    fn into_ipp_request(self, uri: &str) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(Operation::GetPrinterAttributes, uri);

        if !self.attributes.is_empty() {
            let vals: Vec<IppValue> = self
                .attributes
                .iter()
                .map(|a| IppValue::Keyword(a.clone()))
                .collect();
            retval.set_attribute(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(REQUESTED_ATTRIBUTES, IppValue::ListOf(vals)),
            );
        }

        retval
    }
}

/// IPP operation Create-Job
pub struct CreateJob {
    job_name: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl CreateJob {
    /// Create Create-Job operation
    ///
    /// * `job_name` - optional job name (job-name)<br/>
    pub fn new<T>(job_name: Option<T>) -> CreateJob
    where
        T: AsRef<str>,
    {
        CreateJob {
            job_name: job_name.map(|v| v.as_ref().to_string()),
            attributes: Vec::new(),
        }
    }

    /// Set extra job attribute for this operation, for example `colormodel=grayscale`
    pub fn add_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }
}

impl IppOperation for CreateJob {
    fn into_ipp_request(self, uri: &str) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(Operation::CreateJob, uri);

        if let Some(ref job_name) = self.job_name {
            retval.set_attribute(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(JOB_NAME, IppValue::NameWithoutLanguage(job_name.clone())),
            )
        }

        for attr in &self.attributes {
            retval.set_attribute(DelimiterTag::JobAttributes, attr.clone());
        }
        retval
    }
}

/// IPP operation Send-Document
pub struct SendDocument {
    job_id: i32,
    reader: Box<Read>,
    user_name: String,
    last: bool,
}

impl SendDocument {
    /// Create Send-Document operation
    ///
    /// * `job_id` - job ID returned by Create-Job operation<br/>
    /// * `reader` - [std::io::Read](https://doc.rust-lang.org/stable/std/io/trait.Read.html) reference which points to the data to be printed<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `last` - whether this document is a last one<br/>
    pub fn new<T>(job_id: i32, reader: Box<Read>, user_name: T, last: bool) -> SendDocument
    where
        T: AsRef<str>,
    {
        SendDocument {
            job_id,
            reader,
            user_name: user_name.as_ref().to_string(),
            last,
        }
    }
}

impl IppOperation for SendDocument {
    fn into_ipp_request(self, uri: &str) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(Operation::SendDocument, uri);

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(JOB_ID, IppValue::Integer(self.job_id)),
        );

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                REQUESTING_USER_NAME,
                IppValue::NameWithoutLanguage(self.user_name.clone()),
            ),
        );

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(LAST_DOCUMENT, IppValue::Boolean(self.last)),
        );

        retval.set_payload(self.reader);

        retval
    }
}
