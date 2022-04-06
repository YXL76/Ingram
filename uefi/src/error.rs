use {core::result, uefi::Status};

pub(crate) type Result<T> = result::Result<T, Error>;

/// The error type for UEFI.
#[derive(Debug)]
pub(crate) enum Error {
    Simple(ErrorKind),
    UEFI(Status),
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// An entity was not found, often a file.
    NotFound,
    /// The filesystem object is, unexpectedly, a directory.
    ///
    /// A directory was specified when a non-directory was expected.
    IsADirectory,
}

impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    ///
    /// This conversion allocates a new error with a simple representation of error kind.
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error::Simple(kind)
    }
}

impl From<Status> for Error {
    /// Converts an [`Status`] into an [`Error`].
    #[inline]
    fn from(status: Status) -> Error {
        Error::UEFI(status)
    }
}
