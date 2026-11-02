use crate::info::{Player, Properties};
use crate::script::{Function, Language, Style, functions, skipped_end};
use crate::{Error, Result};

const CONFIG: &str = "config";
const PLAYER_SLOTS: &str = "InitCustomPlayerSlots";
const TEAMS: &str = "InitCustomTeams";
const PRIORITIES: &str = "InitAllyPriorities";
const MAIN: &str = "main";

const MELEE: u32 = 0x0004;
const FIXED_PLAYER_SETTINGS: u32 = 0x0020;
const CUSTOM_FORCES: u32 = 0x0040;
const TERRAIN_FOG: u32 = 0x2000;
const WATER_TINTING: u32 = 0x1_0000;

const FORCE_ALLIED: u32 = 0x01;
const FORCE_ALLIED_VICTORY: u32 = 0x02;
const FORCE_VISION: u32 = 0x08;
const FORCE_CONTROL: u32 = 0x10;
const FORCE_ADVANCED_CONTROL: u32 = 0x20;

const RESCUABLE: u32 = 4;
const USER: u32 = 1;
const NO_WEATHER: [u8; 4] = [0; 4];
const COLOR_SCALE: f32 = 255.0;

// Lines of `main` between `SetDayNightModels` and `NewSoundEnvironment` that
// come from the map's properties.
const DAY_NIGHT: &str = "SetDayNightModels";
const SOUND: &str = "NewSoundEnvironment";
const DEFAULT_SOUND: &str = "Default";
const HD_WATER: &str = "SetHDWaterParamsEx";

fn controller(value: u32) -> &'static str {
    match value {
        1 => "MAP_CONTROL_USER",
        2 => "MAP_CONTROL_COMPUTER",
        3 => "MAP_CONTROL_NEUTRAL",
        4 => "MAP_CONTROL_RESCUABLE",
        _ => "MAP_CONTROL_NONE",
    }
}

fn race(value: u32) -> &'static str {
    match value {
        1 => "RACE_PREF_HUMAN",
        2 => "RACE_PREF_ORC",
        3 => "RACE_PREF_UNDEAD",
        4 => "RACE_PREF_NIGHTELF",
        _ => "RACE_PREF_RANDOM",
    }
}

// The HUD skin is a race preference bit; 0 and 1 (human) write nothing, and
// the World Editor writes any value it does not know as random.
fn skin(value: u32) -> Option<&'static str> {
    match value {
        0 | 1 => None,
        2 => Some("RACE_PREF_ORC"),
        4 => Some("RACE_PREF_NIGHTELF"),
        8 => Some("RACE_PREF_UNDEAD"),
        16 => Some("RACE_PREF_DEMON"),
        64 => Some("RACE_PREF_USER_SELECTABLE"),
        128 => Some("RACE_PREF_FORSAKEN"),
        _ => Some("RACE_PREF_RANDOM"),
    }
}

fn has_bit(mask: u32, bit: u32) -> bool {
    mask.checked_shr(bit).is_some_and(|rest| rest & 1 != 0)
}

fn player(number: u32) -> String {
    format!("Player({number})")
}

fn decimal(value: f32, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

fn bool_text(value: bool) -> String {
    value.to_string()
}

struct Setup<'a> {
    style: Style,
    // No property decides the game placement of maps that are not melee, so
    // the script keeps the one it has.
    placement: Option<String>,
    // Newer World Editors end `InitCustomPlayerSlots` with a blank line.
    slots_gap: bool,
    properties: &'a Properties,
    bounds: Option<[f32; 4]>,
}

impl Setup<'_> {
    fn jass(&self) -> bool {
        self.style.language == Language::Jass
    }

    // JASS gets the World Editor's comments and blank lines; Lua has neither.
    fn comment(&self, lines: &mut Vec<String>, text: &str) {
        if self.jass() {
            lines.push(format!("// {text}"));
        }
    }

    fn gap(&self, lines: &mut Vec<String>) {
        if self.jass() {
            lines.push(String::new());
        }
    }

    fn call(&self, function: &str, args: &[String]) -> String {
        self.style.call(function, args)
    }

    fn players(&self) -> &[Player] {
        &self.properties.players
    }

    fn custom_forces(&self) -> bool {
        self.properties.flags & CUSTOM_FORCES != 0
    }

    fn config(&self) -> String {
        let p = self.properties;
        let mut lines = vec![
            self.call("SetMapName", &[format!("\"{}\"", p.name)]),
            self.call("SetMapDescription", &[format!("\"{}\"", p.description)]),
            self.call("SetPlayers", &[self.players().len().to_string()]),
            self.call("SetTeams", &[self.players().len().to_string()]),
        ];
        let melee = match self.properties.flags & MELEE != 0 {
            true => "MAP_PLACEMENT_TEAMS_TOGETHER",
            false => "MAP_PLACEMENT_USE_MAP_SETTINGS",
        };
        let placement = self.placement.as_deref().unwrap_or(melee);

        lines.push(self.call("SetGamePlacement", &[placement.to_owned()]));
        self.gap(&mut lines);
        for (index, player) in self.players().iter().enumerate() {
            let [x, y] = player.start();

            lines.push(self.call(
                "DefineStartLocation",
                &[index.to_string(), decimal(x, 1), decimal(y, 1)],
            ));
        }

        self.gap(&mut lines);
        self.comment(&mut lines, "Player setup");
        lines.push(self.style.call_plain(PLAYER_SLOTS));
        if self.custom_forces() {
            lines.push(self.style.call_plain(TEAMS));
        } else {
            for player in self.players() {
                lines.push(self.call(
                    "SetPlayerSlotAvailable",
                    &[self::player(player.number), controller(USER).to_owned()],
                ));
            }

            lines.push(self.style.call_plain("InitGenericPlayerSlots"));
        }

        if self.has_priorities() {
            lines.push(self.style.call_plain(PRIORITIES));
        }

        self.style.function(CONFIG, &lines)
    }

    fn player_slots(&self) -> String {
        let fixed = self.properties.flags & FIXED_PLAYER_SETTINGS != 0;
        let mut lines = Vec::new();

        for (index, p) in self.players().iter().enumerate() {
            let me = player(p.number);

            self.gap(&mut lines);
            self.comment(&mut lines, &format!("Player {}", p.number));
            lines.push(self.call("SetPlayerStartLocation", &[me.clone(), index.to_string()]));
            if p.fixed_start {
                lines.push(self.call("ForcePlayerStartLocation", &[me.clone(), index.to_string()]));
            }

            lines.push(self.call(
                "SetPlayerColor",
                &[me.clone(), format!("ConvertPlayerColor({})", p.number)],
            ));
            lines.push(self.call(
                "SetPlayerRacePreference",
                &[me.clone(), race(p.race).to_owned()],
            ));
            if let Some(skin) = p.hud_skin().and_then(skin) {
                lines.push(self.call("SetPlayerRaceSkin", &[me.clone(), skin.to_owned()]));
            }

            lines.push(self.call("SetPlayerRaceSelectable", &[me.clone(), bool_text(!fixed)]));
            lines.push(self.call(
                "SetPlayerController",
                &[me.clone(), controller(p.controller).to_owned()],
            ));
            if p.controller == RESCUABLE {
                for other in self.players().iter().filter(|o| o.controller == USER) {
                    lines.push(self.call(
                        "SetPlayerAlliance",
                        &[
                            me.clone(),
                            player(other.number),
                            "ALLIANCE_RESCUABLE".to_owned(),
                            bool_text(true),
                        ],
                    ));
                }
            }
        }

        if self.slots_gap {
            lines.push(String::new());
        }

        self.style.function(PLAYER_SLOTS, &lines)
    }

    fn teams(&self) -> String {
        let mut lines = Vec::new();
        let sections = [
            (FORCE_ALLIED, "Allied", "SetPlayerAllianceStateAllyBJ"),
            (
                FORCE_VISION,
                "Shared Vision",
                "SetPlayerAllianceStateVisionBJ",
            ),
            (
                FORCE_CONTROL,
                "Shared Control",
                "SetPlayerAllianceStateControlBJ",
            ),
            (
                FORCE_ADVANCED_CONTROL,
                "Shared Advanced Control",
                "SetPlayerAllianceStateFullControlBJ",
            ),
        ];

        for (team, force) in self.properties.forces.iter().enumerate() {
            let members: Vec<u32> = self
                .players()
                .iter()
                .map(|p| p.number)
                .filter(|n| has_bit(force.players, *n))
                .collect();

            if members.is_empty() {
                continue;
            }

            self.comment(&mut lines, &format!("Force: {}", force.name));
            for member in &members {
                lines.push(self.call("SetPlayerTeam", &[player(*member), team.to_string()]));
                if force.flags & FORCE_ALLIED_VICTORY != 0 {
                    lines.push(self.call(
                        "SetPlayerState",
                        &[
                            player(*member),
                            "PLAYER_STATE_ALLIED_VICTORY".to_owned(),
                            "1".to_owned(),
                        ],
                    ));
                }
            }

            self.gap(&mut lines);
            for (flag, title, function) in sections {
                if force.flags & flag == 0 || members.len() < 2 {
                    continue;
                }

                self.comment(&mut lines, &format!("  {title}"));
                for a in &members {
                    for b in members.iter().filter(|b| *b != a) {
                        lines.push(self.call(function, &[player(*a), player(*b), bool_text(true)]));
                    }
                }

                self.gap(&mut lines);
            }
        }

        self.style.function(TEAMS, &lines)
    }

    fn index_of(&self, number: u32) -> Option<usize> {
        self.players().iter().position(|p| p.number == number)
    }

    fn priority_lines(&self, index: usize, masks: [u32; 2], enemy: bool) -> Vec<String> {
        let [low, high] = masks;
        // The count includes the player's own start location, which gets no
        // line; a location in both masks is low priority.
        let marked: Vec<(usize, &str)> = (0..u32::BITS)
            .filter(|bit| has_bit(low | high, *bit))
            .filter_map(|bit| {
                let at = self.index_of(bit)?;
                let level = match has_bit(low, bit) {
                    true => "MAP_LOC_PRIO_LOW",
                    false => "MAP_LOC_PRIO_HIGH",
                };

                Some((at, level))
            })
            .collect();
        let others: Vec<&(usize, &str)> = marked.iter().filter(|(at, _)| *at != index).collect();
        let (count, one) = match enemy {
            true => ("SetEnemyStartLocPrioCount", "SetEnemyStartLocPrio"),
            false => ("SetStartLocPrioCount", "SetStartLocPrio"),
        };

        if marked.is_empty() {
            return Vec::new();
        }

        let mut lines = Vec::new();

        self.gap(&mut lines);
        lines.push(self.call(count, &[index.to_string(), marked.len().to_string()]));
        for (slot, (at, level)) in others.iter().enumerate() {
            lines.push(self.call(
                one,
                &[
                    index.to_string(),
                    slot.to_string(),
                    at.to_string(),
                    (*level).to_owned(),
                ],
            ));
        }

        lines
    }

    fn priorities_body(&self) -> Vec<String> {
        let mut lines = Vec::new();

        for (index, p) in self.players().iter().enumerate() {
            lines.extend(self.priority_lines(index, p.ally_priorities(), false));
            if let Some(enemy) = p.enemy_priorities() {
                lines.extend(self.priority_lines(index, enemy, true));
            }
        }

        lines
    }

    fn has_priorities(&self) -> bool {
        !self.priorities_body().is_empty()
    }

    fn priorities(&self) -> String {
        self.style.function(PRIORITIES, &self.priorities_body())
    }

    fn fog(&self) -> Vec<String> {
        let p = self.properties;
        let f = &p.fog;

        if p.flags & TERRAIN_FOG == 0 {
            return Vec::new();
        }

        let [red, green, blue] =
            [f.color[2], f.color[1], f.color[0]].map(|c| decimal(f32::from(c) / COLOR_SCALE, 3));
        let head = [
            f.style.to_string(),
            decimal(f.start, 1),
            decimal(f.end, 1),
            decimal(f.density, 3),
        ];

        match p.fog_extra() {
            Some((extra, max_density)) => vec![
                self.call(
                    "SetTerrainFogExV",
                    &[
                        head.to_vec(),
                        extra.map(|v| decimal(v, 1)).to_vec(),
                        vec![red, green, blue],
                    ]
                    .concat(),
                ),
                self.call(
                    "BlzSetTerrainFogMaxLinearDensity",
                    &[decimal(max_density, 1)],
                ),
                self.call(
                    "BlzSetTerrainFogDrawOverSky",
                    &[bool_text(p.fog_over_sky())],
                ),
            ],
            None => vec![self.call(
                "SetTerrainFogEx",
                &[head.to_vec(), vec![red, green, blue]].concat(),
            )],
        }
    }

    fn water(&self) -> Vec<String> {
        let p = self.properties;

        if p.flags & WATER_TINTING == 0 {
            return Vec::new();
        }

        let [blue, green, red, alpha] = p.water_color;

        vec![self.call(
            "SetWaterBaseColor",
            &[red, green, blue, alpha].map(|c| c.to_string()),
        )]
    }

    fn weather(&self) -> Vec<String> {
        let p = self.properties;

        if p.weather.0 == NO_WEATHER {
            return Vec::new();
        }

        let Some(bounds) = self.bounds else {
            return Vec::new();
        };
        let corners = bounds.map(|v| decimal(v, 1));
        let rect = match self.style.language {
            Language::Lua => format!("Rect({})", corners.join(", ")),
            Language::Jass => format!("Rect({})", corners.join(",")),
        };

        vec![
            self.style.assign(
                "we",
                &self
                    .style
                    .expression("AddWeatherEffect", &[rect, self.style.id(p.weather)]),
            ),
            self.call("EnableWeatherEffect", &["we".to_owned(), bool_text(true)]),
        ]
    }

    // An empty sound environment is the default one.
    fn sound(&self) -> String {
        let name = match self.properties.sound_environment.as_str() {
            "" => DEFAULT_SOUND,
            name => name,
        };

        self.call(SOUND, &[format!("\"{name}\"")])
    }

    fn weather_local(&self) -> Vec<String> {
        if self.weather().is_empty() {
            return Vec::new();
        }

        match self.style.language {
            Language::Lua => vec!["local we".to_owned(), String::new()],
            Language::Jass => vec!["local weathereffect we".to_owned()],
        }
    }
}

/// The map's properties and the terrain bounds weather covers.
pub struct SetupSection<'a> {
    pub properties: &'a Properties,
    pub bounds: Option<[f32; 4]>,
}

fn placement_of(config: &str) -> Option<String> {
    let at = config.find("MAP_PLACEMENT_")?;
    let name: String = config[at..]
        .chars()
        .take_while(|c| c.is_ascii_uppercase() || *c == '_')
        .collect();

    Some(name)
}

fn ends_with_gap(function: &str) -> bool {
    function
        .lines()
        .rev()
        .nth(1)
        .is_some_and(|line| line.trim().is_empty())
}

fn find<'f>(found: &'f [Function], name: &str) -> Option<&'f Function> {
    found.iter().find(|f| f.name == name)
}

fn setup_style(language: Language, text: &str) -> Style {
    let mut style = Style::detect(language, text);
    let found = functions(text, language);

    if let Some(config) = find(&found, CONFIG) {
        let body = &text[config.start..config.end];

        if let Some(line) = body.lines().nth(1) {
            style.indent = line[..line.len() - line.trim_start().len()].to_owned();
        }
    }

    style
}

fn rewrite_main(setup: &Setup<'_>, body: &str) -> Option<String> {
    let lines: Vec<&str> = body.split_inclusive('\n').collect();
    let indent = &setup.style.indent;
    let calls = |line: &str, name: &str| {
        let trimmed = line.trim_start();

        trimmed
            .strip_prefix("call ")
            .unwrap_or(trimmed)
            .starts_with(name)
    };
    let day_night = lines.iter().position(|l| calls(l, DAY_NIGHT))?;
    let sound = lines.iter().position(|l| calls(l, SOUND))?;

    if sound <= day_night {
        return None;
    }

    let hd_water: Vec<&str> = lines[day_night + 1..sound]
        .iter()
        .copied()
        .filter(|l| l.contains(HD_WATER))
        .collect();
    let mut generated: Vec<String> = setup.fog();

    generated.extend(setup.water());
    let mut out: Vec<String> = generated.iter().map(|l| format!("{indent}{l}\n")).collect();

    out.extend(hd_water.iter().map(|l| (*l).to_owned()));
    out.extend(setup.weather().iter().map(|l| format!("{indent}{l}\n")));
    out.push(format!("{indent}{}\n", setup.sound()));

    let declarations = |line: &str| {
        let trimmed = line.trim();

        trimmed == "local we" || trimmed == "local weathereffect we"
    };
    let header = lines.first()?;
    let mut rest = 1;

    while lines.get(rest).is_some_and(|l| declarations(l)) {
        rest += 1;
        if setup.style.language == Language::Lua
            && lines.get(rest).is_some_and(|l| l.trim().is_empty())
        {
            rest += 1;
        }
    }

    let mut text = (*header).to_owned();

    for line in setup.weather_local() {
        if line.is_empty() {
            text.push('\n');
            continue;
        }

        text.push_str(&format!("{indent}{line}\n"));
    }

    text.extend(lines[rest..=day_night].iter().copied());
    text.extend(out);
    text.extend(lines[sound + 1..].iter().copied());
    Some(text)
}

pub fn rewrite(language: Language, text: &str, section: &SetupSection<'_>) -> Result<String> {
    let found = functions(text, language);
    let setup = Setup {
        style: setup_style(language, text),
        placement: find(&found, CONFIG).and_then(|f| placement_of(&text[f.start..f.end])),
        slots_gap: find(&found, PLAYER_SLOTS).is_some_and(|f| ends_with_gap(&text[f.start..f.end])),
        properties: section.properties,
        bounds: section.bounds,
    };
    let main = find(&found, MAIN)
        .ok_or_else(|| Error::Invalid("The map script has no main function.".to_owned()))?;
    let config = find(&found, CONFIG)
        .ok_or_else(|| Error::Invalid("The map script has no config function.".to_owned()))?;
    let new_main = rewrite_main(&setup, &text[main.start..main.end]).ok_or_else(|| {
        Error::Invalid("The map script's main function is not the World Editor's.".to_owned())
    })?;
    let wanted: Vec<(&str, Option<String>)> = vec![
        (PLAYER_SLOTS, Some(setup.player_slots())),
        (TEAMS, Some(setup.teams())),
        (
            PRIORITIES,
            setup.has_priorities().then(|| setup.priorities()),
        ),
    ];
    let mut edits: Vec<(usize, usize, String)> = vec![
        (main.start, main.end, new_main),
        (config.start, config.end, setup.config()),
    ];

    for (name, generated) in wanted {
        match (find(&found, name), generated) {
            (Some(existing), Some(generated)) => {
                edits.push((existing.start, existing.end, generated))
            }
            (Some(existing), None) => edits.push((
                existing.start,
                skipped_end(text, existing.end),
                String::new(),
            )),
            (None, Some(generated)) => {
                edits.push((main.start, main.start, format!("{generated}\n")))
            }
            (None, None) => {}
        }
    }

    edits.sort_by_key(|(start, end, _)| (*start, *end));
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;

    for (start, end, replacement) in edits {
        out.push_str(&text[cursor..start]);
        out.push_str(&replacement);
        cursor = end;
    }

    out.push_str(&text[cursor..]);
    Ok(out)
}

pub fn reproduces(language: Language, text: &str, section: &SetupSection<'_>) -> bool {
    rewrite(language, text, section).is_ok_and(|rewritten| rewritten == text)
}
