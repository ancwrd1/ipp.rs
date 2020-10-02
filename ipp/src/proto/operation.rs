//!
//! High-level IPP operation abstractions
//!
use http::Uri;

use crate::proto::{
    model::DelimiterTag, request::IppRequestResponse, IppAttribute, IppPayload, IppValue, IppVersion, Operation,
};

pub mod cups;

/// Trait which represents a single IPP operation
pub trait IppOperation {
    /// Convert this operation to IPP request which is ready for sending
    fn into_ipp_request(self, uri: Uri) -> IppRequestResponse;

    /// Return IPP version for this operation. Default is 1.1
    fn version(&self) -> IppVersion {
        IppVersion::v1_1()
    }
}

/// IPP operation Print-Job
pub struct PrintJob {
    payload: IppPayload,
    user_name: Option<String>,
    job_name: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl PrintJob {
    /// Create Print-Job operation
    ///
    /// * `payload` - job payload<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `job_name` - job name (job-name)<br/>
    pub fn new<S, U, N>(payload: S, user_name: Option<U>, job_name: Option<N>) -> PrintJob
    where
        S: Into<IppPayload>,
        U: AsRef<str>,
        N: AsRef<str>,
    {
        PrintJob {
            payload: payload.into(),
            user_name: user_name.map(|v| v.as_ref().to_string()),
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
    fn into_ipp_request(self, uri: Uri) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(self.version(), Operation::PrintJob, Some(uri));

        if let Some(user_name) = self.user_name {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(
                    IppAttribute::REQUESTING_USER_NAME,
                    IppValue::NameWithoutLanguage(user_name),
                ),
            );
        }

        if let Some(job_name) = self.job_name {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(IppAttribute::JOB_NAME, IppValue::NameWithoutLanguage(job_name)),
            )
        }

        for attr in self.attributes {
            retval.attributes_mut().add(DelimiterTag::JobAttributes, attr);
        }
        *retval.payload_mut() = self.payload;

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
    fn into_ipp_request(self, uri: Uri) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(self.version(), Operation::GetPrinterAttributes, Some(uri));

        if !self.attributes.is_empty() {
            let vals: Vec<IppValue> = self.attributes.into_iter().map(IppValue::Keyword).collect();
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(IppAttribute::REQUESTED_ATTRIBUTES, IppValue::Array(vals)),
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
    fn into_ipp_request(self, uri: Uri) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(self.version(), Operation::CreateJob, Some(uri));

        if let Some(job_name) = self.job_name {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(IppAttribute::JOB_NAME, IppValue::NameWithoutLanguage(job_name)),
            )
        }

        for attr in self.attributes {
            retval.attributes_mut().add(DelimiterTag::JobAttributes, attr);
        }
        retval
    }
}

/// IPP operation Send-Document
pub struct SendDocument {
    job_id: i32,
    payload: IppPayload,
    user_name: Option<String>,
    last: bool,
}

impl SendDocument {
    /// Create Send-Document operation
    ///
    /// * `job_id` - job ID returned by Create-Job operation<br/>
    /// * `payload` - `IppPayload`<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `last` - whether this document is a last one<br/>
    pub fn new<S, U>(job_id: i32, payload: S, user_name: Option<U>, last: bool) -> SendDocument
    where
        S: Into<IppPayload>,
        U: AsRef<str>,
    {
        SendDocument {
            job_id,
            payload: payload.into(),
            user_name: user_name.map(|v| v.as_ref().to_string()),
            last,
        }
    }
}

impl IppOperation for SendDocument {
    fn into_ipp_request(self, uri: Uri) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new(self.version(), Operation::SendDocument, Some(uri));

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::JOB_ID, IppValue::Integer(self.job_id)),
        );

        if let Some(user_name) = self.user_name {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(
                    IppAttribute::REQUESTING_USER_NAME,
                    IppValue::NameWithoutLanguage(user_name),
                ),
            );
        }

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::LAST_DOCUMENT, IppValue::Boolean(self.last)),
        );

        *retval.payload_mut() = self.payload;

        retval
    }
}
