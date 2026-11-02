//! The parts of the map script the World Editor writes from the map's data:
//! placed units (`CreateAllUnits` and the functions it calls) and the item
//! drops of placed units and item tables. The game creates placed units from
//! these functions, not from `war3mapUnits.doo`, so changed placements only
//! take effect once these functions are written again. Everything else in the
//! script (triggers, custom code) is kept as it is.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::info::ItemTable;
use crate::placements::{DropChance, Placement, UnitPlacements};
use crate::{Error, FourCC, Result};

pub const LUA_FILE: &str = "war3map.lua";
pub const JASS_FILES: &[&str] = &["war3map.j", "scripts\\war3map.j"];

const CREATE_ALL: &str = "CreateAllUnits";
const NEUTRAL_HOSTILE: u32 = 24;
const NEUTRAL_PASSIVE: u32 = 27;
const CAMP_ACQUISITION: f32 = -2.0;
const NORMAL_ACQUISITION: f32 = -1.0;
const CAMP_RANGE: f32 = 200.0;
const DEFAULT_STAT: i32 = -1;
const PERCENT: f32 = 100.0;
const NO_ITEM: i32 = -1;
const BLIGHTED_MINE: FourCC = FourCC(*b"ugol");
const RANDOM_PREFIX: u8 = b'Y';
const RANDOM_MARK: u8 = b'I';
const ANY_CLASS: u8 = b'Y';
// Random item codes `Y<class>I<level>`: classes in JASS ITEM_TYPE order.
const ITEM_TYPES: &[(u8, &str)] = &[
    (b'i', "ITEM_TYPE_PERMANENT"),
    (b'j', "ITEM_TYPE_CHARGED"),
    (b'k', "ITEM_TYPE_POWERUP"),
    (b'l', "ITEM_TYPE_ARTIFACT"),
    (b'm', "ITEM_TYPE_PURCHASABLE"),
    (b'n', "ITEM_TYPE_CAMPAIGN"),
    (b'o', "ITEM_TYPE_MISCELLANEOUS"),
    (ANY_CLASS, "ITEM_TYPE_ANY"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Lua,
    Jass,
}

/// What the generator needs to know about unit types, from the game data.
pub trait UnitKinds {
    fn is_building(&self, id: FourCC) -> bool;
    fn is_hero(&self, id: FourCC) -> bool;
    fn is_item(&self, id: FourCC) -> bool;
    /// Whether the unit holds gold (its gold amount is written to the script).
    fn is_gold_mine(&self, id: FourCC) -> bool;
    /// The unit type's own team color (Art - Team Color), when it has one.
    fn team_color(&self, id: FourCC) -> Option<i32>;
    /// The orders turning an ability's autocast on and off (`healon`,
    /// `healoff`), if it has autocast.
    fn autocast_orders(&self, ability: FourCC) -> Option<(String, String)>;
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Style {
    pub(crate) language: Language,
    pub(crate) indent: String,
    filtered_items: bool,
    // Whether the locals of drop and creation functions are followed by a
    // blank line: World Editor builds differ, so it is read from the script.
    drop_gap: bool,
    create_gap: bool,
}

impl Style {
    pub(crate) fn detect(language: Language, text: &str) -> Self {
        let generated_local = |line: &&str| {
            let trimmed = line.trim_start();

            trimmed.starts_with("local p = ")
                || trimmed.starts_with("local trigWidget")
                || trimmed.starts_with("local player p = ")
                || trimmed.starts_with("local widget  trigWidget")
        };
        let indented = text
            .lines()
            .find(generated_local)
            .or_else(|| {
                text.lines().find(|line| {
                    line.trim_start().starts_with("local ") || line.trim_start().starts_with("set ")
                })
            })
            .is_some_and(|line| line.starts_with("    "));
        let indent = match (language, indented) {
            (Language::Jass, _) | (Language::Lua, true) => "    ".to_owned(),
            (Language::Lua, false) => String::new(),
        };

        let gap_after = |marker: &str| {
            let lines: Vec<&str> = text.lines().collect();

            lines
                .iter()
                .position(|l| l.trim_start().starts_with(marker))
                .map(|at| {
                    let after = lines[at..]
                        .iter()
                        .position(|l| !l.trim_start().starts_with("local "))
                        .map_or(at, |n| at + n);

                    lines.get(after).is_some_and(|l| l.trim().is_empty())
                })
        };

        Self {
            language,
            indent,
            filtered_items: text.contains("ChooseRandomItemExWithFilter"),
            drop_gap: gap_after("local trigWidget")
                .or_else(|| gap_after("local widget"))
                .unwrap_or(true),
            create_gap: gap_after("local p = ")
                .or_else(|| gap_after("local player p = "))
                .unwrap_or(true),
        }
    }

    pub(crate) fn id(&self, id: FourCC) -> String {
        match self.language {
            Language::Lua => format!("FourCC(\"{id}\")"),
            Language::Jass => format!("'{id}'"),
        }
    }

    // JASS calls are written `call F( a, b )`, with two spaces for no
    // arguments; Lua calls `F(a, b)`.
    pub(crate) fn call(&self, function: &str, args: &[String]) -> String {
        match self.language {
            Language::Lua => format!("{function}({})", args.join(", ")),
            Language::Jass => format!("call {function}( {} )", args.join(", ")),
        }
    }

    pub(crate) fn expression(&self, function: &str, args: &[String]) -> String {
        match self.language {
            Language::Lua => format!("{function}({})", args.join(", ")),
            Language::Jass => format!("{function}( {} )", args.join(", ")),
        }
    }

    pub(crate) fn assign(&self, target: &str, value: &str) -> String {
        match self.language {
            Language::Lua => format!("{target} = {value}"),
            Language::Jass => format!("set {target} = {value}"),
        }
    }

    pub(crate) fn function(&self, name: &str, body: &[String]) -> String {
        let mut out = match self.language {
            Language::Lua => format!("function {name}()\n"),
            Language::Jass => format!("function {name} takes nothing returns nothing\n"),
        };

        for line in body {
            if line.is_empty() {
                out.push('\n');
                continue;
            }

            out.push_str(&self.indent);
            out.push_str(line);
            out.push('\n');
        }

        match self.language {
            Language::Lua => out.push_str("end\n"),
            Language::Jass => out.push_str("endfunction\n"),
        }

        out
    }
}

pub fn language_of_file(name: &str) -> Option<Language> {
    if name.eq_ignore_ascii_case(LUA_FILE) {
        return Some(Language::Lua);
    }

    JASS_FILES
        .iter()
        .any(|f| f.eq_ignore_ascii_case(name))
        .then_some(Language::Jass)
}

pub(crate) fn functions(text: &str, language: Language) -> Vec<Function> {
    if language == Language::Lua
        && let Some(found) = lua_functions(text)
    {
        return found;
    }

    line_functions(text, language)
}

// Lua scripts from current World Editors do not indent nested blocks, so a
// function's end is only known by parsing.
fn lua_functions(text: &str) -> Option<Vec<Function>> {
    use full_moon::ast::Stmt;
    use full_moon::node::Node;

    let ast = full_moon::parse_fallible(text, full_moon::LuaVersion::lua53())
        .into_result()
        .ok()?;
    let found = ast
        .nodes()
        .stmts()
        .filter_map(|stmt| {
            let Stmt::FunctionDeclaration(declaration) = stmt else {
                return None;
            };
            let (start, end) = declaration.range()?;
            let end = end.bytes();
            let line_end = text[end..].find('\n').map_or(text.len(), |i| end + i + 1);

            Some(Function {
                name: declaration.name().to_string().trim().to_owned(),
                start: start.bytes(),
                end: line_end,
            })
        })
        .collect();

    Some(found)
}

fn line_functions(text: &str, language: Language) -> Vec<Function> {
    let (open, close) = match language {
        Language::Lua => ("function ", "end"),
        Language::Jass => ("function ", "endfunction"),
    };
    let mut found = Vec::new();
    let mut current: Option<(String, usize)> = None;
    let mut offset = 0;

    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();

        if current.is_none() && trimmed.starts_with(open) {
            let name: String = trimmed[open.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();

            current = Some((name, offset));
        } else if trimmed == close
            && let Some((name, start)) = current.take()
        {
            found.push(Function {
                name,
                start,
                end: offset + line.len(),
            });
        }

        offset += line.len();
    }

    found
}

pub fn is_unit_function(name: &str) -> bool {
    const FIXED: &[&str] = &[
        CREATE_ALL,
        "CreatePlayerBuildings",
        "CreatePlayerUnits",
        "CreateNeutralHostile",
        "CreateNeutralHostileBuildings",
        "CreateNeutralPassive",
        "CreateNeutralPassiveBuildings",
    ];
    let numbered = |prefix: &str, suffix: &str| {
        name.strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(suffix))
            .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
    };

    FIXED.contains(&name)
        || numbered("CreateUnitsForPlayer", "")
        || numbered("CreateBuildingsForPlayer", "")
        || numbered("Unit", "_DropItems")
        || numbered("ItemTable", "_DropItems")
}

fn number(value: f32, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

fn degrees(radians: f32) -> f32 {
    radians.to_degrees()
}

fn drop_item(style: &Style, item: FourCC) -> String {
    let [prefix, class, mark, digit] = item.0;

    if item.0 == [0; 4] {
        return NO_ITEM.to_string();
    }

    if prefix == RANDOM_PREFIX
        && mark == RANDOM_MARK
        && digit.is_ascii_digit()
        && let Some((_, item_type)) = ITEM_TYPES.iter().find(|(letter, _)| *letter == class)
    {
        let level = char::from(digit).to_string();

        if style.filtered_items {
            return style.expression(
                "ChooseRandomItemExWithFilter",
                &[
                    (*item_type).to_owned(),
                    level,
                    "EQUIPMENT_TYPE_ANY".to_owned(),
                    "ITEMTAG_TYPE_ANY".to_owned(),
                ],
            );
        }

        return style.expression("ChooseRandomItemEx", &[(*item_type).to_owned(), level]);
    }

    style.id(item)
}

fn drop_function(style: &Style, name: &str, sets: &[Vec<DropChance>]) -> String {
    let lua = style.language == Language::Lua;
    let mut body: Vec<String> = Vec::new();
    let (null, ne, end_if, comment) = match style.language {
        Language::Lua => ("nil", "~=", "end", false),
        Language::Jass => ("null", "!=", "endif", true),
    };
    let inner = |line: &str| format!("{}{line}", style.indent);

    if lua {
        body.extend([
            "local trigWidget = nil".to_owned(),
            "local trigUnit = nil".to_owned(),
            "local itemID = 0".to_owned(),
            "local canDrop = true".to_owned(),
        ]);
    } else {
        body.extend([
            "local widget  trigWidget = null".to_owned(),
            "local unit    trigUnit   = null".to_owned(),
            "local integer itemID     = 0".to_owned(),
            "local boolean canDrop    = true".to_owned(),
        ]);
    }

    if style.drop_gap {
        body.push(String::new());
    }

    body.push(style.assign("trigWidget", "bj_lastDyingWidget"));
    body.push(format!("if (trigWidget == {null}) then"));
    body.push(inner(&style.assign("trigUnit", "GetTriggerUnit()")));
    body.push(end_if.to_owned());
    if !lua {
        body.push(String::new());
    }

    body.push(format!("if (trigUnit {ne} {null}) then"));
    body.push(inner(
        &style.assign("canDrop", "not IsUnitHidden(trigUnit)"),
    ));
    body.push(inner(&format!(
        "if (canDrop and GetChangingUnit() {ne} {null}) then"
    )));
    body.push(inner(&inner(&style.assign(
        "canDrop",
        "(GetChangingUnitPrevOwner() == Player(PLAYER_NEUTRAL_AGGRESSIVE))",
    ))));
    body.push(inner(end_if));
    body.push(end_if.to_owned());
    if !lua {
        body.push(String::new());
    }

    body.push("if (canDrop) then".to_owned());
    for (index, set) in sets.iter().enumerate() {
        if comment {
            body.push(inner(&format!("// Item set {index}")));
        }

        body.push(inner(&style.call("RandomDistReset", &[String::new()])));
        for drop in set {
            body.push(inner(&style.call(
                "RandomDistAddItem",
                &[drop_item(style, drop.item), drop.chance.to_string()],
            )));
        }

        body.push(inner(&style.assign(
            "itemID",
            &style.expression("RandomDistChoose", &[String::new()]),
        )));
        body.push(inner(&format!("if (trigUnit {ne} {null}) then")));
        body.push(inner(&inner(&style.call(
            "UnitDropItem",
            &["trigUnit".to_owned(), "itemID".to_owned()],
        ))));
        body.push(inner("else"));
        body.push(inner(&inner(&style.call(
            "WidgetDropItem",
            &["trigWidget".to_owned(), "itemID".to_owned()],
        ))));
        body.push(inner(end_if));
        if !lua {
            body.push(String::new());
        }
    }

    body.push(end_if.to_owned());
    if !lua {
        body.push(String::new());
    }

    body.push(style.assign("bj_lastDyingWidget", null));
    body.push(style.call_raw("DestroyTrigger(GetTriggeringTrigger())"));

    style.function(name, &body)
}

impl Style {
    // `DestroyTrigger(GetTriggeringTrigger())` keeps Lua's spacing in JASS.
    fn call_raw(&self, text: &str) -> String {
        match self.language {
            Language::Lua => text.to_owned(),
            Language::Jass => format!("call {text}"),
        }
    }
}

// Item sets with no items drop nothing, and the World Editor leaves them out.
fn drop_sets(unit: &Placement) -> Vec<Vec<DropChance>> {
    unit.item_sets()
        .iter()
        .filter(|set| !set.is_empty())
        .cloned()
        .collect()
}

pub fn unit_drop_name(index: usize) -> String {
    format!("Unit{index:06}_DropItems")
}

pub fn table_drop_name(number: u32) -> String {
    format!("ItemTable{number:06}_DropItems")
}

/// The statements creating one placed unit, without indentation.
fn unit_block(
    style: &Style,
    unit: &Placement,
    index: usize,
    target: &str,
    kinds: &dyn UnitKinds,
    waygate: &[String],
) -> Vec<String> {
    let mut lines = Vec::new();
    let skin = unit.skin.unwrap_or(unit.id);
    let x = number(unit.x, 1);
    let y = number(unit.y, 1);
    let facing = number(degrees(unit.angle), 3);
    let create = if unit.id == BLIGHTED_MINE {
        style.expression("CreateBlightedGoldmine", &["p".to_owned(), x, y, facing])
    } else {
        style.expression(
            "BlzCreateUnitWithSkin",
            &[
                "p".to_owned(),
                style.id(unit.id),
                x,
                y,
                facing,
                style.id(skin),
            ],
        )
    };

    lines.push(style.assign(target, &create));
    if kinds.is_gold_mine(unit.id) || unit.id == BLIGHTED_MINE {
        lines.push(style.call(
            "SetResourceAmount",
            &[target.to_owned(), unit.gold.to_string()],
        ));
    }

    lines.extend(waygate.iter().cloned());
    // A placement without its own color still gets its unit type's.
    let color = Some(unit.color)
        .filter(|c| *c != DEFAULT_STAT)
        .or_else(|| kinds.team_color(unit.id));

    if let Some(color) = color {
        lines.push(style.call(
            "SetUnitColor",
            &[target.to_owned(), format!("ConvertPlayerColor({color})")],
        ));
    }

    if kinds.is_hero(unit.id) {
        if unit.level > 1 {
            lines.push(style.call(
                "SetHeroLevel",
                &[
                    target.to_owned(),
                    unit.level.to_string(),
                    "false".to_owned(),
                ],
            ));
        }

        let [strength, agility, intelligence] = unit.attributes.unwrap_or_default();

        for (function, value) in [
            ("SetHeroStr", strength),
            ("SetHeroAgi", agility),
            ("SetHeroInt", intelligence),
        ] {
            if value > 0 {
                lines.push(style.call(
                    function,
                    &[target.to_owned(), value.to_string(), "true".to_owned()],
                ));
            }
        }
    }

    if unit.life != DEFAULT_STAT {
        lines.push(style.assign(
            "life",
            &style.expression(
                "GetUnitState",
                &[target.to_owned(), "UNIT_STATE_LIFE".to_owned()],
            ),
        ));
        lines.push(style.call(
            "SetUnitState",
            &[
                target.to_owned(),
                "UNIT_STATE_LIFE".to_owned(),
                format!("{} * life", number(unit.life as f32 / PERCENT, 2)),
            ],
        ));
    }

    if unit.mana != DEFAULT_STAT {
        lines.push(style.call(
            "SetUnitState",
            &[
                target.to_owned(),
                "UNIT_STATE_MANA".to_owned(),
                unit.mana.to_string(),
            ],
        ));
    }

    if unit.acquisition != NORMAL_ACQUISITION {
        let range = if unit.acquisition == CAMP_ACQUISITION {
            CAMP_RANGE
        } else {
            unit.acquisition
        };

        lines.push(style.call(
            "SetUnitAcquireRange",
            &[target.to_owned(), number(range, 1)],
        ));
    }

    for ability in &unit.abilities {
        if kinds.is_hero(unit.id) {
            for _ in 0..ability.level {
                lines.push(style.call(
                    "SelectHeroSkill",
                    &[target.to_owned(), style.id(ability.ability)],
                ));
            }
        }

        if let Some((on, off)) = kinds.autocast_orders(ability.ability) {
            let order = if ability.autocast { on } else { off };

            lines.push(style.call(
                "IssueImmediateOrder",
                &[target.to_owned(), format!("\"{order}\"")],
            ));
        }
    }

    for slot in unit.inventory() {
        lines.push(style.call(
            "UnitAddItemToSlotById",
            &[
                target.to_owned(),
                style.id(slot.item),
                slot.slot.to_string(),
            ],
        ));
    }

    let action = match (unit.item_table(), drop_sets(unit).is_empty()) {
        (Some(table), _) => u32::try_from(table).ok().map(table_drop_name),
        (None, false) => Some(unit_drop_name(index)),
        (None, true) => None,
    };

    if let Some(action) = action {
        let reference = match style.language {
            Language::Lua => action,
            Language::Jass => format!("function {action}"),
        };

        lines.push(style.assign("t", &style.expression("CreateTrigger", &[String::new()])));
        for event in ["EVENT_UNIT_DEATH", "EVENT_UNIT_CHANGE_OWNER"] {
            lines.push(style.call(
                "TriggerRegisterUnitEvent",
                &["t".to_owned(), target.to_owned(), event.to_owned()],
            ));
        }

        lines.push(style.call("TriggerAddAction", &["t".to_owned(), reference]));
    }

    lines
}

/// A creation function's group and its units, with their placement indexes.
type GroupUnits<'a> = (Group, Vec<(usize, &'a Placement)>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Group {
    Buildings(u32),
    Units(u32),
    HostileBuildings,
    Hostile,
    PassiveBuildings,
    Passive,
}

impl Group {
    // Players' functions in player order, buildings first; then the neutrals.
    fn order(&self) -> (u32, u32, u32) {
        match self {
            Self::Buildings(p) => (0, *p, 0),
            Self::Units(p) => (0, *p, 1),
            Self::HostileBuildings => (1, 0, 0),
            Self::Hostile => (1, 0, 1),
            Self::PassiveBuildings => (1, 0, 2),
            Self::Passive => (1, 0, 3),
        }
    }

    fn of(unit: &Placement, kinds: &dyn UnitKinds) -> Self {
        let building = kinds.is_building(unit.id);

        match (unit.player, building) {
            (NEUTRAL_HOSTILE, true) => Self::HostileBuildings,
            (NEUTRAL_HOSTILE, false) => Self::Hostile,
            (NEUTRAL_PASSIVE, true) => Self::PassiveBuildings,
            (NEUTRAL_PASSIVE, false) => Self::Passive,
            (player, true) => Self::Buildings(player),
            (player, false) => Self::Units(player),
        }
    }

    fn name(&self) -> String {
        match self {
            Self::Buildings(p) => format!("CreateBuildingsForPlayer{p}"),
            Self::Units(p) => format!("CreateUnitsForPlayer{p}"),
            Self::HostileBuildings => "CreateNeutralHostileBuildings".to_owned(),
            Self::Hostile => "CreateNeutralHostile".to_owned(),
            Self::PassiveBuildings => "CreateNeutralPassiveBuildings".to_owned(),
            Self::Passive => "CreateNeutralPassive".to_owned(),
        }
    }

    fn player(&self) -> String {
        match self {
            Self::Buildings(p) | Self::Units(p) => format!("Player({p})"),
            Self::HostileBuildings | Self::Hostile => {
                "Player(PLAYER_NEUTRAL_AGGRESSIVE)".to_owned()
            }
            Self::PassiveBuildings | Self::Passive => "Player(PLAYER_NEUTRAL_PASSIVE)".to_owned(),
        }
    }
}

fn locals(style: &Style, group: &Group) -> Vec<String> {
    match style.language {
        Language::Lua => vec![
            format!("local p = {}", group.player()),
            "local u".to_owned(),
            "local unitID".to_owned(),
            "local t".to_owned(),
            "local life".to_owned(),
        ],
        Language::Jass => vec![
            format!("local player p = {}", group.player()),
            "local unit u".to_owned(),
            "local integer unitID".to_owned(),
            "local trigger t".to_owned(),
            "local real life".to_owned(),
        ],
    }
}

/// Splits an existing creation function into the statements of each unit,
/// keyed by the unit's creation line (`id`, x, y as written).
fn original_blocks(text: &str, function: &Function) -> Vec<(String, Vec<String>)> {
    let body = &text[function.start..function.end];
    let mut blocks: Vec<(String, Vec<String>)> = Vec::new();

    for line in body.lines().skip(1) {
        let trimmed = line.trim();

        if trimmed.contains("BlzCreateUnitWithSkin(") || trimmed.contains("CreateBlightedGoldmine(")
        {
            blocks.push((creation_key(trimmed), vec![trimmed.to_owned()]));
        } else if let Some((_, block)) = blocks.last_mut()
            && !trimmed.is_empty()
            && trimmed != "end"
            && trimmed != "endfunction"
        {
            block.push(trimmed.to_owned());
        }
    }

    blocks
}

fn creation_key(line: &str) -> String {
    let inside = line
        .split_once('(')
        .map(|(_, rest)| rest)
        .unwrap_or_default();
    let parts: Vec<&str> = inside.split(',').map(str::trim).collect();

    parts
        .iter()
        .skip(1)
        .take(3)
        .copied()
        .collect::<Vec<_>>()
        .join(",")
}

fn placement_key(style: &Style, unit: &Placement) -> String {
    if unit.id == BLIGHTED_MINE {
        return format!("{},{}", number(unit.x, 1), number(unit.y, 1));
    }

    format!(
        "{},{},{}",
        style.id(unit.id),
        number(unit.x, 1),
        number(unit.y, 1)
    )
}

/// Script text for everything the World Editor generates from placed units:
/// what the functions should be, given the placements and the script they go into.
pub struct UnitSection<'a> {
    pub placements: &'a UnitPlacements,
    /// The placements the script was written from, to keep the text of
    /// unchanged units as the World Editor wrote it.
    pub original: Option<&'a UnitPlacements>,
    pub tables: &'a [ItemTable],
    pub kinds: &'a dyn UnitKinds,
}

fn global_unit_name(unit: &Placement) -> String {
    format!("gg_unit_{}_{:04}", unit.id, unit.creation_number)
}

impl UnitSection<'_> {
    fn generate(&self, text: &str, style: &Style) -> Result<Vec<(String, String)>> {
        let existing = functions(text, style.language);
        let mut originals: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(original) = self.original {
            let mut by_key: HashMap<String, Vec<Vec<String>>> = HashMap::new();

            for function in existing.iter().filter(|f| f.name.starts_with("Create")) {
                for (key, block) in original_blocks(text, function) {
                    by_key.entry(key).or_default().push(block);
                }
            }

            let unchanged: Vec<&Placement> = original.units().iter().collect();

            for unit in unchanged {
                let key = placement_key(style, unit);

                if let Some(block) = by_key.get_mut(&key).and_then(Vec::pop) {
                    originals.insert(identity(unit), block);
                }
            }
        }

        let previous: HashMap<u32, &Placement> = self
            .original
            .map(|o| o.units().iter().map(|u| (u.creation_number, u)).collect())
            .unwrap_or_default();
        let globals: HashSet<String> = global_names(text);
        let mut groups: BTreeMap<(u32, u32, u32), GroupUnits<'_>> = BTreeMap::new();

        for (index, unit) in self.placements.units().iter().enumerate() {
            if self.kinds.is_item(unit.id)
                || unit.id == FourCC(*b"sloc")
                || unit.player > NEUTRAL_PASSIVE
            {
                continue;
            }

            let group = Group::of(unit, self.kinds);

            groups
                .entry(group.order())
                .or_insert_with(|| (group, Vec::new()))
                .1
                .push((index, unit));
        }

        if let Some(original) = self.original {
            let kept: HashSet<u32> = self
                .placements
                .units()
                .iter()
                .map(|u| u.creation_number)
                .collect();

            for unit in original.units() {
                let name = global_unit_name(unit);

                if globals.contains(&name) && !kept.contains(&unit.creation_number) {
                    return Err(Error::Invalid(format!(
                        "{name} is used by the map's triggers, so it cannot be removed here."
                    )));
                }
            }
        }

        let mut generated = Vec::new();

        for (index, unit) in self.placements.units().iter().enumerate() {
            let sets = drop_sets(unit);

            if unit.item_table().is_none() && !sets.is_empty() && !self.kinds.is_item(unit.id) {
                generated.push((
                    unit_drop_name(index),
                    drop_function(style, &unit_drop_name(index), &sets),
                ));
            }
        }

        let mut tables: Vec<&ItemTable> = self.tables.iter().collect();

        tables.sort_by_key(|t| t.number);
        for table in tables {
            let name = table_drop_name(table.number);

            generated.push((name.clone(), drop_function(style, &name, &table.sets)));
        }

        if groups.is_empty() && generated.is_empty() {
            return Ok(Vec::new());
        }

        let mut calls: BTreeMap<&str, Vec<String>> = BTreeMap::new();

        for (group, mut units) in groups.into_values() {
            units.sort_by_key(|(_, u)| u.creation_number);
            let mut body = locals(style, &group);

            if style.create_gap {
                body.push(String::new());
            }

            for (index, unit) in units {
                let unchanged = previous
                    .get(&unit.creation_number)
                    .is_some_and(|before| same_script(before, unit));
                let kept = unchanged.then(|| originals.get(&identity(unit))).flatten();
                let block = match kept {
                    Some(block) => block.clone(),
                    None => {
                        let global = global_unit_name(unit);
                        let target = if globals.contains(&global) {
                            global
                        } else {
                            "u".to_owned()
                        };
                        let waygate: Vec<String> = previous
                            .get(&unit.creation_number)
                            .and_then(|before| originals.get(&identity(before)))
                            .map(|b| {
                                b.iter()
                                    .filter(|l| l.contains("Waygate"))
                                    .cloned()
                                    .collect()
                            })
                            .unwrap_or_default();

                        if is_random(unit.id) {
                            return Err(Error::Invalid(format!(
                                "Unit {} is a random unit; change it in the World Editor.",
                                unit.creation_number
                            )));
                        }

                        unit_block(style, unit, index, &target, self.kinds, &waygate)
                    }
                };

                body.extend(block);
            }

            let caller = match group {
                Group::Buildings(_) => "CreatePlayerBuildings",
                Group::Units(_) => "CreatePlayerUnits",
                _ => "",
            };
            let name = group.name();

            if !caller.is_empty() {
                calls.entry(caller).or_default().push(name.clone());
            }

            generated.push((name.clone(), style.function(&name, &body)));
        }

        for caller in ["CreatePlayerBuildings", "CreatePlayerUnits"] {
            let body: Vec<String> = calls
                .get(caller)
                .map(|names| names.iter().map(|n| style.call_plain(n)).collect())
                .unwrap_or_default();

            generated.push((caller.to_owned(), style.function(caller, &body)));
        }

        let mut all = Vec::new();

        for name in [
            "CreateNeutralHostileBuildings",
            "CreateNeutralPassiveBuildings",
            "CreatePlayerBuildings",
            "CreateNeutralHostile",
            "CreateNeutralPassive",
            "CreatePlayerUnits",
        ] {
            if generated.iter().any(|(n, _)| n == name) {
                all.push(style.call_plain(name));
            }
        }

        generated.push((CREATE_ALL.to_owned(), style.function(CREATE_ALL, &all)));
        Ok(generated)
    }
}

impl Style {
    pub(crate) fn call_plain(&self, name: &str) -> String {
        match self.language {
            Language::Lua => format!("{name}()"),
            Language::Jass => format!("call {name}(  )"),
        }
    }
}

// Random units are placed as `uDNR` (unit) or `bDNR` (building); the game
// picks the real unit from a random group or the placement's own list.
fn is_random(id: FourCC) -> bool {
    id.0[1..] == *b"DNR"
}

fn identity(unit: &Placement) -> String {
    format!("{}#{}", unit.id, unit.creation_number)
}

// Fields that change what the script does for a unit.
fn same_script(a: &Placement, b: &Placement) -> bool {
    a.id == b.id
        && a.x == b.x
        && a.y == b.y
        && a.angle == b.angle
        && a.skin == b.skin
        && a.player == b.player
        && a.life == b.life
        && a.mana == b.mana
        && a.gold == b.gold
        && a.acquisition == b.acquisition
        && a.level == b.level
        && a.attributes == b.attributes
        && a.inventory() == b.inventory()
        && a.abilities == b.abilities
        && a.color == b.color
        && a.waygate == b.waygate
        && a.random == b.random
        && a.drops() == b.drops()
}

fn global_names(text: &str) -> HashSet<String> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|word| word.starts_with("gg_unit_"))
        .map(str::to_owned)
        .collect()
}

/// The generated functions as the World Editor would write them now, in the
/// order they appear in the script. For tests and for writing.
pub fn unit_functions(
    language: Language,
    text: &str,
    section: &UnitSection<'_>,
) -> Result<Vec<(String, String)>> {
    let style = Style::detect(language, text);

    section.generate(text, &style)
}

/// The existing generated unit functions of a script, in file order.
pub fn existing_unit_functions(language: Language, text: &str) -> Vec<(String, String)> {
    functions(text, language)
        .into_iter()
        .filter(|f| is_unit_function(&f.name))
        .map(|f| (f.name, text[f.start..f.end].to_owned()))
        .collect()
}

/// Replaces the script's generated unit functions with `generated`: drop
/// functions where the first old drop function was, creation functions where
/// the first old creation function was, and makes `main` call `CreateAllUnits`.
pub fn splice_units(
    language: Language,
    text: &str,
    generated: &[(String, String)],
) -> Result<String> {
    let is_drop = |name: &str| name.ends_with("_DropItems");
    let block = |drops: bool| -> String {
        generated
            .iter()
            .filter(|(n, _)| is_drop(n) == drops)
            .map(|(_, t)| format!("{t}\n"))
            .collect()
    };
    let all = functions(text, language);
    let fallback = all
        .iter()
        .find(|f| f.name.starts_with("Trig_") || f.name == "InitCustomTriggers" || f.name == "main")
        .map(|f| f.start)
        .ok_or_else(|| Error::Invalid("The map script has no main function.".to_owned()))?;
    let unit_functions: Vec<&Function> = all.iter().filter(|f| is_unit_function(&f.name)).collect();
    let first = |drops: bool| {
        unit_functions
            .iter()
            .find(|f| is_drop(&f.name) == drops)
            .map(|f| f.start)
    };
    let create_at = first(false).unwrap_or(fallback);
    let drop_at = first(true).unwrap_or(create_at);
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut cuts: Vec<(usize, usize)> = unit_functions
        .iter()
        .map(|f| (f.start, skipped_end(text, f.end)))
        .collect();

    cuts.sort_unstable();
    let mut pending = vec![(drop_at, block(true)), (create_at, block(false))];

    pending.sort_by_key(|(at, _)| *at);
    let mut pending = pending.into_iter().peekable();

    for (start, end) in cuts
        .into_iter()
        .chain(std::iter::once((text.len(), text.len())))
    {
        while let Some((at, insert)) = pending.next_if(|(at, _)| *at <= start) {
            out.push_str(&text[cursor..at.max(cursor)]);
            cursor = cursor.max(at);
            out.push_str(&insert);
        }

        out.push_str(&text[cursor..start.max(cursor)]);
        cursor = cursor.max(end);
    }

    let units = generated.iter().any(|(name, _)| name == CREATE_ALL);

    Ok(with_create_call(language, &out, units))
}

// A function is followed by one blank line, which goes with it.
pub(crate) fn skipped_end(text: &str, end: usize) -> usize {
    let rest = &text[end..];

    if rest.starts_with("\r\n") {
        return end + 2;
    }

    if rest.starts_with('\n') {
        return end + 1;
    }

    end
}

/// Makes `main` call `CreateAllUnits` just before `InitBlizzard` when there
/// are units, and drops the call when there are none, as the World Editor does.
fn with_create_call(language: Language, text: &str, units: bool) -> String {
    let (call, before) = match language {
        Language::Lua => ("CreateAllUnits()", "InitBlizzard()"),
        Language::Jass => ("call CreateAllUnits(  )", "call InitBlizzard(  )"),
    };
    let Some(main) = functions(text, language)
        .into_iter()
        .find(|f| f.name == "main")
    else {
        return text.to_owned();
    };
    let body = &text[main.start..main.end];
    let line_of = |at: usize| {
        let start = body[..at].rfind('\n').map_or(0, |i| i + 1);
        let end = body[at..].find('\n').map_or(body.len(), |i| at + i + 1);

        (start, end)
    };

    match (body.find(call), units) {
        (Some(_), true) | (None, false) => text.to_owned(),
        (Some(at), false) => {
            let (start, end) = line_of(at);

            format!(
                "{}{}",
                &text[..main.start + start],
                &text[main.start + end..]
            )
        }
        (None, true) => {
            let Some(at) = body.find(before) else {
                return text.to_owned();
            };
            let (start, _) = line_of(at);
            let indent = &body[start..at];
            let position = main.start + start;

            format!("{}{indent}{call}\n{}", &text[..position], &text[position..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    const FOOTMAN: FourCC = FourCC(*b"hfoo");
    const EMPTY_PLACEMENTS: &[u8] = b"W3do\x0d\x00\x00\x00\x0b\x00\x00\x00\x00\x00\x00\x00";
    const SCRIPT: &str = "function InitGlobals()\nend\n\nfunction main()\nSetMapMusic(\"Music\", true, 0)\nInitBlizzard()\nInitGlobals()\nend\n";

    struct Plain;

    impl UnitKinds for Plain {
        fn is_building(&self, _: FourCC) -> bool {
            false
        }

        fn is_hero(&self, _: FourCC) -> bool {
            false
        }

        fn is_item(&self, _: FourCC) -> bool {
            false
        }

        fn is_gold_mine(&self, _: FourCC) -> bool {
            false
        }

        fn team_color(&self, _: FourCC) -> Option<i32> {
            None
        }

        fn autocast_orders(&self, _: FourCC) -> Option<(String, String)> {
            None
        }
    }

    fn rewritten(
        text: &str,
        placements: &UnitPlacements,
        original: &UnitPlacements,
    ) -> Result<String> {
        let section = UnitSection {
            placements,
            original: Some(original),
            tables: &[],
            kinds: &Plain,
        };
        let generated = unit_functions(Language::Lua, text, &section)?;

        splice_units(Language::Lua, text, &generated)
    }

    #[test]
    fn adding_units_writes_their_functions_and_the_call_in_main() -> TestResult {
        let empty = UnitPlacements::parse(EMPTY_PLACEMENTS)?;
        let mut placements = empty.clone();

        placements.add(FOOTMAN, 0, 128.0, -64.0, 0.0);
        placements.unit_mut(0)?.life = 50;
        let text = rewritten(SCRIPT, &placements, &empty)?;

        assert!(text.contains("u = BlzCreateUnitWithSkin(p, FourCC(\"hfoo\"), 128.0, -64.0, 270.000, FourCC(\"hfoo\"))"));
        assert!(text.contains("SetUnitState(u, UNIT_STATE_LIFE, 0.50 * life)"));
        assert!(text.contains("SetMapMusic(\"Music\", true, 0)\nCreateAllUnits()\nInitBlizzard()"));
        assert!(text.find("function CreateAllUnits()") < text.find("function main()"));
        assert_eq!(rewritten(&text, &placements, &placements)?, text);
        Ok(())
    }

    #[test]
    fn removing_every_unit_removes_the_functions_and_the_call() -> TestResult {
        let empty = UnitPlacements::parse(EMPTY_PLACEMENTS)?;
        let mut placements = empty.clone();

        placements.add(FOOTMAN, 0, 0.0, 0.0, 0.0);
        let with_unit = rewritten(SCRIPT, &placements, &empty)?;

        placements.remove(0)?;
        assert_eq!(rewritten(&with_unit, &placements, &placements)?, SCRIPT);
        Ok(())
    }
}
