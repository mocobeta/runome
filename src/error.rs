use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunomeError {
    // Dictionary loading errors
    #[error("Dictionary directory not found: {path}")]
    DictDirectoryNotFound { path: String },

    #[error("Required dictionary file missing: {filename}")]
    DictFileMissing { filename: String },

    #[error("Failed to deserialize dictionary {component}: {source}")]
    DictDeserializationError {
        component: String,
        #[source]
        source: bincode::Error,
    },

    #[error("Invalid connection matrix access: left_id={left_id}, right_id={right_id}")]
    InvalidConnectionId { left_id: u16, right_id: u16 },

    #[error("Dictionary validation failed: {reason}")]
    DictValidationError { reason: String },

    #[error("Character classification error: {reason}")]
    CharClassificationError { reason: String },

    #[error("SystemDictionary initialization failed: {reason}")]
    SystemDictInitError { reason: String },

    // User dictionary errors
    #[error("User dictionary error: {reason}")]
    UserDictError { reason: String },

    #[error("CSV parsing error at line {line}: {reason}")]
    CsvParseError { line: usize, reason: String },

    #[error("FST building error: {reason}")]
    FstBuildError { reason: String },

    // CharFilter errors
    #[error("Invalid regex pattern: {pattern}")]
    InvalidRegexPattern {
        pattern: String,
        #[source]
        source: regex::Error,
    },

    #[error("Invalid Unicode normalization form: {form}")]
    InvalidNormalizationForm { form: String },

    #[error("CharFilter error: {message}")]
    CharFilterError { message: String },

    // Analyzer errors
    #[error("Analyzer error: {message}")]
    AnalyzerError { message: String },

    #[error("Invalid tokenizer configuration: {reason}")]
    InvalidTokenizerConfig { reason: String },

    #[error("Filter chain error: {message}")]
    FilterChainError { message: String },

    // General IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    // Future: Add variants for other components (FST, tokenizer, etc.)
}

pub type Result<T> = std::result::Result<T, RunomeError>;
