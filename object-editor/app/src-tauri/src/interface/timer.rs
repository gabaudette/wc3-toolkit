//! Timer windows: a countdown with a title, such as "Next wave 1:30". It can
//! start with the game, repeat, hide when done, and run a Lua function of the
//! map's when the time runs out.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wc3_map::lua::check_syntax;

use super::{
    MODULE_MARKER, SHOW_DELAY, color_code, comment_text, is_identifier, lua_string, module_file,
};

/// A day in seconds: longer countdowns are surely a typing mistake.
const MAX_DURATION: f64 = 86_400.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimerWindowDesign {
    pub id: String,
    pub name: String,
    pub title: String,
    /// The title's color as `RRGGBB`; the game's own when `None`.
    #[serde(default)]
    pub title_color: Option<String>,
    /// In seconds.
    pub duration: f64,
    /// Starts over each time it runs out.
    pub repeats: bool,
    /// Starts counting shortly after the game starts.
    pub start_at_start: bool,
    /// The window goes away when the time runs out (never for repeating ones).
    pub hide_when_done: bool,
    /// A global Lua function run each time the time runs out, if the map
    /// defines one; empty for none.
    #[serde(default)]
    pub on_done: String,
}

fn check(timer: &TimerWindowDesign) -> Result<(), String> {
    let problem = |what: String| Err(format!("{}: {what}", timer.name));

    if !(timer.duration > 0.0 && timer.duration <= MAX_DURATION) {
        return problem("the countdown must last more than 0 seconds and at most a day".to_owned());
    }

    let function = timer.on_done.trim();

    if !function.is_empty() && !is_identifier(function) {
        return problem(format!(
            "{function:?} is not a Lua function name (letters, digits and _, not starting with a digit)"
        ));
    }

    if let Some(color) = &timer.title_color {
        color_code(color).map_err(|e| format!("{}: {e}", timer.name))?;
    }

    Ok(())
}

/// The Lua module for one timer window, checked as Lua 5.3.
pub fn timer_lua(
    timer: &TimerWindowDesign,
    names: &HashMap<String, String>,
    source: &str,
) -> Result<String, String> {
    check(timer)?;
    let name = names
        .get(&timer.id)
        .ok_or_else(|| format!("{} has no Lua name", timer.name))?;

    let title = match &timer.title_color {
        Some(color) => lua_string(&format!("{}{}|r", color_code(color)?, timer.title)),
        None => lua_string(&timer.title),
    };
    let function = timer.on_done.trim();
    let on_done = if function.is_empty() {
        String::new()
    } else {
        format!(
            "\n        if type({function}) == \"function\" then\n            {function}()\n        end"
        )
    };
    let at_start = if timer.start_at_start {
        format!(
            r#"
        local delay = CreateTimer()
        TimerStart(delay, {SHOW_DELAY}, false, function()
            DestroyTimer(delay)
            UI.{name}.Start()
        end)"#
        )
    } else {
        String::new()
    };

    let code = format!(
        r#"{MODULE_MARKER}: the timer window {name_comment} (UI.{name}), from {source_comment}.
-- Edit it there: Tinkerbench rewrites this file. From your own code:
--   UI.{name}.Start(seconds)   -- seconds left out: the designed {duration} s
--   UI.{name}.Stop(), .Show(), .Hide(), .Remaining()

UI = UI or {{}}

do
    local TITLE = {title}
    local DURATION = {duration}
    local REPEATS = {repeats}
    local HIDE_WHEN_DONE = {hide_when_done}

    local timer = nil
    local window = nil

    local function done(){on_done}
        if HIDE_WHEN_DONE and not REPEATS then
            TimerDialogDisplay(window, false)
        end
    end

    UI.{name} = {{
        Start = function(seconds)
            if timer == nil then return end
            TimerStart(timer, seconds or DURATION, REPEATS, done)
            TimerDialogDisplay(window, true)
        end,
        Stop = function()
            if timer == nil then return end
            PauseTimer(timer)
            TimerDialogDisplay(window, false)
        end,
        Show = function()
            if window ~= nil then TimerDialogDisplay(window, true) end
        end,
        Hide = function()
            if window ~= nil then TimerDialogDisplay(window, false) end
        end,
        -- Seconds left, or 0 before it starts.
        Remaining = function()
            if timer == nil then return 0 end
            return TimerGetRemaining(timer)
        end,
    }}

    Dev.OnStart(function()
        timer = CreateTimer()
        window = CreateTimerDialog(timer)
        TimerDialogSetTitle(window, TITLE){at_start}
    end)
end
"#,
        name_comment = comment_text(&timer.name),
        source_comment = comment_text(source),
        duration = timer.duration,
        repeats = timer.repeats,
        hide_when_done = timer.hide_when_done,
    );

    check_syntax(&module_file(name), &code).map_err(|e| e.to_string())?;
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::{Design, lua_names};

    fn next_wave() -> TimerWindowDesign {
        TimerWindowDesign {
            id: "t".to_owned(),
            name: "Next wave".to_owned(),
            title: "Next wave".to_owned(),
            title_color: Some("FFCC00".to_owned()),
            duration: 90.0,
            repeats: true,
            start_at_start: true,
            hide_when_done: true,
            on_done: "SpawnWave".to_owned(),
        }
    }

    fn names(timer: &TimerWindowDesign) -> HashMap<String, String> {
        lua_names(&[Design::TimerWindow(timer.clone())])
    }

    #[test]
    fn timer_windows_become_valid_lua() {
        let timer = next_wave();
        let code = timer_lua(&timer, &names(&timer), "test UI.json").unwrap();

        assert!(code.starts_with(MODULE_MARKER), "{code}");
        assert!(
            code.contains("local TITLE = \"|cffffcc00Next wave|r\""),
            "{code}"
        );
        assert!(code.contains("local DURATION = 90"), "{code}");
        assert!(code.contains("local REPEATS = true"), "{code}");
        assert!(code.contains("SpawnWave()"), "{code}");
        assert!(code.contains("UI.NextWave.Start()"), "{code}");

        let mut quiet = next_wave();

        quiet.on_done.clear();
        quiet.start_at_start = false;
        let code = timer_lua(&quiet, &names(&quiet), "s").unwrap();

        assert!(!code.contains("SpawnWave"), "{code}");
        assert!(!code.contains("UI.NextWave.Start()\n"), "{code}");
    }

    #[test]
    fn bad_timer_windows_are_reported() {
        let mut instant = next_wave();

        instant.duration = 0.0;
        assert!(timer_lua(&instant, &names(&instant), "s").is_err());

        let mut bad_function = next_wave();

        bad_function.on_done = "spawn wave".to_owned();
        assert!(timer_lua(&bad_function, &names(&bad_function), "s").is_err());
    }
}
