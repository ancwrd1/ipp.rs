use super::{
    attribute::IppAttribute,
    operation::{
        cups::{CupsDeletePrinter, CupsGetPrinters},
        CreateJob, GetPrinterAttributes, IppOperation, PrintJob, SendDocument,
    },
    IppPayload,
};

/// Builder to create IPP operations
pub struct IppOperationBuilder;

impl IppOperationBuilder {
    /// Create PrintJob operation
    ///
    /// * `payload` - `IppPayload`
    pub fn print_job<T>(payload: T) -> PrintJobBuilder
    where
        IppPayload: From<T>,
    {
        PrintJobBuilder::new(payload.into())
    }

    /// Create GetPrinterAttributes operation
    pub fn get_printer_attributes() -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder::new()
    }

    /// Create CreateJob operation
    pub fn create_job() -> CreateJobBuilder {
        CreateJobBuilder::new()
    }

    /// Create CUPS-specific operations
    pub fn cups() -> CupsBuilder {
        CupsBuilder::new()
    }

    /// Create SendDocument operation
    ///
    /// * `job_id` - job id returned by Create-Job operation <br/>
    /// * `payload` - `IppPayload` <br/>
    pub fn send_document<T>(job_id: i32, payload: T) -> SendDocumentBuilder
    where
        IppPayload: From<T>,
    {
        SendDocumentBuilder::new(job_id, payload.into())
    }
}

/// Builder to create PrintJob operation
pub struct PrintJobBuilder {
    payload: IppPayload,
    user_name: Option<String>,
    job_title: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl PrintJobBuilder {
    fn new(payload: IppPayload) -> PrintJobBuilder {
        PrintJobBuilder {
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

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        let op = PrintJob::new(self.payload, self.user_name.as_ref(), self.job_title.as_ref());
        self.attributes.into_iter().fold(op, |mut op, attr| {
            op.add_attribute(attr);
            op
        })
    }
}

/// Builder to create GetPrinterAttributes operation
pub struct GetPrinterAttributesBuilder {
    attributes: Vec<String>,
}

impl GetPrinterAttributesBuilder {
    fn new() -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder { attributes: Vec::new() }
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
        GetPrinterAttributes::with_attributes(&self.attributes)
    }
}

/// Builder to create CreateJob operation
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

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        let op = CreateJob::new(self.job_name.as_ref());
        self.attributes.into_iter().fold(op, |mut op, attr| {
            op.add_attribute(attr);
            op
        })
    }
}

/// Builder to create SendDocument operation
pub struct SendDocumentBuilder {
    job_id: i32,
    payload: IppPayload,
    user_name: Option<String>,
    is_last: bool,
}

impl SendDocumentBuilder {
    fn new(job_id: i32, payload: IppPayload) -> SendDocumentBuilder {
        SendDocumentBuilder {
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
        SendDocument::new(self.job_id, self.payload, self.user_name.as_ref(), self.is_last)
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
    pub fn delete_printer(&self) -> impl IppOperation {
        CupsDeletePrinter::new()
    }
}
