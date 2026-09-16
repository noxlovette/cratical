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

/// Builder for [`RequestStatus`].
#[derive(Debug)]
pub struct RequestStatusBuilder {
    value: Text,
    language: Option<Language>,
}

impl RequestStatusBuilder {
    /// Starts a new builder from the property's required value
    /// (`statcode;statdesc[;extdata]`, per RFC 5545 §3.8.8.3).
    pub fn new(value: Text) -> Self {
        Self {
            value,
            language: None,
        }
    }

    /// Sets the `LANGUAGE` parameter.
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Finishes the builder, producing a [`RequestStatus`].
    pub fn build(self) -> RequestStatus {
        RequestStatus {
            value: self.value,
            params: RequestStatusParams {
                shared: SharedParams::default(),
                language: self.language,
            },
        }
    }
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "REQUEST-STATUS{}:{}", self.params, self.value.as_str())
    }
}

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
                    params.language =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for RequestStatusParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.language {
            write!(f, ";LANGUAGE={v}")?;
        }
        write!(f, "{}", self.shared)
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

    #[test]
    fn request_status_builder_round_trips() {
        let rs = RequestStatusBuilder::new("2.0;Success".into()).build();
        assert_eq!(rs.to_string(), "REQUEST-STATUS:2.0;Success");
    }
}
