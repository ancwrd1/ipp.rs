//!
//! High-level IPP operation abstractions
//!

use std::io::Read;
use attribute::IppAttribute;
use request::IppRequest;
use value::IppValue;
use consts::tag::*;
use consts::operation::*;
use consts::attribute::*;

/// Trait which represents a single IPP operation
pub trait IppOperation {
    /// Convert this operation to IPP request which is ready for sending
    fn to_ipp_request(&mut self, uri: &str) -> IppRequest;
}

/// IPP operation Print-Job
pub struct PrintJob<'a> {
    reader: &'a mut Read,
    user_name: String,
    job_name: Option<String>,
    attributes: Vec<IppAttribute>
}

impl<'a> PrintJob<'a> {
    /// Create Print-Job operation
    ///
    /// * `reader` - [std::io::Read](https://doc.rust-lang.org/stable/std/io/trait.Read.html) reference which points to the data to be printed<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `job_name` - optional job name (job-name)<br/>
    pub fn new(reader: &'a mut Read,
               user_name: &str, job_name: Option<&str>) -> PrintJob<'a> {
        PrintJob {
            reader: reader,
            user_name: user_name.to_string(),
            job_name: if let Some(name) = job_name { Some(name.to_string()) } else { None },
            attributes: Vec::new()
        }
    }

    /// Set extra job attribute for this operation, for example `colormodel=grayscale`
    pub fn add_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }
}

impl<'a> IppOperation for PrintJob<'a> {
    fn to_ipp_request(&mut self, uri: &str) -> IppRequest {
        let mut retval = IppRequest::new(PRINT_JOB, uri);

        retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(REQUESTING_USER_NAME,
                IppValue::NameWithoutLanguage(self.user_name.clone())));

        if let Some(ref job_name) = self.job_name {
            retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
                                IppAttribute::new(JOB_NAME,
                                IppValue::NameWithoutLanguage(job_name.clone())))
        }

        for attr in &self.attributes {
            retval.set_attribute(JOB_ATTRIBUTES_TAG, attr.clone());
        }
        retval.set_payload(self.reader);
        retval
    }
}

/// IPP operation Get-Printer-Attributes
#[derive(Default)]
pub struct GetPrinterAttributes {
    attributes: Vec<String>
}

impl GetPrinterAttributes {
    /// Create Get-Printer-Attributes operation
    ///
    pub fn new() -> GetPrinterAttributes {
        GetPrinterAttributes::default()
    }

    /// Set attributes to request from the printer
    pub fn with_attributes(attributes: &[String]) -> GetPrinterAttributes {
        GetPrinterAttributes { attributes: attributes.to_vec() }
    }
}

impl IppOperation for GetPrinterAttributes {
    fn to_ipp_request(&mut self, uri: &str) -> IppRequest {
        let mut retval = IppRequest::new(GET_PRINTER_ATTRIBUTES, uri);

        if !self.attributes.is_empty() {
            let vals: Vec<IppValue> = self.attributes.iter().map(|a| IppValue::Keyword(a.clone())).collect();
            retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
                IppAttribute::new(REQUESTED_ATTRIBUTES, IppValue::ListOf(vals)));
        }

        retval
    }
}

/// IPP operation Create-Job
pub struct CreateJob {
    job_name: Option<String>,
    attributes: Vec<IppAttribute>
}

impl CreateJob {
    /// Create Create-Job operation
    ///
    /// * `job_name` - optional job name (job-name)<br/>
    pub fn new(job_name: Option<&str>) -> CreateJob {
        CreateJob {
            job_name: if let Some(name) = job_name { Some(name.to_string()) } else { None },
            attributes: Vec::new()
        }
    }

    /// Set extra job attribute for this operation, for example `colormodel=grayscale`
    pub fn add_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }


}

impl IppOperation for CreateJob {
    fn to_ipp_request(&mut self, uri: &str) -> IppRequest {
        let mut retval = IppRequest::new(CREATE_JOB, uri);

        if let Some(ref job_name) = self.job_name {
            retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
                                IppAttribute::new(JOB_NAME,
                                IppValue::NameWithoutLanguage(job_name.clone())))
        }

        for attr in &self.attributes {
            retval.set_attribute(JOB_ATTRIBUTES_TAG, attr.clone());
        }
        retval
    }
}

/// IPP operation Print-Job
pub struct SendDocument<'a> {
    job_id: i32,
    reader: &'a mut Read,
    user_name: String,
    last: bool
}

impl<'a> SendDocument<'a> {
    /// Create Send-Document operation
    ///
    /// * `job_id` - job ID returned by Create-Job operation<br/>
    /// * `reader` - [std::io::Read](https://doc.rust-lang.org/stable/std/io/trait.Read.html) reference which points to the data to be printed<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `last` - whether this document is a last one<br/>
    pub fn new(job_id: i32, reader: &'a mut Read,
               user_name: &str, last: bool) -> SendDocument<'a> {
        SendDocument {
            job_id: job_id,
            reader: reader,
            user_name: user_name.to_string(),
            last: last
        }
    }
}

impl<'a> IppOperation for SendDocument<'a> {
    fn to_ipp_request(&mut self, uri: &str) -> IppRequest {
        let mut retval = IppRequest::new(SEND_DOCUMENT, uri);

        retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(JOB_ID, IppValue::Integer(self.job_id)));

        retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(REQUESTING_USER_NAME,
                IppValue::NameWithoutLanguage(self.user_name.clone())));

        retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(LAST_DOCUMENT,
                IppValue::Boolean(self.last)));

        retval.set_payload(self.reader);

        retval
    }
}
