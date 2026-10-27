use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

const RECIPES: &str = include_str!("../data/spell-recipes.toml");
const RECIPES_FILE: &str = "spell-recipes.toml";
const TEMPLATES: &str = include_str!("../data/spell-templates.toml");
const TEMPLATES_FILE: &str = "spell-templates.toml";
const TEMPLATE_SOURCES: &[(&str, &str)] = &[
    (
        "attribute_damage.lua",
        include_str!("../data/spell-templates/attribute_damage.lua"),
    ),
    (
        "knockback_wave.lua",
        include_str!("../data/spell-templates/knockback_wave.lua"),
    ),
    (
        "bounce_damage.lua",
        include_str!("../data/spell-templates/bounce_damage.lua"),
    ),
];

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellKind {
    pub id: String,
    pub label: String,
    pub description: String,
    #[serde(rename = "base")]
    pub bases: Vec<SuggestedBase>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestedBase {
    pub code: String,
    pub why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    kind: Vec<SpellKind>,
}

pub fn spell_kinds() -> Result<Vec<SpellKind>> {
    parse(RECIPES).map_err(|e| e.in_file(RECIPES_FILE))
}

fn parse(text: &str) -> Result<Vec<SpellKind>> {
    let file: RecipeFile = toml::from_str(text).map_err(|e| Error::Parse {
        line: 0,
        reason: e.to_string(),
    })?;

    Ok(file.kind)
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpellTemplate {
    pub id: String,
    pub label: String,
    pub description: String,
    pub file: String,
    pub target: String,
    /// Targets allowed, as the editor stores them; empty for none.
    pub targets: String,
    pub icon: String,
    pub hotkey: String,
    pub range: f64,
    pub area: f64,
    pub cooldown: f64,
    pub mana: f64,
    pub tooltip: String,
    #[serde(rename(deserialize = "param"))]
    pub params: Vec<TemplateParam>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateParam {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub values: Vec<f64>,
    #[serde(default)]
    pub whole: bool,
    #[serde(default)]
    pub choices: Vec<TemplateChoice>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateChoice {
    pub label: String,
    pub lua: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateFile {
    template: Vec<SpellTemplate>,
}

pub fn spell_templates() -> Result<Vec<SpellTemplate>> {
    let file: TemplateFile = toml::from_str(TEMPLATES).map_err(|e| {
        Error::Parse {
            line: 0,
            reason: e.to_string(),
        }
        .in_file(TEMPLATES_FILE)
    })?;

    Ok(file.template)
}

fn lua_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

pub fn fill_tooltip(
    template: &SpellTemplate,
    spell_id: &str,
    level: u32,
    values: &HashMap<String, Vec<f64>>,
) -> String {
    let mut text = template
        .tooltip
        .replace("{area}", &format!("<{spell_id},Area{level}>"));

    for param in &template.params {
        let given = values.get(&param.id);
        let shown = if param.choices.is_empty() {
            let numbers = given.unwrap_or(&param.values);
            let index = (level as usize).clamp(1, numbers.len().max(1)) - 1;
            numbers
                .get(index)
                .map(|v| lua_number(*v))
                .unwrap_or_default()
        } else {
            let index = given.and_then(|v| v.first()).copied().unwrap_or(0.0) as usize;
            param
                .choices
                .get(index)
                .map(|c| c.label.clone())
                .unwrap_or_default()
        };

        text = text.replace(&format!("{{{}}}", param.id), &shown);
    }

    text
}

pub fn render_template(
    template: &SpellTemplate,
    spell_id: &str,
    spell_name: &str,
    values: &HashMap<String, Vec<f64>>,
) -> Result<String> {
    let invalid = |reason: String| Error::Parse { line: 0, reason }.in_file(&template.file);

    if spell_id.len() != 4 || !spell_id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(invalid(format!("{spell_id:?} is not an ability ID")));
    }

    let source = TEMPLATE_SOURCES
        .iter()
        .find(|(name, _)| *name == template.file)
        .map(|(_, source)| *source)
        .ok_or_else(|| invalid("no such template file".to_owned()))?;
    let name: String = spell_name
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();

    let mut text = source
        .replace("{{SPELL_ID}}", spell_id)
        .replace("{{SPELL_NAME}}", &name)
        .replace("{{TEMPLATE}}", &template.label);

    for param in &template.params {
        let given = values.get(&param.id);
        let lua = if param.choices.is_empty() {
            let numbers = given.unwrap_or(&param.values);
            if numbers.is_empty() || numbers.iter().any(|v| !v.is_finite()) {
                return Err(invalid(format!("{} needs a number per level", param.label)));
            }
            if param.whole && numbers.iter().any(|v| v.fract() != 0.0) {
                return Err(invalid(format!("{} takes whole numbers", param.label)));
            }

            let list: Vec<String> = numbers.iter().map(|v| lua_number(*v)).collect();
            format!("{{ {} }}", list.join(", "))
        } else {
            let index = given.and_then(|v| v.first()).copied().unwrap_or(0.0);
            param
                .choices
                .get(index as usize)
                .filter(|_| index >= 0.0 && index.fract() == 0.0)
                .ok_or_else(|| invalid(format!("no choice {index} for {}", param.label)))?
                .lua
                .clone()
        };

        text = text.replace(&format!("{{{{{}}}}}", param.id), &lua);
    }

    if let Some(at) = text.find("{{") {
        let rest = &text[at..];
        let end = rest.find("}}").map_or(rest.len(), |e| e + 2);

        return Err(invalid(format!("unknown placeholder {}", &rest[..end])));
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_label_and_suggested_bases() {
        let kinds = spell_kinds().unwrap();

        assert!(kinds.len() >= 10, "only {} kinds", kinds.len());

        let mut ids: Vec<&str> = kinds.iter().map(|k| k.id.as_str()).collect();

        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), kinds.len(), "kind IDs repeat");

        for kind in &kinds {
            assert!(
                !kind.label.is_empty() && !kind.description.is_empty(),
                "{}",
                kind.id
            );
            assert!(!kind.bases.is_empty(), "{} suggests no base", kind.id);
            for base in &kind.bases {
                assert_eq!(base.code.len(), 4, "{}: {}", kind.id, base.code);
                assert!(!base.why.is_empty(), "{}: {}", kind.id, base.code);
            }
        }
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(
            parse("[[kind]]\nid = \"x\"\nlabel = \"X\"\ndescription = \"x\"\nbase = []\nextra = 1")
                .is_err()
        );
    }

    #[test]
    fn templates_fill_in_ids_names_and_values() {
        let templates = spell_templates().unwrap();

        assert!(templates.len() >= 3);
        for template in &templates {
            let text = render_template(template, "A00K", "Test\nName", &HashMap::new()).unwrap();

            assert!(text.contains("FourCC(\"A00K\")"), "{}", template.id);
            assert!(!text.contains("{{"), "{}", template.id);
        }

        let wave = templates.iter().find(|t| t.id == "knockback-wave").unwrap();
        let values = HashMap::from([("damage".to_owned(), vec![10.0, 20.5])]);
        let text = render_template(wave, "A001", "Wave", &values).unwrap();

        assert!(text.contains("local DAMAGE = { 10, 20.5 }"), "{text}");
    }

    #[test]
    fn templates_reject_bad_ids_and_values() {
        let templates = spell_templates().unwrap();
        let arc = templates.iter().find(|t| t.id == "bounce-damage").unwrap();
        let none = HashMap::new();

        assert!(render_template(arc, "A0\"1", "x", &none).is_err());

        let half_bounce = HashMap::from([("bounces".to_owned(), vec![1.5])]);

        assert!(render_template(arc, "A001", "x", &half_bounce).is_err());

        let strike = templates
            .iter()
            .find(|t| t.id == "attribute-damage")
            .unwrap();
        let fourth = HashMap::from([("attribute".to_owned(), vec![3.0])]);

        assert!(render_template(strike, "A001", "x", &fourth).is_err());
    }
}
