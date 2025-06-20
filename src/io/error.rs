#[non_exhaustive]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    QuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    InProgress,
    InvalidState,
    Other,
    #[doc(hidden)]
    __Uncategorized,
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::NotFound => f.write_str("Not Found"),
            ErrorKind::PermissionDenied => f.write_str("Permission Denied"),
            ErrorKind::ConnectionRefused => f.write_str("Connection Refused"),
            ErrorKind::ConnectionReset => f.write_str("Connection Reset"),
            ErrorKind::HostUnreachable => f.write_str("Host Unreachable"),
            ErrorKind::NetworkUnreachable => f.write_str("Network Unreachable"),
            ErrorKind::ConnectionAborted => f.write_str("Connection Aborted"),
            ErrorKind::NotConnected => f.write_str("Not Connected"),
            ErrorKind::AddrInUse => f.write_str("Address in Use"),
            ErrorKind::AddrNotAvailable => f.write_str("Address Not Available"),
            ErrorKind::NetworkDown => f.write_str("Network Down"),
            ErrorKind::BrokenPipe => f.write_str("Broken Pipe"),
            ErrorKind::AlreadyExists => f.write_str("Already Exists"),
            ErrorKind::WouldBlock => f.write_str("Would Block"),
            ErrorKind::NotADirectory => f.write_str("Not A Directory"),
            ErrorKind::IsADirectory => f.write_str("Is A Directory"),
            ErrorKind::DirectoryNotEmpty => f.write_str("Directory Not Empty"),
            ErrorKind::ReadOnlyFilesystem => f.write_str("Read Only Filesystem"),
            ErrorKind::FilesystemLoop => f.write_str("Filesystem Loop"),
            ErrorKind::StaleNetworkFileHandle => f.write_str("Stale Remote Object"),
            ErrorKind::InvalidInput => f.write_str("Invalid Input"),
            ErrorKind::InvalidData => f.write_str("Invalid Data"),
            ErrorKind::TimedOut => f.write_str("Timed Out"),
            ErrorKind::WriteZero => f.write_str("Write returned 0"),
            ErrorKind::StorageFull => f.write_str("Storage Full"),
            ErrorKind::NotSeekable => f.write_str("Not Seekable"),
            ErrorKind::QuotaExceeded => f.write_str("Quota Exceeded"),
            ErrorKind::FileTooLarge => f.write_str("File Too Large"),
            ErrorKind::ResourceBusy => f.write_str("Resource Busy"),
            ErrorKind::ExecutableFileBusy => f.write_str("Text Busy"),
            ErrorKind::Deadlock => f.write_str("Deadlock (Avoided)"),
            ErrorKind::CrossesDevices => f.write_str("Crosses Devices"),
            ErrorKind::TooManyLinks => f.write_str("Too Many (Hard) Links"),
            ErrorKind::InvalidFilename => f.write_str("Invalid Filename"),
            ErrorKind::ArgumentListTooLong => f.write_str("Argument List Too Long"),
            ErrorKind::Interrupted => f.write_str("Interrupted"),
            ErrorKind::Unsupported => f.write_str("Unsupported"),
            ErrorKind::UnexpectedEof => f.write_str("Unexpected EOF"),
            ErrorKind::OutOfMemory => f.write_str("Out Of Memory"),
            ErrorKind::InProgress => f.write_str("In Progress"),
            ErrorKind::InvalidState => f.write_str("Invalid Object State"),
            ErrorKind::Other => f.write_str("Other Error"),
            ErrorKind::__Uncategorized => f.write_str("(Uncategorized)"),
        }
    }
}

impl From<lilium_sys::result::Error> for ErrorKind {
    fn from(value: lilium_sys::result::Error) -> Self {
        match value {
            lilium_sys::result::Error::Permission => ErrorKind::PermissionDenied,
            lilium_sys::result::Error::InvalidHandle => ErrorKind::InvalidInput,
            lilium_sys::result::Error::InvalidMemory => ErrorKind::InvalidInput,
            lilium_sys::result::Error::Busy => ErrorKind::ResourceBusy,
            lilium_sys::result::Error::InvalidOperation => ErrorKind::InvalidInput,
            lilium_sys::result::Error::InvalidString => ErrorKind::InvalidData,
            lilium_sys::result::Error::InsufficientLength => ErrorKind::InvalidInput,
            lilium_sys::result::Error::ResourceLimitExhausted => ErrorKind::QuotaExceeded,
            lilium_sys::result::Error::InvalidState => ErrorKind::InvalidState,
            lilium_sys::result::Error::InvalidOption => ErrorKind::Unsupported,
            lilium_sys::result::Error::InsufficientMemory => ErrorKind::OutOfMemory,
            lilium_sys::result::Error::UnsupportedKernelFunction => ErrorKind::Unsupported,
            lilium_sys::result::Error::KernelFunctionWouldBlock => ErrorKind::WouldBlock,
            lilium_sys::result::Error::FinishedEnumerate => ErrorKind::__Uncategorized,
            lilium_sys::result::Error::Timeout => ErrorKind::TimedOut,
            lilium_sys::result::Error::Interrupted => ErrorKind::Interrupted,
            lilium_sys::result::Error::Killed => ErrorKind::__Uncategorized,
            lilium_sys::result::Error::Deadlocked => ErrorKind::Deadlock,
            lilium_sys::result::Error::UnsupportedOperation => ErrorKind::Unsupported,
            lilium_sys::result::Error::Pending => ErrorKind::InProgress,
            lilium_sys::result::Error::DoesNotExist => ErrorKind::NotFound,
            lilium_sys::result::Error::AlreadyExists => ErrorKind::AlreadyExists,
            lilium_sys::result::Error::UnknownDevice => ErrorKind::InvalidData,
            lilium_sys::result::Error::WouldBlock => ErrorKind::WouldBlock,
            lilium_sys::result::Error::DeviceFull => ErrorKind::StorageFull,
            lilium_sys::result::Error::DeviceUnavailable => ErrorKind::ResourceBusy,
            lilium_sys::result::Error::LinkResolutionLoop => ErrorKind::FilesystemLoop,
            lilium_sys::result::Error::OrphanedObjects => ErrorKind::__Uncategorized,
            lilium_sys::result::Error::ClosedRemotely => ErrorKind::ConnectionReset,
            lilium_sys::result::Error::ConnectionInterrupted => ErrorKind::ConnectionAborted,
            lilium_sys::result::Error::AddressNotAvailable => ErrorKind::AddrNotAvailable,
            lilium_sys::result::Error::Signaled => ErrorKind::__Uncategorized,
            lilium_sys::result::Error::MappingInaccessible => ErrorKind::InvalidInput,
            lilium_sys::result::Error::PrivilegeCheckFailed => ErrorKind::PermissionDenied,
            lilium_sys::result::Error::InterpError => ErrorKind::NotFound,
            _ => ErrorKind::__Uncategorized,
        }
    }
}

impl error_repr::kind::ErrorKind for ErrorKind {
    const OTHER: Self = Self::Other;
    fn uncategorized() -> Self {
        ErrorKind::__Uncategorized
    }
}

impl error_repr::kind::FromRawOsError for ErrorKind {
    fn from_raw_os_error(raw: error_repr::RawOsError) -> Self {
        match lilium_sys::result::Error::from_code(raw).map_err(Into::into) {
            Ok(()) => Self::Other,
            Err(v) => v,
        }
    }
}

pub type Error = error_repr::Error<ErrorKind>;
