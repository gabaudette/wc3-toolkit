//! What the map code needs from a game database whose fields can have a value
//! per level: upgrades (per research level) and doodads (per model variation).

use crate::game::ArtMode;
use crate::meta::FieldMeta;

pub trait LeveledData {
    fn fields(&self) -> &[FieldMeta];

    fn field(&self, field_id: &str) -> Option<&FieldMeta>;

    fn field_name(&self, field: &FieldMeta) -> String;

    fn category_label(&self, category: &str) -> String;

    fn ids(&self) -> &[String];

    fn contains(&self, id: &str) -> bool;

    fn raw_value(&self, id: &str, field: &FieldMeta, level: u32, mode: ArtMode) -> Option<String>;
}
