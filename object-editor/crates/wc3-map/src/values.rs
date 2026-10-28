use wc3_data::meta::FieldMeta;
use wc3_data::objects::Value;

use crate::{Error, Result};

const TEXT_CATEGORY: &str = "text";
/// Lists of object IDs. Their metadata maximum is how many the list holds
/// (5 hero abilities, 4 item abilities, 12 items sold), as the World Editor
/// enforces; on text lists it is a length instead.
const ID_LIST_TYPES: &[&str] = &[
    "abilityList",
    "heroAbilityList",
    "itemList",
    "unitList",
    "upgradeList",
];

/// Text the game shows to players (names, tooltips), which maps keep in
/// their string table.
pub(crate) fn is_translatable(field: &FieldMeta) -> bool {
    field.category == TEXT_CATEGORY && field.value_type == "string"
}

pub(crate) fn parse_value(field: &FieldMeta, text: &str) -> Result<Value> {
    let text = text.trim();

    if let Some(max) = list_limit(field) {
        let count = text
            .split(',')
            .filter(|code| !code.trim().is_empty())
            .count();

        if count > max {
            return Err(Error::Invalid(format!(
                "{} holds at most {max}, got {count}",
                field.id
            )));
        }
    }

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

fn list_limit(field: &FieldMeta) -> Option<usize> {
    if !ID_LIST_TYPES.contains(&field.value_type.as_str()) {
        return None;
    }

    field.max.as_deref()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(value_type: &str) -> FieldMeta {
        FieldMeta {
            id: "test".to_owned(),
            field: "Test".to_owned(),
            section: String::new(),
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

    #[test]
    fn id_lists_hold_at_most_the_metadata_maximum() {
        let mut heroes = field("heroAbilityList");

        heroes.max = Some("5".to_owned());
        assert!(parse_value(&heroes, "AHhb,AHds,AHre,AHad,AHtb").is_ok());
        let err = parse_value(&heroes, "AHhb,AHds,AHre,AHad,AHtb,AHbz").unwrap_err();

        assert!(err.to_string().contains("at most 5"), "{err}");

        // On text lists the maximum is a length, not a count.
        let mut names = field("stringList");

        names.max = Some("1".to_owned());
        assert!(parse_value(&names, "Arthas,Uther").is_ok());
    }
}
