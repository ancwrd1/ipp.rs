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
    /// Create PrintJob operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `payload` - `IppPayload`
    pub fn print_job(printer_uri: Uri, payload: IppPayload) -> PrintJobBuilder {
        PrintJobBuilder::new(printer_uri, payload)
    }

    /// Create GetPrinterAttributes operation
    ///
    /// * `printer_uri` - printer URI
    pub fn get_printer_attributes(printer_uri: Uri) -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder::new(printer_uri)
    }

    /// Create CreateJob operation
    ///
    /// * `printer_uri` - printer URI
    pub fn create_job(printer_uri: Uri) -> CreateJobBuilder {
        CreateJobBuilder::new(printer_uri)
    }

    /// Create CUPS-specific operations
    pub fn cups() -> CupsBuilder {
        CupsBuilder::new()
    }

    /// Create SendDocument operation
    ///
    /// * `printer_uri` - printer URI<br/>
    /// * `job_id` - job id returned by Create-Job operation <br/>
    /// * `payload` - `IppPayload`
    pub fn send_document(printer_uri: Uri, job_id: i32, payload: IppPayload) -> SendDocumentBuilder {
        SendDocumentBuilder::new(printer_uri, job_id, payload)
    }

    /// Create PurgeJobs operation
    ///
    /// * `printer_uri` - printer URI
    pub fn purge_jobs(printer_uri: Uri) -> PurgeJobsBuilder {
        PurgeJobsBuilder::new(printer_uri)
    }

    /// Create PurgeJobs operation
    ///
    /// * `printer_uri` - printer URI
    /// * `job_id` - job id to cancel <br/>
    pub fn cancel_job(printer_uri: Uri, job_id: i32) -> CancelJobBuilder {
        CancelJobBuilder::new(printer_uri, job_id)
    }

    /// Create PurgeJobs operation
    ///
    /// * `printer_uri` - printer URI
    pub fn get_jobs(printer_uri: Uri) -> GetJobsBuilder {
        GetJobsBuilder::new(printer_uri)
    }
}

/// Builder to create PrintJob operation
pub struct PrintJobBuilder {
    printer_uri: Uri,
    payload: IppPayload,
    user_name: Option<String>,
    job_title: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl PrintJobBuilder {
    fn new(printer_uri: Uri, payload: IppPayload) -> PrintJobBuilder {
        PrintJobBuilder {
            printer_uri,
            payload,
            user_name: None,
            job_title: None,
            attributes: Vec::new(),
        }
    }
    /// Specify requesting-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Specify job-name attribute
    pub fn job_title<S>(mut self, job_title: S) -> Self
    where
        S: AsRef<str>,
    {
        self.job_title = Some(job_title.as_ref().to_owned());
        self
    }

    /// Specify custom job attribute
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Specify custom job attributes
    pub fn attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = IppAttribute>,
    {
        self.attributes.extend(attributes.into_iter());
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        let op = PrintJob::new(
            self.printer_uri,
            self.payload,
            self.user_name.as_ref(),
            self.job_title.as_ref(),
        );
        self.attributes.into_iter().fold(op, |mut op, attr| {
            op.add_attribute(attr);
            op
        })
    }
}

/// Builder to create GetPrinterAttributes operation
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

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        GetPrinterAttributes::with_attributes(self.printer_uri, &self.attributes)
    }
}

/// Builder to create CreateJob operation
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

    /// Specify job-name attribute
    pub fn job_name<S>(mut self, job_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.job_name = Some(job_name.as_ref().to_owned());
        self
    }

    /// Specify custom job attribute
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Specify custom job attributes
    pub fn attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = IppAttribute>,
    {
        self.attributes.extend(attributes.into_iter());
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        let op = CreateJob::new(self.printer_uri, self.job_name.as_ref());
        self.attributes.into_iter().fold(op, |mut op, attr| {
            op.add_attribute(attr);
            op
        })
    }
}

/// Builder to create SendDocument operation
pub struct SendDocumentBuilder {
    printer_uri: Uri,
    job_id: i32,
    payload: IppPayload,
    user_name: Option<String>,
    is_last: bool,
}

impl SendDocumentBuilder {
    fn new(printer_uri: Uri, job_id: i32, payload: IppPayload) -> SendDocumentBuilder {
        SendDocumentBuilder {
            printer_uri,
            job_id,
            payload,
            user_name: None,
            is_last: true,
        }
    }

    /// Specify originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Parameter which indicates whether this document is a last one
    pub fn last(mut self, last: bool) -> Self {
        self.is_last = last;
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        SendDocument::new(
            self.printer_uri,
            self.job_id,
            self.payload,
            self.user_name.as_ref(),
            self.is_last,
        )
    }
}

/// Builder to create PurgeJobs operation
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

    /// Specify originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        PurgeJobs::new(self.printer_uri, self.user_name)
    }
}

/// Builder to create CancelJob operation
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

    /// Specify originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        CancelJob::new(self.printer_uri, self.job_id, self.user_name)
    }
}

/// Builder to create GetJobs operation
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

    /// Specify originating-user-name attribute
    pub fn user_name<S>(mut self, user_name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.user_name = Some(user_name.as_ref().to_owned());
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        GetJobs::new(self.printer_uri, self.user_name)
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
    pub fn delete_printer(&self, printer_uri: Uri) -> impl IppOperation {
        CupsDeletePrinter::new(printer_uri)
    }
}
