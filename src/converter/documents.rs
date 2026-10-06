#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Text,
    Sheet,
    Slides,
}

pub struct Document {
    pub extension: &'static str,
    pub kind: Kind,
}

// libreoffice picks the export filter from the extension; any kind of file can become a pdf
pub const DOCUMENTS: &[Document] = &[
    Document { extension: "pdf", kind: Kind::Text },
    Document { extension: "docx", kind: Kind::Text },
    Document { extension: "odt", kind: Kind::Text },
    Document { extension: "rtf", kind: Kind::Text },
    Document { extension: "xlsx", kind: Kind::Sheet },
    Document { extension: "ods", kind: Kind::Sheet },
    Document { extension: "csv", kind: Kind::Sheet },
    Document { extension: "pptx", kind: Kind::Slides },
    Document { extension: "odp", kind: Kind::Slides },
];
