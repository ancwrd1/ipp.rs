use crate::{
    attribute::IppAttribute,
    operation::{CreateJob, GetPrinterAttributes, IppOperation, PrintJob, SendDocument},
    IppReadStream,
};

/// Builder to create IPP operations
pub struct IppOperationBuilder;

impl IppOperationBuilder {
    /// Create PrintJob operation
    pub fn print_job<T>(stream: T) -> PrintJobBuilder
    where
        IppReadStream: From<T>,
    {
        PrintJobBuilder::new(stream.into())
    }

    /// Create GetPrinterAttributes operation
    pub fn get_printer_attributes() -> GetPrinterAttributesBuilder {
        GetPrinterAttributesBuilder::new()
    }

    /// Create CreateJob operation
    pub fn create_job() -> CreateJobBuilder {
        CreateJobBuilder::new()
    }

    /// Create SendDocument operation
    pub fn send_document<T>(job_id: i32, stream: T) -> SendDocumentBuilder
    where
        IppReadStream: From<T>,
    {
        SendDocumentBuilder::new(job_id, stream.into())
    }
}

/// Builder to create PrintJob operation
pub struct PrintJobBuilder {
    stream: IppReadStream,
    user_name: Option<String>,
    job_title: Option<String>,
    attributes: Vec<IppAttribute>,
}

impl PrintJobBuilder {
    fn new(stream: IppReadStream) -> PrintJobBuilder {
        PrintJobBuilder {
            stream,
            user_name: None,
            job_title: None,
            attributes: Vec::new(),
        }
    }
    /// Specify requesting-user-name attribute
    pub fn user_name(mut self, user_name: &str) -> Self {
        self.user_name = Some(user_name.to_owned());
        self
    }

    /// Specify job-name attribute
    pub fn job_title(mut self, job_title: &str) -> Self {
        self.job_title = Some(job_title.to_owned());
        self
    }

    /// Specify custom job attribute
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        let mut op = PrintJob::new(self.stream, self.user_name.as_ref(), self.job_title.as_ref());
        for attr in self.attributes {
            op.add_attribute(attr);
        }
        op
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
    pub fn attribute(mut self, attribute: &str) -> Self {
        self.attributes.push(attribute.to_owned());
        self
    }

    /// Specify which attributes to retrieve from the printer
    pub fn attributes<T>(mut self, attributes: &[T]) -> Self
    where
        T: AsRef<str>,
    {
        self.attributes
            .extend(attributes.iter().map(|s| s.as_ref().to_string()));
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
    pub fn job_name(mut self, job_name: &str) -> Self {
        self.job_name = Some(job_name.to_owned());
        self
    }

    /// Specify custom job attribute
    pub fn attribute(mut self, attribute: IppAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        let mut op = CreateJob::new(self.job_name.as_ref());
        for attr in self.attributes {
            op.add_attribute(attr);
        }
        op
    }
}

/// Builder to create SendDocument operation
pub struct SendDocumentBuilder {
    job_id: i32,
    stream: IppReadStream,
    user_name: Option<String>,
    is_last: bool,
}

impl SendDocumentBuilder {
    fn new(job_id: i32, stream: IppReadStream) -> SendDocumentBuilder {
        SendDocumentBuilder {
            job_id,
            stream,
            user_name: None,
            is_last: true,
        }
    }

    /// Specify originating-user-name attribute
    pub fn user_name(mut self, user_name: &str) -> Self {
        self.user_name = Some(user_name.to_owned());
        self
    }

    /// Parameter which indicates whether this document is a last one
    pub fn last(mut self, last: bool) -> Self {
        self.is_last = last;
        self
    }

    /// Build operation
    pub fn build(self) -> impl IppOperation {
        SendDocument::new(self.job_id, self.stream, self.user_name.as_ref(), self.is_last)
    }
}
