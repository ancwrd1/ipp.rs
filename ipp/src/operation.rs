//!
//! High-level IPP operation abstractions
//!
use http::Uri;

use crate::{
    attribute::IppAttribute,
    model::{DelimiterTag, IppVersion, Operation},
    parser::IppParseError,
    payload::IppPayload,
    request::IppRequestResponse,
    value::{IppKeyword, IppMimeMediaType, IppName, IppString, IppTextValue, IppValue},
};

pub mod builder;
pub mod cups;

fn with_user_name(user_name: Option<IppName>, req: &mut IppRequestResponse) {
    if let Some(user_name) = user_name {
        req.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                IppAttribute::REQUESTING_USER_NAME,
                IppValue::NameWithoutLanguage(user_name),
            ),
        );
    }
}

fn with_document_format(document_format: Option<IppMimeMediaType>, req: &mut IppRequestResponse) {
    if let Some(document_format) = document_format {
        req.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::DOCUMENT_FORMAT, IppValue::MimeMediaType(document_format)),
        );
    }
}

/// Trait which represents a single IPP operation
pub trait IppOperation {
    /// Convert this operation to an IPP request which is ready for sending
    fn into_ipp_request(self) -> IppRequestResponse;

    /// Return the IPP version for this operation. Default is 1.1
    fn version(&self) -> IppVersion {
        IppVersion::v1_1()
    }
}

impl<T: IppOperation> From<T> for IppRequestResponse {
    fn from(op: T) -> Self {
        op.into_ipp_request()
    }
}

/// IPP operation Print-Job
pub struct PrintJob {
    printer_uri: IppString,
    payload: IppPayload,
    user_name: Option<IppName>,
    job_name: Option<IppName>,
    document_format: Option<IppMimeMediaType>,
    attributes: Vec<IppAttribute>,
}

impl PrintJob {
    /// Create a Print-Job operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `payload` - job payload<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `document_format` - mime-type of the payload<br/>
    /// * `job_name` - job name (job-name)<br/>
    pub fn new<S, U, N, D>(
        printer_uri: Uri,
        payload: S,
        user_name: Option<U>,
        job_name: Option<N>,
        document_format: Option<D>,
    ) -> Result<PrintJob, IppParseError>
    where
        S: Into<IppPayload>,
        U: AsRef<str>,
        N: AsRef<str>,
        D: AsRef<str>,
    {
        Ok(PrintJob {
            printer_uri: printer_uri.try_into()?,
            payload: payload.into(),
            user_name: user_name.map(|v| v.as_ref().to_string().try_into()).transpose()?,
            job_name: job_name.map(|v| v.as_ref().to_string().try_into()).transpose()?,
            document_format: document_format.map(|v| v.as_ref().to_string().try_into()).transpose()?,
            attributes: Vec::new(),
        })
    }

    /// Set an extra job attribute for this operation, for example `colormodel=grayscale`
    pub fn add_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }
}

impl IppOperation for PrintJob {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new_internal(self.version(), Operation::PrintJob, Some(self.printer_uri));

        with_user_name(self.user_name, &mut retval);
        with_document_format(self.document_format, &mut retval);

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
pub struct GetPrinterAttributes {
    printer_uri: IppString,
    attributes: Vec<IppKeyword>,
}

impl GetPrinterAttributes {
    /// Create a Get-Printer-Attributes operation to return all attributes
    ///
    /// * `printer_uri` - printer URI
    pub fn new(printer_uri: Uri) -> Result<GetPrinterAttributes, IppParseError> {
        Ok(GetPrinterAttributes {
            printer_uri: printer_uri.try_into()?,
            attributes: Vec::new(),
        })
    }

    /// Create a Get-Printer-Attributes operation to get a given list of attributes
    ///
    /// * `printer_uri` - printer URI
    /// * `attributes` - list of attribute names to request from the printer
    pub fn with_attributes<I, T>(printer_uri: Uri, attributes: I) -> Result<GetPrinterAttributes, IppParseError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        Ok(GetPrinterAttributes {
            printer_uri: printer_uri.try_into()?,
            attributes: attributes
                .into_iter()
                .map(|a| a.as_ref().try_into())
                .collect::<Result<Vec<IppKeyword>, IppParseError>>()?,
        })
    }
}

impl IppOperation for GetPrinterAttributes {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::GetPrinterAttributes, Some(self.printer_uri));

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
    printer_uri: IppString,
    job_name: Option<IppName>,
    attributes: Vec<IppAttribute>,
}

impl CreateJob {
    /// Create a Create-Job operation
    ///
    /// * `printer_uri` - printer URI
    /// * `job_name` - optional job name (job-name)<br/>
    pub fn new<T>(printer_uri: Uri, job_name: Option<T>) -> Result<CreateJob, IppParseError>
    where
        T: AsRef<str>,
    {
        Ok(CreateJob {
            printer_uri: printer_uri.try_into()?,
            job_name: job_name.map(|v| v.as_ref().to_string().try_into()).transpose()?,
            attributes: Vec::new(),
        })
    }

    /// Set an extra job attribute for this operation, for example `colormodel=grayscale`
    pub fn add_attribute(&mut self, attribute: IppAttribute) {
        self.attributes.push(attribute);
    }
}

impl IppOperation for CreateJob {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new_internal(self.version(), Operation::CreateJob, Some(self.printer_uri));

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
    printer_uri: IppString,
    job_id: i32,
    payload: IppPayload,
    user_name: Option<IppName>,
    document_format: Option<IppMimeMediaType>,
    last: bool,
}

impl SendDocument {
    /// Create a Send-Document operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - job ID returned by Create-Job operation<br/>
    /// * `payload` - `IppPayload`<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `document_format` - mime-type of the payload<br/>
    /// * `last` - whether this document is a last one<br/>
    pub fn new<S, U, D>(
        printer_uri: Uri,
        job_id: i32,
        payload: S,
        user_name: Option<U>,
        document_format: Option<D>,
        last: bool,
    ) -> Result<SendDocument, IppParseError>
    where
        S: Into<IppPayload>,
        U: AsRef<str>,
        D: AsRef<str>,
    {
        Ok(SendDocument {
            printer_uri: printer_uri.try_into()?,
            job_id,
            payload: payload.into(),
            user_name: user_name.map(|v| v.as_ref().to_string().try_into()).transpose()?,
            document_format: document_format.map(|v| v.as_ref().to_string().try_into()).transpose()?,
            last,
        })
    }
}

impl IppOperation for SendDocument {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::SendDocument, Some(self.printer_uri));

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::JOB_ID, IppValue::Integer(self.job_id)),
        );

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::LAST_DOCUMENT, IppValue::Boolean(self.last)),
        );

        with_user_name(self.user_name, &mut retval);
        with_document_format(self.document_format, &mut retval);

        *retval.payload_mut() = self.payload;

        retval
    }
}

/// IPP operation Purge-Jobs
pub struct PurgeJobs {
    printer_uri: IppString,
    user_name: Option<IppName>,
}

impl PurgeJobs {
    /// Create a Purge-Jobs operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for PurgeJobs {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new_internal(self.version(), Operation::PurgeJobs, Some(self.printer_uri));

        with_user_name(self.user_name, &mut retval);

        retval
    }
}

/// IPP operation Cancel-Job
pub struct CancelJob {
    printer_uri: IppString,
    job_id: i32,
    user_name: Option<IppName>,
}

impl CancelJob {
    /// Create a Cancel-Job operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - job ID<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, job_id: i32, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            job_id,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for CancelJob {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new_internal(self.version(), Operation::CancelJob, Some(self.printer_uri));
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::JOB_ID, IppValue::Integer(self.job_id)),
        );
        with_user_name(self.user_name, &mut retval);
        retval
    }
}

/// IPP operation Get-Job-Attributes
pub struct GetJobAttributes {
    printer_uri: IppString,
    job_id: i32,
    user_name: Option<IppName>,
}

impl GetJobAttributes {
    /// Create a Get-Job-Attributes operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - job ID<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, job_id: i32, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            job_id,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for GetJobAttributes {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::GetJobAttributes, Some(self.printer_uri));
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::JOB_ID, IppValue::Integer(self.job_id)),
        );
        with_user_name(self.user_name, &mut retval);
        retval
    }
}

/// IPP operation Close-Job
pub struct CloseJob {
    printer_uri: IppString,
    job_id: i32,
    user_name: Option<IppName>,
}

impl CloseJob {
    /// Create a Close-Job operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - job ID returned by Create-Job operation<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, job_id: i32, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            job_id,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for CloseJob {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new_internal(self.version(), Operation::CloseJob, Some(self.printer_uri));
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::JOB_ID, IppValue::Integer(self.job_id)),
        );
        with_user_name(self.user_name, &mut retval);
        retval
    }
}

/// IPP operation Resubmit-Job
pub struct ResubmitJob {
    printer_uri: IppString,
    job_id: i32,
    user_name: Option<IppName>,
    document_format: Option<IppMimeMediaType>,
}

impl ResubmitJob {
    /// Create a Resubmit-Job operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - ID of the job to resubmit<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `document_format` - mime-type of the document, if it is being changed<br/>
    pub fn new<U, D>(
        printer_uri: Uri,
        job_id: i32,
        user_name: Option<U>,
        document_format: Option<D>,
    ) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
        D: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            job_id,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
            document_format: document_format.map(|v| v.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for ResubmitJob {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::ResubmitJob, Some(self.printer_uri));
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::JOB_ID, IppValue::Integer(self.job_id)),
        );
        with_user_name(self.user_name, &mut retval);
        with_document_format(self.document_format, &mut retval);
        retval
    }
}

/// IPP operation Cancel-Jobs
pub struct CancelJobs {
    printer_uri: IppString,
    user_name: Option<IppName>,
}

impl CancelJobs {
    /// Create a Cancel-Jobs operation, which cancels all jobs on the printer
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for CancelJobs {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::CancelJobs, Some(self.printer_uri));
        with_user_name(self.user_name, &mut retval);
        retval
    }
}

/// IPP operation Cancel-My-Jobs
pub struct CancelMyJobs {
    printer_uri: IppString,
    user_name: Option<IppName>,
}

impl CancelMyJobs {
    /// Create a Cancel-My-Jobs operation, which cancels the requesting user's jobs
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for CancelMyJobs {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::CancelMyJobs, Some(self.printer_uri));
        with_user_name(self.user_name, &mut retval);
        retval
    }
}

/// IPP operation Identify-Printer
pub struct IdentifyPrinter {
    printer_uri: IppString,
    user_name: Option<IppName>,
    actions: Vec<IppKeyword>,
    message: Option<IppTextValue>,
}

impl IdentifyPrinter {
    /// Create an Identify-Printer operation, which makes the printer flash, sound
    /// or display a message so it can be told apart from another
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `actions` - what the printer should do, e.g. `flash`, `sound`, `display`.
    ///   The printer advertises what it supports in `identify-actions-supported`<br/>
    /// * `message` - text to show, for the `display` action<br/>
    pub fn new<U, A, T, M>(
        printer_uri: Uri,
        user_name: Option<U>,
        actions: A,
        message: Option<M>,
    ) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
        A: IntoIterator<Item = T>,
        T: AsRef<str>,
        M: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
            actions: actions
                .into_iter()
                .map(|a| a.as_ref().try_into())
                .collect::<Result<Vec<IppKeyword>, IppParseError>>()?,
            message: message.map(|m| IppTextValue::new(m.as_ref())).transpose()?,
        })
    }
}

impl IppOperation for IdentifyPrinter {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::IdentifyPrinter, Some(self.printer_uri));

        with_user_name(self.user_name, &mut retval);

        if !self.actions.is_empty() {
            let vals: Vec<IppValue> = self.actions.into_iter().map(IppValue::Keyword).collect();
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(IppAttribute::IDENTIFY_ACTIONS, IppValue::Array(vals)),
            );
        }

        if let Some(message) = self.message {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(IppAttribute::MESSAGE, IppValue::TextWithoutLanguage(message)),
            );
        }

        retval
    }
}

/// IPP operation Get-Printer-Supported-Values
pub struct GetPrinterSupportedValues {
    printer_uri: IppString,
    user_name: Option<IppName>,
    attributes: Vec<IppKeyword>,
}

impl GetPrinterSupportedValues {
    /// Create a Get-Printer-Supported-Values operation to return all supported values
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
            attributes: Vec::new(),
        })
    }

    /// Create a Get-Printer-Supported-Values operation for a given list of attributes
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `attributes` - list of attribute names to ask about<br/>
    pub fn with_attributes<U, I, T>(
        printer_uri: Uri,
        user_name: Option<U>,
        attributes: I,
    ) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
            attributes: attributes
                .into_iter()
                .map(|a| a.as_ref().try_into())
                .collect::<Result<Vec<IppKeyword>, IppParseError>>()?,
        })
    }
}

impl IppOperation for GetPrinterSupportedValues {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new_internal(
            self.version(),
            Operation::GetPrinterSupportedValues,
            Some(self.printer_uri),
        );

        with_user_name(self.user_name, &mut retval);

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

/// IPP operation Set-Printer-Attributes
pub struct SetPrinterAttributes {
    printer_uri: IppString,
    user_name: Option<IppName>,
    attributes: Vec<IppAttribute>,
}

impl SetPrinterAttributes {
    /// Create a Set-Printer-Attributes operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `attributes` - printer attributes to set, sent in the printer attributes group<br/>
    pub fn new<U, I>(printer_uri: Uri, user_name: Option<U>, attributes: I) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
        I: IntoIterator<Item = IppAttribute>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
            attributes: attributes.into_iter().collect(),
        })
    }
}

impl IppOperation for SetPrinterAttributes {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::SetPrinterAttributes, Some(self.printer_uri));

        with_user_name(self.user_name, &mut retval);

        for attribute in self.attributes {
            retval.attributes_mut().add(DelimiterTag::PrinterAttributes, attribute);
        }

        retval
    }
}

/// IPP operation Set-Job-Attributes
pub struct SetJobAttributes {
    printer_uri: IppString,
    job_id: i32,
    user_name: Option<IppName>,
    attributes: Vec<IppAttribute>,
}

impl SetJobAttributes {
    /// Create a Set-Job-Attributes operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - job ID<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    /// * `attributes` - job attributes to set, sent in the job attributes group<br/>
    pub fn new<U, I>(printer_uri: Uri, job_id: i32, user_name: Option<U>, attributes: I) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
        I: IntoIterator<Item = IppAttribute>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            job_id,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
            attributes: attributes.into_iter().collect(),
        })
    }
}

impl IppOperation for SetJobAttributes {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval =
            IppRequestResponse::new_internal(self.version(), Operation::SetJobAttributes, Some(self.printer_uri));

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::JOB_ID, IppValue::Integer(self.job_id)),
        );
        with_user_name(self.user_name, &mut retval);

        for attribute in self.attributes {
            retval.attributes_mut().add(DelimiterTag::JobAttributes, attribute);
        }

        retval
    }
}

/// IPP operation Get-Jobs
pub struct GetJobs {
    printer_uri: IppString,
    user_name: Option<IppName>,
}

impl GetJobs {
    /// Create a Get-Jobs operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `user_name` - name of the user (requesting-user-name)<br/>
    pub fn new<U>(printer_uri: Uri, user_name: Option<U>) -> Result<Self, IppParseError>
    where
        U: AsRef<str>,
    {
        Ok(Self {
            printer_uri: printer_uri.try_into()?,
            user_name: user_name.map(|u| u.as_ref().to_owned().try_into()).transpose()?,
        })
    }
}

impl IppOperation for GetJobs {
    fn into_ipp_request(self) -> IppRequestResponse {
        let mut retval = IppRequestResponse::new_internal(self.version(), Operation::GetJobs, Some(self.printer_uri));

        with_user_name(self.user_name, &mut retval);

        retval
    }
}
