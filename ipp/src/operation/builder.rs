//!
//! IPP operation builders
//!
use http::Uri;

use crate::{
    attribute::IppAttribute,
    operation::{cups::*, *},
    payload::IppPayload,
};

/// Builder to create IPP operations
pub struct IppOperationBuilder;

impl IppOperationBuilder {
    /// Create a Print-Job operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `payload` - `IppPayload`
    pub fn print_job(printer_uri: Uri, payload: IppPayload) -> PrintJobBuilder {
        PrintJobBuilder::new(printer_uri, payload)
    }

    /// Create a Get-Printer-Attributes operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn get_printer_attributes(printer_uri: Uri) -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder::new(printer_uri)
    }

    /// Create a Create-Job operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn create_job(printer_uri: Uri) -> CreateJobBuilder {
        CreateJobBuilder::new(printer_uri)
    }

    /// Create CUPS-specific operations
    pub fn cups() -> CupsBuilder {
        CupsBuilder::new()
    }

    /// Create a Send-Document operation builder
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - job id returned by Create-Job operation <br/>
    /// * `payload` - `IppPayload`
    pub fn send_document(printer_uri: Uri, job_id: i32, payload: IppPayload) -> SendDocumentBuilder {
        SendDocumentBuilder::new(printer_uri, job_id, payload)
    }

    /// Create a Purge-Jobs operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn purge_jobs(printer_uri: Uri) -> PurgeJobsBuilder {
        PurgeJobsBuilder::new(printer_uri)
    }

    /// Create a Cancel-Job operation builder
    ///
    /// * `printer_uri` - printer URI
    /// * `job_id` - job id to cancel <br/>
    pub fn cancel_job(printer_uri: Uri, job_id: i32) -> CancelJobBuilder {
        CancelJobBuilder::new(printer_uri, job_id)
    }

    /// Create a Get-Job-Attributes operation builder
    ///
    /// * `printer_uri` - printer URI
    /// * `job_id` - job id <br/>
    pub fn get_job_attributes(printer_uri: Uri, job_id: i32) -> GetJobAttributesBuilder {
        GetJobAttributesBuilder::new(printer_uri, job_id)
    }

    /// Create a Close-Job operation builder
    ///
    /// * `printer_uri` - printer URI
    /// * `job_id` - job id to close <br/>
    pub fn close_job(printer_uri: Uri, job_id: i32) -> CloseJobBuilder {
        CloseJobBuilder::new(printer_uri, job_id)
    }

    /// Create a Resubmit-Job operation builder
    ///
    /// * `printer_uri` - printer URI
    /// * `job_id` - job id to resubmit <br/>
    pub fn resubmit_job(printer_uri: Uri, job_id: i32) -> ResubmitJobBuilder {
        ResubmitJobBuilder::new(printer_uri, job_id)
    }

    /// Create a Cancel-Jobs operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn cancel_jobs(printer_uri: Uri) -> CancelJobsBuilder {
        CancelJobsBuilder::new(printer_uri)
    }

    /// Create a Cancel-My-Jobs operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn cancel_my_jobs(printer_uri: Uri) -> CancelMyJobsBuilder {
        CancelMyJobsBuilder::new(printer_uri)
    }

    /// Create an Identify-Printer operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn identify_printer(printer_uri: Uri) -> IdentifyPrinterBuilder {
        IdentifyPrinterBuilder::new(printer_uri)
    }

    /// Create a Get-Printer-Supported-Values operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn get_printer_supported_values(printer_uri: Uri) -> GetPrinterSupportedValuesBuilder {
        GetPrinterSupportedValuesBuilder::new(printer_uri)
    }

    /// Create a Set-Printer-Attributes operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn set_printer_attributes(printer_uri: Uri) -> SetPrinterAttributesBuilder {
        SetPrinterAttributesBuilder::new(printer_uri)
    }

    /// Create a Set-Job-Attributes operation builder
    ///
    /// * `printer_uri` - printer URI
    /// * `job_id` - job id to modify <br/>
    pub fn set_job_attributes(printer_uri: Uri, job_id: i32) -> SetJobAttributesBuilder {
        SetJobAttributesBuilder::new(printer_uri, job_id)
    }

    /// Create a Get-Jobs operation builder
    ///
    /// * `printer_uri` - printer URI
    pub fn get_jobs(printer_uri: Uri) -> GetJobsBuilder {
        GetJobsBuilder::new(printer_uri)
    }
}

/// Builder to create a Print-Job operation
pub struct PrintJobBuilder {
    printer_uri: Uri,
    payload: IppPayload,
    user_name: Option<String>,
    job_title: Option<String>,
    document_format: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl PrintJobBuilder {
    fn new(printer_uri: Uri, payload: IppPayload) -> PrintJobBuilder {
        PrintJobBuilder {
            printer_uri,
            payload,
            user_name: None,
            job_title: None,
            document_format: None,
            attributes: Vec::new(),
        }
    }
    /// Specify the requesting-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify the job-name attribute
    pub fn job_title<S>(mut self, job_title: S) -> Self
    where
        S: AsRef<str>,
    {
        self.job_title = Some(job_title.as_ref().to_owned());
        self
    }

    /// Specify the mime-type of the document, e.g. "image/jpeg"
    pub fn document_format<S>(mut self, document_format: S) -> Self
    where
        S: AsRef<str>,
    {
        self.document_format = Some(document_format.as_ref().to_owned());
        self
    }

    /// Specify a custom job attribute
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Specify a custom job attributes
    pub fn attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = IppAttribute>,
    {
        self.attributes.extend(attributes);
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        let op = PrintJob::new(
            self.printer_uri,
            self.payload,
            self.user_name.as_ref(),
            self.job_title.as_ref(),
            self.document_format.as_ref(),
        )?;
        Ok(self.attributes.into_iter().fold(op, |mut op, attr| {
            op.add_attribute(attr);
            op
        }))
    }
}

/// Builder to create a Get-Printer-Attributes operation
pub struct GetPrinterAttributesBuilder {
    printer_uri: Uri,
    attributes: Vec<String>,
}

impl GetPrinterAttributesBuilder {
    fn new(printer_uri: Uri) -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder {
            printer_uri,
            attributes: Vec::new(),
        }
    }

    /// Specify which attribute to retrieve from the printer. Can be repeated.
    pub fn attribute<S>(mut self, attribute: S) -> Self
    where
        S: AsRef<str>,
    {
        self.attributes.push(attribute.as_ref().to_owned());
        self
    }

    /// Specify which attributes to retrieve from the printer
    pub fn attributes<S, I>(mut self, attributes: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        self.attributes
            .extend(attributes.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        GetPrinterAttributes::with_attributes(self.printer_uri, &self.attributes)
    }
}

/// Builder to create a Create-Job operation
pub struct CreateJobBuilder {
    printer_uri: Uri,
    job_name: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl CreateJobBuilder {
    fn new(printer_uri: Uri) -> CreateJobBuilder {
        CreateJobBuilder {
            printer_uri,
            job_name: None,
            attributes: Vec::new(),
        }
    }

    /// Specify the job-name attribute
    pub fn job_name<S>(mut self, job_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.job_name = Some(job_name.as_ref().to_owned());
        self
    }

    /// Specify a custom job attribute
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Specify a custom job attributes
    pub fn attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = IppAttribute>,
    {
        self.attributes.extend(attributes);
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        let op = CreateJob::new(self.printer_uri, self.job_name.as_ref())?;
        Ok(self.attributes.into_iter().fold(op, |mut op, attr| {
            op.add_attribute(attr);
            op
        }))
    }
}

/// Builder to create a Send-Document operation
pub struct SendDocumentBuilder {
    printer_uri: Uri,
    job_id: i32,
    payload: IppPayload,
    user_name: Option<String>,
    document_format: Option<String>,
    is_last: bool,
}

impl SendDocumentBuilder {
    fn new(printer_uri: Uri, job_id: i32, payload: IppPayload) -> SendDocumentBuilder {
        SendDocumentBuilder {
            printer_uri,
            job_id,
            payload,
            user_name: None,
            document_format: None,
            is_last: true,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify the mime-type of the document, e.g. "image/jpeg"
    pub fn document_format<S>(mut self, document_format: S) -> Self
    where
        S: AsRef<str>,
    {
        self.document_format = Some(document_format.as_ref().to_owned());
        self
    }

    /// Parameter which indicates whether this document is the last one
    pub fn last(mut self, last: bool) -> Self {
        self.is_last = last;
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        SendDocument::new(
            self.printer_uri,
            self.job_id,
            self.payload,
            self.user_name.as_ref(),
            self.document_format.as_ref(),
            self.is_last,
        )
    }
}

/// Builder to create a Purge-Jobs operation
pub struct PurgeJobsBuilder {
    printer_uri: Uri,
    user_name: Option<String>,
}

impl PurgeJobsBuilder {
    fn new(printer_uri: Uri) -> PurgeJobsBuilder {
        PurgeJobsBuilder {
            printer_uri,
            user_name: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        PurgeJobs::new(self.printer_uri, self.user_name)
    }
}

/// Builder to create a Cancel-Job operation
pub struct CancelJobBuilder {
    printer_uri: Uri,
    job_id: i32,
    user_name: Option<String>,
}

impl CancelJobBuilder {
    fn new(printer_uri: Uri, job_id: i32) -> CancelJobBuilder {
        CancelJobBuilder {
            printer_uri,
            job_id,
            user_name: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        CancelJob::new(self.printer_uri, self.job_id, self.user_name)
    }
}

/// Builder to create a Get-Job-Attributes operation
pub struct GetJobAttributesBuilder {
    printer_uri: Uri,
    job_id: i32,
    user_name: Option<String>,
}

impl GetJobAttributesBuilder {
    fn new(printer_uri: Uri, job_id: i32) -> GetJobAttributesBuilder {
        GetJobAttributesBuilder {
            printer_uri,
            job_id,
            user_name: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        GetJobAttributes::new(self.printer_uri, self.job_id, self.user_name)
    }
}

/// Builder to create a Get-Jobs operation
pub struct GetJobsBuilder {
    printer_uri: Uri,
    user_name: Option<String>,
}

impl GetJobsBuilder {
    fn new(printer_uri: Uri) -> GetJobsBuilder {
        GetJobsBuilder {
            printer_uri,
            user_name: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        GetJobs::new(self.printer_uri, self.user_name)
    }
}

/// Builder to create a Close-Job operation
pub struct CloseJobBuilder {
    printer_uri: Uri,
    job_id: i32,
    user_name: Option<String>,
}

impl CloseJobBuilder {
    fn new(printer_uri: Uri, job_id: i32) -> CloseJobBuilder {
        CloseJobBuilder {
            printer_uri,
            job_id,
            user_name: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        CloseJob::new(self.printer_uri, self.job_id, self.user_name)
    }
}

/// Builder to create a Resubmit-Job operation
pub struct ResubmitJobBuilder {
    printer_uri: Uri,
    job_id: i32,
    user_name: Option<String>,
    document_format: Option<String>,
}

impl ResubmitJobBuilder {
    fn new(printer_uri: Uri, job_id: i32) -> ResubmitJobBuilder {
        ResubmitJobBuilder {
            printer_uri,
            job_id,
            user_name: None,
            document_format: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify the document-format attribute
    pub fn document_format<S>(mut self, document_format: S) -> Self
    where
        S: AsRef<str>,
    {
        self.document_format = Some(document_format.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        ResubmitJob::new(self.printer_uri, self.job_id, self.user_name, self.document_format)
    }
}

/// Builder to create a Cancel-Jobs operation
pub struct CancelJobsBuilder {
    printer_uri: Uri,
    user_name: Option<String>,
}

impl CancelJobsBuilder {
    fn new(printer_uri: Uri) -> CancelJobsBuilder {
        CancelJobsBuilder {
            printer_uri,
            user_name: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        CancelJobs::new(self.printer_uri, self.user_name)
    }
}

/// Builder to create a Cancel-My-Jobs operation
pub struct CancelMyJobsBuilder {
    printer_uri: Uri,
    user_name: Option<String>,
}

impl CancelMyJobsBuilder {
    fn new(printer_uri: Uri) -> CancelMyJobsBuilder {
        CancelMyJobsBuilder {
            printer_uri,
            user_name: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        CancelMyJobs::new(self.printer_uri, self.user_name)
    }
}

/// Builder to create an Identify-Printer operation
pub struct IdentifyPrinterBuilder {
    printer_uri: Uri,
    user_name: Option<String>,
    actions: Vec<String>,
    message: Option<String>,
}

impl IdentifyPrinterBuilder {
    fn new(printer_uri: Uri) -> IdentifyPrinterBuilder {
        IdentifyPrinterBuilder {
            printer_uri,
            user_name: None,
            actions: Vec::new(),
            message: None,
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify what the printer should do, e.g. `flash`, `sound` or `display`. Can be repeated.
    pub fn action<S>(mut self, action: S) -> Self
    where
        S: AsRef<str>,
    {
        self.actions.push(action.as_ref().to_owned());
        self
    }

    /// Specify the message to display, for the `display` action
    pub fn message<S>(mut self, message: S) -> Self
    where
        S: AsRef<str>,
    {
        self.message = Some(message.as_ref().to_owned());
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        IdentifyPrinter::new(self.printer_uri, self.user_name, &self.actions, self.message)
    }
}

/// Builder to create a Get-Printer-Supported-Values operation
pub struct GetPrinterSupportedValuesBuilder {
    printer_uri: Uri,
    user_name: Option<String>,
    attributes: Vec<String>,
}

impl GetPrinterSupportedValuesBuilder {
    fn new(printer_uri: Uri) -> GetPrinterSupportedValuesBuilder {
        GetPrinterSupportedValuesBuilder {
            printer_uri,
            user_name: None,
            attributes: Vec::new(),
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify which attribute to ask about. Can be repeated.
    pub fn attribute<S>(mut self, attribute: S) -> Self
    where
        S: AsRef<str>,
    {
        self.attributes.push(attribute.as_ref().to_owned());
        self
    }

    /// Specify which attributes to ask about
    pub fn attributes<S, I>(mut self, attributes: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        self.attributes
            .extend(attributes.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        GetPrinterSupportedValues::with_attributes(self.printer_uri, self.user_name, &self.attributes)
    }
}

/// Builder to create a Set-Printer-Attributes operation
pub struct SetPrinterAttributesBuilder {
    printer_uri: Uri,
    user_name: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl SetPrinterAttributesBuilder {
    fn new(printer_uri: Uri) -> SetPrinterAttributesBuilder {
        SetPrinterAttributesBuilder {
            printer_uri,
            user_name: None,
            attributes: Vec::new(),
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify a printer attribute to set. Can be repeated.
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Specify the printer attributes to set
    pub fn attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = IppAttribute>,
    {
        self.attributes.extend(attributes);
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        SetPrinterAttributes::new(self.printer_uri, self.user_name, self.attributes)
    }
}

/// Builder to create a Set-Job-Attributes operation
pub struct SetJobAttributesBuilder {
    printer_uri: Uri,
    job_id: i32,
    user_name: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl SetJobAttributesBuilder {
    fn new(printer_uri: Uri, job_id: i32) -> SetJobAttributesBuilder {
        SetJobAttributesBuilder {
            printer_uri,
            job_id,
            user_name: None,
            attributes: Vec::new(),
        }
    }

    /// Specify the originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify a job attribute to set. Can be repeated.
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Specify the job attributes to set
    pub fn attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = IppAttribute>,
    {
        self.attributes.extend(attributes);
        self
    }

    /// Build the operation
    pub fn build(self) -> Result<impl IppOperation, IppParseError> {
        SetJobAttributes::new(self.printer_uri, self.job_id, self.user_name, self.attributes)
    }
}

/// CUPS operations builder
pub struct CupsBuilder;

impl CupsBuilder {
    fn new() -> CupsBuilder {
        CupsBuilder
    }

    /// CUPS-Get-Printers operation
    pub fn get_printers(&self) -> impl IppOperation {
        CupsGetPrinters::new()
    }

    /// CUPS-Delete-Printer operation
    pub fn delete_printer(&self, printer_uri: Uri) -> Result<impl IppOperation, IppParseError> {
        CupsDeletePrinter::new(printer_uri)
    }
}
