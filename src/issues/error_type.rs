/// Represents the types of errors that can be generated by `tweep`
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorType {
    /// Passage header has no name specified
    EmptyName,

    /// Passage header has whitespace before sigil `::`
    LeadingWhitespace,

    /// Passage header has metadata and tags in wrong order
    MetadataBeforeTags,

    /// Passage header is missing sigil `::`
    MissingSigil,

    /// Passage name has an unescaped `[` character
    UnescapedOpenSquare,

    /// Passage name has an unescaped `{` character
    UnescapedOpenCurly,

    /// Passage name has an unescaped `]` character
    UnescapedCloseSquare,

    /// Passage name has an unescaped `}` character
    UnescapedCloseCurly,

    /// Passage header has an unclosed tag block
    UnclosedTagBlock,

    /// Passage header has an unclosed metadata block
    UnclosedMetadataBlock,

    /// An error was encountered when attempting to parse from the given [`Path`](std::path::Path).
    /// Contains the path string and the error string
    BadInputPath(String, String),
    
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ErrorType::EmptyName => "Passage header has an empty name".to_string(),
            ErrorType::LeadingWhitespace => "Passage header has whitespace before sigil (::)".to_string(),
            ErrorType::MetadataBeforeTags => "Passage header has metadata before tags".to_string(),
            ErrorType::MissingSigil => "Passage header missing sigil (::)".to_string(),
            ErrorType::UnescapedOpenSquare => "Unescaped [ character in passage header".to_string(),
            ErrorType::UnescapedOpenCurly => "Unescaped { character in passage header".to_string(),
            ErrorType::UnescapedCloseSquare => "Unescaped ] character in passage header".to_string(),
            ErrorType::UnescapedCloseCurly => "Unescaped } character in passage header".to_string(),
            ErrorType::UnclosedTagBlock => "Unclosed tag block in passage header".to_string(),
            ErrorType::UnclosedMetadataBlock => "Unclosed metadata block in passage header".to_string(),
            ErrorType::BadInputPath(path, err_str) => format!("Error opening path {}: {}", path, err_str),
        })
    }
}
