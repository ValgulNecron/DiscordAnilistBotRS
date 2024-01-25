#[derive(Debug, Clone)]
pub enum AppError {
    OptionError(String),
    CommandSendingError(String),
    LocalisationFileError(String),
    LocalisationReadError(String),
    LocalisationParsingError(String),
    NoLangageError(String),
    FailedToGetUser(String),
    NoCommandOption(String),
    SqlInsertError(String),
    SqlSelectError(String),
    SqlCreateError(String),
    ModuleError(String),
    ModuleOffError(String),
    UnknownCommandError(String),
    NoMediaDifferedError(String),
    CreatingWebhookDifferedError(String),
    CreatingPoolError(String),
    FailedToCreateAFile(String),
    DifferedTokenError(String),
    DifferedImageModelError(String),
    DifferedHeaderError(String),
    DifferedResponseError(String),
    DifferedFailedUrlError(String),
    DifferedOptionError(String),
    DifferedFailedToGetBytes(String),
    DifferedWritingFile(String),
    DifferedCommandSendingError(String),
    SetLoggerError(String),
    DifferedFileTypeError(String),
    DifferedFileExtensionError(String),
    DifferedCopyBytesError(String),
    DifferedGettingBytesError(String),
    MediaGettingError(String),
    UserGettingError(String),
    DifferedNotAiringError(String),
    NoStatisticDifferedError(String),
    NotAValidTypeError(String),
    DifferedCreatingImageError(String),
    NotNSFWError(String),
    NotAValidUrlError(String),
    NotAValidGameError(String),
    ErrorGettingUserList(String),
    CreatingImageError(String),
    DecodingImageError(String),
    FailedToGetImage(String),
    FailedToCreateFolder(String),
    FailedToUploadImage(String),
    FailedToWriteFile(String),
    FailedToUpdateDatabase(String),
}
