use crate::{
    params::Language,
    properties::{
        ParameterError, SharedParams, param_name, param_segments, param_value,
    },
    values::Text,
};

/// This property defines the status code returned for a scheduling request.
///
/// Example:
///
/// > REQUEST-STATUS:2.0;Success
///
/// [Section 3.8.8.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.8.3)
#[derive(Debug)]
pub struct RequestStatus {
    value: Text,
    params: RequestStatusParams,
}

impl_try_from_bytes!(RequestStatus, Text, RequestStatusParams);

#[derive(Debug, Default)]
struct RequestStatusParams {
    shared: SharedParams,
    language: Option<Language>,
}

impl TryFrom<&[u8]> for RequestStatusParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"LANGUAGE" => {
                    params.language = Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_status_keeps_the_internal_semicolon_in_the_value() {
        // Regression test: REQUEST-STATUS's own value format is
        // "statcode;statdesc[;extdata]" — a naive first-';' split would
        // truncate it at "2.0".
        let rs = RequestStatus::try_from(b":2.0;Success".as_slice()).unwrap();
        assert_eq!(&*rs.value, "2.0;Success");
    }
}
