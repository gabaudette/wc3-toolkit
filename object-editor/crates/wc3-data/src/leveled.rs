//! What the map code needs from a game database whose fields can have a value
//! per level: upgrades (per research level) and doodads (per model variation).

use crate::game::ArtMode;
use crate::meta::FieldMeta;

pub trait LeveledData {
    /// Every field the catalog shows; all apply to every object.
    fn fields(&self) -> &[FieldMeta];

    fn field(&self, field_id: &str) -> Option<&FieldMeta>;

    /// A field's display name, e.g. `Gold Base`.
    fn field_name(&self, field: &FieldMeta) -> String;

    /// The editor's name for a metadata category, e.g. `art` → `Art`.
    fn category_label(&self, category: &str) -> String;

    /// Every object ID in the game data, in table order.
    fn ids(&self) -> &[String];

    fn contains(&self, id: &str) -> bool;

    /// The game's raw value of a field at `level` (1-based; ignored for fields
    /// without levels), or `None` when the object leaves it empty.
    fn raw_value(&self, id: &str, field: &FieldMeta, level: u32, mode: ArtMode) -> Option<String>;
}
