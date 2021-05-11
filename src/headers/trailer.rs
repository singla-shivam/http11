use crate::errors::Error as HttpErrors;
use crate::headers::{GeneralHeader, Header, Headers, TRAILER_HEADER_NAME};
use std::any::Any;
use std::convert::TryFrom;

pub struct Trailer {
    fields: Vec<String>,
}

impl Header for Trailer {
    fn name(&self) -> &str {
        TRAILER_HEADER_NAME
    }

    fn value(&self) -> String {
        self.fields.join(", ")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GeneralHeader for Trailer {}

impl TryFrom<&str> for Trailer {
    type Error = HttpErrors;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let field_names = value.split(",");
        let field_names: Vec<&str> =
            field_names.map(|f| f.trim()).filter(|f| f != &"").collect();

        for f in field_names.iter() {
            if !Headers::is_valid_header_name(f) {
                return Err(HttpErrors::InvalidHeaderField(f.to_string()));
            }
        }

        let field_names: Vec<String> =
            field_names.into_iter().map(|f| f.into()).collect();

        let trailer = Trailer {
            fields: field_names,
        };

        Ok(trailer)
    }
}
