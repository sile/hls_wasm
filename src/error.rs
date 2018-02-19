use std;
use hls_m3u8;
use trackable::Trackable;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt, Event, TrackableError};
use url;

/// This crate specific error type.
#[derive(Debug, Clone)]
pub struct Error(TrackableError<ErrorKind>);
derive_traits_for_trackable_error_newtype!(Error, ErrorKind);
impl Error {
    pub fn to_json_string(&self) -> String {
        use std::error::Error as StdError;

        let kind = format!("{:?}", self.kind());
        let reason = self.cause()
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "".to_owned());
        let mut trace = Vec::new();
        for event in self.history().iter().flat_map(|h| h.events()) {
            if let Event::Track(ref location) = *event {
                if location.message().is_empty() {
                    trace.push(json!({
                            "file": location.file().to_owned(),
                            "line": location.line()
                        }));
                } else {
                    trace.push(json!({
                            "file": location.file().to_owned(),
                            "line": location.line(),
                            "messsage": location.message().to_owned()
                        }));
                }
            }
        }
        let json = json!({
            "kind": kind,
            "reason": reason,
            "trace": trace
            });
        json.to_string()
    }
}
impl From<hls_m3u8::Error> for Error {
    fn from(f: hls_m3u8::Error) -> Self {
        match *f.kind() {
            hls_m3u8::ErrorKind::InvalidInput => ErrorKind::InvalidInput.takes_over(f).into(),
        }
    }
}
impl From<url::ParseError> for Error {
    fn from(f: url::ParseError) -> Self {
        ErrorKind::InvalidInput.cause(f).into()
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(f: std::str::Utf8Error) -> Self {
        ErrorKind::InvalidInput.cause(f).into()
    }
}

/// The list of the possible error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Input data is invalid.
    InvalidInput,

    /// Other errors (e.g., I/O error).
    Other,
}
impl TrackableErrorKind for ErrorKind {}
