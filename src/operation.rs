//!
//! High-level IPP operation abstractions
//!

use std::io::Read;
use attribute::{IppAttribute, IppAttributeList};
use request::IppRequest;
use value::IppValue;
use consts::tag::*;
use consts::operation::*;
use consts::attribute::*;
use client::IppClient;
use ::{Result, IppError};

/// Trait which represents a single IPP operation
pub trait IppOperation {
    /// Convert this operation to IPP request which is ready for sending
    fn to_ipp_request(&mut self) -> IppRequest;

    /// Execute this operation (send it to IPP server)
    fn execute(&mut self) -> Result<IppAttributeList> {
        let client = IppClient::new();
        client.send(&mut self.to_ipp_request())
    }
}

/// IPP operation Print-Job
pub struct PrintJob<'a> {
    uri: String,
    reader: &'a mut Read,
    user_name: String,
    job_name: Option<String>,
    attributes: Vec<IppAttribute>
}

impl<'a> PrintJob<'a> {
    /// Create Print-Job operation
    /// Parameters:
    /// * uri - printer URI
    /// * reader - std::io::Read reference which points to the data to be printed
    /// * user_name - name of the user (requesting-user-name)
    /// * job_name - optional job name (job-name)
    pub fn new(uri: &str, reader: &'a mut Read,
               user_name: &str, job_name: Option<&str>) -> PrintJob<'a> {
        PrintJob {
            uri: uri.to_string(),
            reader: reader,
            user_name: user_name.to_string(),
            job_name: match job_name { Some(name) => Some(name.to_string()), None => None },
            attributes: Vec::new()
        }
    }

    /// Set extra job attribute for this operation, for example colormodel=grayscale
    pub fn set_job_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }

    pub fn execute_and_get_job_id(&mut self) -> Result<i32> {
        let attrs = try!(self.execute());
        let val = attrs.get(JOB_ATTRIBUTES_TAG, JOB_ID);
        match val {
            Some(attr) => match attr.value() {
                &IppValue::Integer(id) => Ok(id),
                _ => {
                    error!("Invalid job-id attribute in the response");
                    Err(IppError::AttributeError(JOB_ID.to_string()))
                }
            },
            None => {
                error!("No job-id attribute in the response");
                Err(IppError::AttributeError(JOB_ID.to_string()))
            }
        }
    }
}

impl<'a> IppOperation for PrintJob<'a> {
    fn to_ipp_request(&mut self) -> IppRequest {
        let mut retval = IppRequest::new(PRINT_JOB, &self.uri);

        retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(REQUESTING_USER_NAME,
                IppValue::NameWithoutLanguage(self.user_name.clone())));

        if self.job_name.is_some() {
            retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
                IppAttribute::new(JOB_NAME,
                    IppValue::NameWithoutLanguage(self.job_name.clone().unwrap())));
        }

        for attr in &self.attributes {
            retval.set_attribute(JOB_ATTRIBUTES_TAG, attr.clone());
        }
        retval.set_payload(self.reader);
        retval
    }
}

/// IPP operation Get-Printer-Attributes
pub struct GetPrinterAttributes {
    uri: String,
    attributes: Vec<String>
}

impl GetPrinterAttributes {
    /// Create Get-Printer-Attributes operation
    /// Parameters:
    /// * uri - printer URI
    pub fn new(uri: &str) -> GetPrinterAttributes {
        GetPrinterAttributes { uri: uri.to_string(), attributes: Vec::new() }
    }

    /// Set attributes to request from the printer
    pub fn set_attributes(&mut self, attributes: &[String]) {
        for a in attributes {
            self.attributes.push(a.to_string())
        }
    }
}

impl IppOperation for GetPrinterAttributes {
    fn to_ipp_request(&mut self) -> IppRequest {
        let mut retval = IppRequest::new(GET_PRINTER_ATTRIBUTES, &self.uri);

        if self.attributes.len() > 0 {
            let vals: Vec<IppValue> = self.attributes.iter().map(|a| IppValue::Keyword(a.clone())).collect();
            retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
                IppAttribute::new(REQUESTED_ATTRIBUTES, IppValue::ListOf(vals)));
        }

        retval
    }
}

/// IPP operation Create-Job
pub struct CreateJob {
    uri: String,
    job_name: Option<String>,
    attributes: Vec<IppAttribute>
}

impl CreateJob {
    /// Create Create-Job operation
    /// Parameters:
    /// * uri - printer URI
    /// * job_name - optional job name (job-name)
    pub fn new(uri: &str, job_name: Option<&str>) -> CreateJob {
        CreateJob {
            uri: uri.to_string(),
            job_name: match job_name { Some(name) => Some(name.to_string()), None => None },
            attributes: Vec::new()
        }
    }

    /// Set extra job attribute for this operation, for example colormodel=grayscale
    pub fn set_job_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }

    /// Convenience method to execute the request and return the job-id
    pub fn execute_and_get_job_id(&mut self) -> Result<i32> {
        let attrs = try!(self.execute());
        let val = attrs.get(JOB_ATTRIBUTES_TAG, JOB_ID);
        match val {
            Some(attr) => match attr.value() {
                &IppValue::Integer(id) => Ok(id),
                _ => {
                    error!("Invalid job-id attribute in the response");
                    Err(IppError::AttributeError(JOB_ID.to_string()))
                }
            },
            None => {
                error!("No job-id attribute in the response");
                Err(IppError::AttributeError(JOB_ID.to_string()))
            }
        }
    }
}

impl IppOperation for CreateJob {
    fn to_ipp_request(&mut self) -> IppRequest {
        let mut retval = IppRequest::new(CREATE_JOB, &self.uri);

        if self.job_name.is_some() {
            retval.set_attribute(OPERATION_ATTRIBUTES_TAG,
                IppAttribute::new(JOB_NAME,
                    IppValue::NameWithoutLanguage(self.job_name.clone().unwrap())));
        }

        for attr in &self.attributes {
            retval.set_attribute(JOB_ATTRIBUTES_TAG, attr.clone());
        }
        retval
    }
}

/// IPP operation Print-Job
pub struct SendDocument<'a> {
    uri: String,
    job_id: i32,
    reader: &'a mut Read,
    user_name: String,
    last: bool
}

impl<'a> SendDocument<'a> {
    /// Create Send-Document operation
    /// Parameters:
    /// * uri - printer URI
    /// * job_id - job ID returned by Create-Job operation
    /// * reader - std::io::Read reference which points to the data to be printed
    /// * user_name - name of the user (requesting-user-name)
    /// * last - whether this document is a last one
    pub fn new(uri: &str, job_id: i32, reader: &'a mut Read,
               user_name: &str, last: bool) -> SendDocument<'a> {
        SendDocument {
            uri: uri.to_string(),
            job_id: job_id,
            reader: reader,
            user_name: user_name.to_string(),
            last: last
        }
    }
}

impl<'a> IppOperation for SendDocument<'a> {
    fn to_ipp_request(&mut self) -> IppRequest {
        let mut retval = IppRequest::new(SEND_DOCUMENT, &self.uri);

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
