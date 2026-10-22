//! Field values as typed in an editor, converted to what map files store.

use wc3_data::meta::FieldMeta;
use wc3_data::objects::Value;

use crate::{Error, Result};

const TEXT_CATEGORY: &str = "text";

/// Text the game shows to players (names, tooltips), which maps keep in
/// their string table.
pub(crate) fn is_translatable(field: &FieldMeta) -> bool {
    field.category == TEXT_CATEGORY && field.value_type == "string"
}

/// Parses text as typed in an editor: numbers for numeric fields (bit flags
/// and numbered choices included), `true`/`false` or `1`/`0` for booleans,
/// raw codes for lists and choices.
pub(crate) fn parse_value(field: &FieldMeta, text: &str) -> Result<Value> {
    let text = text.trim();
    let invalid = || {
        Error::Invalid(format!(
            "{} expects a {}, got {text:?}",
            field.id, field.value_type
        ))
    };

    Ok(match field.value_type.as_str() {
        "bool" => Value::Int(match text.to_ascii_lowercase().as_str() {
            "1" | "true" => 1,
            "0" | "false" => 0,
            _ => return Err(invalid()),
        }),
        _ if field.is_integer() => Value::Int(text.parse().map_err(|_| invalid())?),
        "real" => Value::Real(text.parse().map_err(|_| invalid())?),
        "unreal" => {
            let v: f32 = text.parse().map_err(|_| invalid())?;
            if v < 0.0 {
                return Err(invalid());
            }

            Value::Unreal(v)
        }
        _ => Value::String(text.to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(value_type: &str) -> FieldMeta {
        FieldMeta {
            id: "test".to_owned(),
            field: "Test".to_owned(),
            source: "Profile".to_owned(),
            index: None,
            category: "stats".to_owned(),
            display_key: String::new(),
            sort: String::new(),
            value_type: value_type.to_owned(),
            min: None,
            max: None,
            can_be_empty: false,
            use_hero: true,
            use_unit: true,
            use_building: true,
            use_item: true,
            use_specific: Vec::new(),
            not_specific: Vec::new(),
            per_level: false,
            data_column: None,
            netsafe: false,
        }
    }

    #[test]
    fn numbered_choices_and_flags_are_stored_as_integers() {
        // The World Editor stores these as integers (seen in every map with them).
        for value_type in ["attackBits", "stackFlags", "versionFlags", "defenseTypeInt"] {
            assert_eq!(
                parse_value(&field(value_type), "3").ok(),
                Some(Value::Int(3))
            );
        }

        assert_eq!(
            parse_value(&field("bool"), "true").ok(),
            Some(Value::Int(1))
        );
        assert_eq!(
            parse_value(&field("targetList"), "air,ground").ok(),
            Some(Value::String("air,ground".to_owned()))
        );
        assert_eq!(
            parse_value(&field("unreal"), "0.5").ok(),
            Some(Value::Unreal(0.5))
        );
        assert!(parse_value(&field("unreal"), "-1").is_err());
        assert!(parse_value(&field("stackFlags"), "damage").is_err());
    }
}
