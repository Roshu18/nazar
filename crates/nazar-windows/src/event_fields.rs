use ferrisetw::parser::{Parser, ParserError, Pointer};
use ferrisetw::{EventRecord, SchemaLocator};

use crate::trace_signal::DecodeError;

pub(crate) struct EventFields<'schema, 'record> {
    parser: Parser<'schema, 'record>,
}

pub(crate) fn decode_with<Decoded>(
    record: &EventRecord,
    schema_locator: &SchemaLocator,
    decode: impl FnOnce(&EventFields<'_, '_>) -> Result<Decoded, DecodeError>,
) -> Result<Decoded, DecodeError> {
    let schema = schema_locator
        .event_schema(record)
        .map_err(|error| DecodeError::SchemaUnavailable(format!("{error:?}")))?;
    let fields = EventFields {
        parser: Parser::create(record, &schema),
    };
    decode(&fields)
}

impl EventFields<'_, '_> {
    pub(crate) fn text(&self, field: &'static str) -> Result<String, DecodeError> {
        self.parser
            .try_parse::<String>(field)
            .map_err(|error| unreadable(field, &error))
    }

    pub(crate) fn number(&self, field: &'static str) -> Result<u32, DecodeError> {
        self.parser
            .try_parse::<u32>(field)
            .map_err(|error| unreadable(field, &error))
    }

    pub(crate) fn small_number(&self, field: &'static str) -> Result<u16, DecodeError> {
        self.parser
            .try_parse::<u16>(field)
            .map_err(|error| unreadable(field, &error))
    }

    pub(crate) fn address(&self, field: &'static str) -> Result<usize, DecodeError> {
        self.parser
            .try_parse::<Pointer>(field)
            .map(|pointer| *pointer)
            .map_err(|error| unreadable(field, &error))
    }

    pub(crate) fn bytes(&self, field: &'static str) -> Result<Vec<u8>, DecodeError> {
        self.parser
            .try_parse::<Vec<u8>>(field)
            .map_err(|error| unreadable(field, &error))
    }
}

fn unreadable(field: &'static str, error: &ParserError) -> DecodeError {
    DecodeError::FieldUnreadable {
        field,
        reason: error.to_string(),
    }
}
