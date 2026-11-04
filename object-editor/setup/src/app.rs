//! The wizard: one page per step, with Back and Next. Slow work (checking the
//! game, installing packages, installing) runs on a thread so the window stays live.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use eframe::egui::{
    self, Align, CentralPanel, Frame, Layout, Margin, ScrollArea, Stroke, StrokeKind, TextEdit,
};
use wc3_install::{FoundGame, GameCheck, SetupChoices, check_game, find_games};

use crate::install::{self, InstallRecord, Plan};
use crate::payload;
use crate::requirements::{self, Requirements, WEBVIEW2_DOWNLOAD};
use crate::theme;

const STEPS_WIDTH: f32 = 210.0;
const PAGE_MARGIN: i8 = 28;
const SIDE_MARGIN: i8 = 20;
const NAV_MARGIN: i8 = 14;
const NOTICES_HEIGHT: f32 = 220.0;
const LICENSE_HEIGHT: f32 = 160.0;
const FRAME_INSET: f32 = 3.0;
/// The keyboard focus ring, drawn around the focused control (WCAG 2.4.7).
const FOCUS_GAP: f32 = 2.0;
const FOCUS_WIDTH: f32 = 2.0;
const SHARED_LUA_FOLDER: &str = "Tinkerbench Lua";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Welcome,
    License,
    Folder,
    Requirements,
    Game,
    Maps,
    TestMaps,
    Install,
    Finish,
    Remove,
    Removed,
}

const INSTALL_STEPS: &[(Page, &str)] = &[
    (Page::Welcome, "Welcome"),
    (Page::License, "License"),
    (Page::Folder, "Install folder"),
    (Page::Requirements, "Requirements"),
    (Page::Game, "Warcraft III"),
    (Page::Maps, "Maps folder"),
    (Page::TestMaps, "Test maps"),
    (Page::Install, "Install"),
    (Page::Finish, "Finish"),
];

const REMOVE_STEPS: &[(Page, &str)] = &[
    (Page::Welcome, "Welcome"),
    (Page::Remove, "Remove"),
    (Page::Removed, "Finish"),
];

/// Work running on another thread; the window repaints when it ends.
enum Job<T> {
    Idle,
    Running(mpsc::Receiver<T>),
    Done(T),
}

impl<T: Send + 'static> Job<T> {
    fn start(ctx: &egui::Context, work: impl FnOnce() -> T + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            sender.send(work()).ok();
            ctx.request_repaint();
        });

        Self::Running(receiver)
    }

    fn poll(&mut self) {
        if let Self::Running(receiver) = self
            && let Ok(result) = receiver.try_recv()
        {
            *self = Self::Done(result);
        }
    }

    fn running(&self) -> bool {
        matches!(self, Self::Running(_))
    }

    fn done(&self) -> Option<&T> {
        match self {
            Self::Done(result) => Some(result),
            _ => None,
        }
    }
}

enum InstallEvent {
    Step(String),
    Done(Result<InstallRecord, String>),
}

pub struct Options {
    pub dry_run: bool,
    pub uninstall: bool,
}

pub struct SetupApp {
    page: Page,
    dry_run: bool,
    existing: Option<InstallRecord>,
    problem: Option<String>,
    remove_chosen: bool,
    install_dir: String,
    desktop_shortcut: bool,
    requirements: Job<Requirements>,
    fix: Job<Result<String, String>>,
    games: Vec<FoundGame>,
    game_dir: String,
    game_check: Job<(String, Result<GameCheck, String>)>,
    hd: bool,
    maps_dir: String,
    shared_lua: bool,
    lua_folder: String,
    install_log: Vec<String>,
    install_events: Option<mpsc::Receiver<InstallEvent>>,
    installed: Option<Result<InstallRecord, String>>,
    launch: bool,
    remove_settings: bool,
    removal: Job<Result<(), String>>,
}

fn text(path: Option<&Path>) -> String {
    path.map(|p| p.display().to_string()).unwrap_or_default()
}

fn pick_folder(current: &str) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();

    if !current.is_empty() {
        dialog = dialog.set_directory(current);
    }

    dialog.pick_folder().map(|p| p.display().to_string())
}

/// A writable folder: test copies of maps are written beside the maps.
fn writable(dir: &Path) -> Result<(), String> {
    tempfile::tempfile_in(dir)
        .map(drop)
        .map_err(|e| format!("Tinkerbench cannot write in {}: {e}", dir.display()))
}

impl SetupApp {
    pub fn new(ctx: &egui::Context, options: Options) -> Self {
        theme::apply(ctx);

        let (existing, problem) = match InstallRecord::load() {
            Ok(record) => (record, None),
            Err(e) => (None, Some(e.to_string())),
        };
        let choices = SetupChoices::load().ok().flatten();
        let games = find_games();
        let game_dir = choices
            .as_ref()
            .map(|c| text(Some(&c.game_dir)))
            .or_else(|| games.first().map(|g| text(Some(&g.dir))))
            .unwrap_or_default();
        let maps_dir = choices
            .as_ref()
            .and_then(|c| c.maps_dir.as_deref())
            .or_else(|| games.first().and_then(|g| g.maps_dir.as_deref()))
            .map(|p| text(Some(p)))
            .unwrap_or_default();
        let lua_folder = choices.as_ref().and_then(|c| c.lua_folder.clone());
        let install_dir = existing
            .as_ref()
            .map(|r| r.dir.clone())
            .or_else(install::default_dir);
        let page = if options.uninstall && existing.is_some() {
            Page::Remove
        } else {
            Page::Welcome
        };

        Self {
            page,
            dry_run: options.dry_run,
            remove_chosen: options.uninstall,
            existing,
            problem,
            install_dir: text(install_dir.as_deref()),
            desktop_shortcut: false,
            requirements: Job::Idle,
            fix: Job::Idle,
            games,
            game_dir,
            game_check: Job::Idle,
            hd: choices.as_ref().is_some_and(|c| c.hd),
            maps_dir,
            shared_lua: lua_folder.is_some(),
            lua_folder: text(lua_folder.as_deref()),
            install_log: Vec::new(),
            install_events: None,
            installed: None,
            launch: true,
            remove_settings: false,
            removal: Job::Idle,
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        self.requirements.poll();
        self.game_check.poll();
        self.removal.poll();

        let fixed = self.fix.running();

        self.fix.poll();

        if fixed && matches!(self.fix.done(), Some(Ok(_))) {
            self.requirements = Job::start(ctx, requirements::check);
        }

        if let Some(events) = &self.install_events {
            while let Ok(event) = events.try_recv() {
                match event {
                    InstallEvent::Step(step) => self.install_log.push(step),
                    InstallEvent::Done(result) => {
                        if result.is_ok() {
                            self.page = Page::Finish;
                        }

                        self.installed = Some(result);
                    }
                }
            }
        }

        if self.page == Page::Game && !self.game_check.running() {
            let checked = self.game_check.done().map(|(dir, _)| dir.as_str());

            if !self.game_dir.trim().is_empty() && checked != Some(self.game_dir.trim()) {
                let dir = self.game_dir.trim().to_owned();

                self.game_check = Job::start(ctx, move || {
                    let result = check_game(Path::new(&dir)).map_err(|e| e.to_string());

                    (dir, result)
                });
            }
        }

        if self.page == Page::Requirements && matches!(self.requirements, Job::Idle) {
            self.requirements = Job::start(ctx, requirements::check);
        }
    }

    fn checked_game(&self) -> Option<&GameCheck> {
        match self.game_check.done() {
            Some((dir, Ok(check))) if dir == self.game_dir.trim() => Some(check),
            _ => None,
        }
    }

    fn plan(&self) -> Plan {
        let maps_dir = self.maps_dir.trim();
        let lua_folder = self.lua_folder.trim();

        Plan {
            dir: PathBuf::from(self.install_dir.trim()),
            desktop_shortcut: self.desktop_shortcut,
            choices: SetupChoices {
                game_dir: PathBuf::from(self.game_dir.trim()),
                hd: self.hd,
                maps_dir: (!maps_dir.is_empty()).then(|| PathBuf::from(maps_dir)),
                lua_folder: (self.shared_lua && !lua_folder.is_empty())
                    .then(|| PathBuf::from(lua_folder)),
            },
        }
    }

    fn start_install(&mut self, ctx: &egui::Context) {
        let plan = self.plan();
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();

        self.install_log.clear();
        self.installed = None;
        self.install_events = Some(receiver);
        std::thread::spawn(move || {
            let mut report = |step: &str| {
                sender.send(InstallEvent::Step(step.to_owned())).ok();
                ctx.request_repaint();
            };
            let result = install::install(&plan, &mut report).map_err(|e| e.to_string());

            sender.send(InstallEvent::Done(result)).ok();
            ctx.request_repaint();
        });
    }

    fn steps(&self) -> &'static [(Page, &'static str)] {
        if self.remove_chosen {
            REMOVE_STEPS
        } else {
            INSTALL_STEPS
        }
    }

    fn step_list(&self, ui: &mut egui::Ui) {
        ui.label(theme::title("Tinkerbench"));
        ui.label(theme::faint(&format!(
            "Setup · version {}",
            env!("CARGO_PKG_VERSION")
        )));
        ui.add_space(18.0);

        let steps = self.steps();
        let current = steps.iter().position(|(page, _)| *page == self.page);

        for (index, (_, name)) in steps.iter().enumerate() {
            let label = match current {
                Some(at) if index < at => theme::muted(&format!("    {name}")),
                Some(at) if index == at => theme::strong(&format!("›  {name}")).color(theme::GOLD),
                _ => theme::upcoming(&format!("    {name}")),
            };

            ui.label(label);
        }
    }

    /// Back, then the page's main action on the right; Cancel on the left.
    fn navigation(&mut self, ui: &mut egui::Ui) {
        let busy = self.requirements.running()
            || self.fix.running()
            || self.install_events.is_some() && self.installed.is_none()
            || self.removal.running();
        let ctx = ui.ctx().clone();

        ui.horizontal(|ui| {
            let closing = matches!(self.page, Page::Finish | Page::Removed);

            if !closing && ui.add_enabled(!busy, egui::Button::new("Cancel")).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                self.forward(ui, &ctx, busy);

                if let Some(back) = self.back_page()
                    && ui.add_enabled(!busy, egui::Button::new("Back")).clicked()
                {
                    self.page = back;
                }
            });
        });
    }

    fn back_page(&self) -> Option<Page> {
        match self.page {
            Page::License => Some(Page::Welcome),
            Page::Folder => Some(Page::License),
            Page::Requirements => Some(Page::Folder),
            Page::Game => Some(Page::Requirements),
            Page::Maps => Some(Page::Game),
            Page::TestMaps => Some(Page::Maps),
            Page::Install => Some(Page::TestMaps),
            Page::Remove => Some(Page::Welcome),
            Page::Welcome | Page::Finish | Page::Removed => None,
        }
    }

    fn forward(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, busy: bool) {
        match self.page {
            Page::Welcome => {
                let next = if self.remove_chosen {
                    Page::Remove
                } else {
                    Page::License
                };

                if ui.add(theme::primary("Next")).clicked() {
                    self.page = next;
                }
            }
            Page::License => self.next(ui, true, Page::Folder),
            Page::Folder => {
                let ready = Path::new(self.install_dir.trim()).is_absolute();

                self.next(ui, ready, Page::Requirements);
            }
            Page::Requirements => self.next(ui, !busy, Page::Game),
            Page::Game => {
                if ui
                    .add_enabled(self.checked_game().is_some(), theme::primary("Next"))
                    .clicked()
                {
                    if self.maps_dir.trim().is_empty()
                        && let Some(found) = self
                            .games
                            .iter()
                            .find(|g| text(Some(&g.dir)) == self.game_dir.trim())
                    {
                        self.maps_dir = text(found.maps_dir.as_deref());
                    }

                    self.page = Page::Maps;
                }
            }
            Page::Maps => {
                if ui.add(theme::primary("Next")).clicked() {
                    let dir = self.maps_dir.trim();

                    self.problem = None;

                    if !dir.is_empty()
                        && let Err(e) = writable(Path::new(dir))
                    {
                        self.problem = Some(e);
                    } else {
                        if self.lua_folder.is_empty()
                            && let Some(parent) = Path::new(dir).parent()
                        {
                            self.lua_folder = text(Some(&parent.join(SHARED_LUA_FOLDER)));
                        }

                        self.page = Page::TestMaps;
                    }
                }
            }
            Page::TestMaps => {
                let ready = !self.shared_lua || Path::new(self.lua_folder.trim()).is_absolute();

                self.next(ui, ready, Page::Install);
            }
            Page::Install => {
                let label = match self.installed {
                    Some(Err(_)) => "Try again",
                    _ => "Install",
                };

                if ui.add_enabled(!busy, theme::primary(label)).clicked() {
                    self.start_install(ctx);
                }
            }
            Page::Finish => {
                if ui.add(theme::primary("Finish")).clicked() {
                    self.finish(ctx);
                }
            }
            Page::Remove => {
                if ui.add_enabled(!busy, theme::primary("Remove")).clicked()
                    && let Some(record) = self.existing.clone()
                {
                    let remove_settings = self.remove_settings;

                    self.removal = Job::start(ctx, move || {
                        install::uninstall(&record, remove_settings).map_err(|e| e.to_string())
                    });
                }

                if matches!(self.removal.done(), Some(Ok(()))) {
                    self.page = Page::Removed;
                }
            }
            Page::Removed => {
                if ui.add(theme::primary("Close")).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn next(&mut self, ui: &mut egui::Ui, ready: bool, to: Page) {
        if ui.add_enabled(ready, theme::primary("Next")).clicked() {
            self.problem = None;
            self.page = to;
        }
    }

    fn finish(&mut self, ctx: &egui::Context) {
        if self.launch
            && let Some(Ok(record)) = &self.installed
            && let Err(e) = install::launch(&record.binary)
        {
            self.problem = Some(format!(
                "Tinkerbench is installed, but it did not start: {e}"
            ));
            return;
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn page(&mut self, ui: &mut egui::Ui) {
        match self.page {
            Page::Welcome => self.welcome(ui),
            Page::License => license(ui),
            Page::Folder => self.folder(ui),
            Page::Requirements => self.requirements_page(ui),
            Page::Game => self.game(ui),
            Page::Maps => self.maps(ui),
            Page::TestMaps => self.test_maps(ui),
            Page::Install => self.install_page(ui),
            Page::Finish => self.finish_page(ui),
            Page::Remove => self.remove_page(ui),
            Page::Removed => removed(ui),
        }

        if let Some(problem) = &self.problem {
            ui.add_space(8.0);
            ui.label(theme::error(problem));
        }
    }

    fn welcome(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Welcome"));
        ui.label(
            "Tinkerbench is a workbench for Warcraft III Reforged maps. This setup installs it \
             for your user, with anything it needs from the system, and finds your game.",
        );
        ui.label(theme::muted(
            "It reads the game data from your own Warcraft III install; nothing is uploaded anywhere.",
        ));

        if self.dry_run {
            ui.label(theme::muted(
                "Dry run: system packages are not installed; the setup shows the command instead.",
            ));
        }

        if let Some(record) = &self.existing {
            ui.add_space(8.0);
            ui.label(theme::strong(&format!(
                "Tinkerbench {} is installed in:",
                record.version
            )));
            ui.label(theme::path(&record.dir.display().to_string()));
            ui.radio_value(
                &mut self.remove_chosen,
                false,
                "Update or repair it, keeping your choices",
            );
            ui.radio_value(&mut self.remove_chosen, true, "Remove it");
        }
    }

    fn folder(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Install folder"));
        ui.label("Tinkerbench installs for your user only, so it needs no administrator rights.");
        folder_field(ui, "Install in", &mut self.install_dir);
        ui.checkbox(&mut self.desktop_shortcut, "Add a shortcut on the desktop");

        if cfg!(windows) {
            ui.label(theme::muted("It is added to the Start menu."));
        } else {
            ui.label(theme::muted(
                "It is added to your applications menu, and as the tinkerbench command.",
            ));
        }
    }

    fn requirements_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Requirements"));
        ui.label(if cfg!(windows) {
            "Tinkerbench draws its window with Microsoft Edge WebView2."
        } else {
            "Tinkerbench draws its window with WebKitGTK, which comes from your distribution."
        });

        match &self.requirements {
            Job::Idle | Job::Running(_) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Checking what is installed…");
                });
            }
            Job::Done(Requirements::Ready) => {
                ui.label(theme::done("Everything Tinkerbench needs is installed."));
            }
            Job::Done(Requirements::Unknown(reason)) => {
                ui.label(theme::muted(reason));
            }
            Job::Done(Requirements::Missing { missing, fix }) => {
                let missing = missing.clone();
                let fix = fix.clone();

                ui.label(theme::strong("Missing:"));

                for item in &missing {
                    ui.label(theme::path(item));
                }

                match fix {
                    Some(fix) => {
                        ui.label(fix.explanation());

                        let ctx = ui.ctx().clone();

                        if ui
                            .add_enabled(!self.fix.running(), theme::primary("Install it"))
                            .clicked()
                        {
                            let dry_run = self.dry_run;
                            let fix = fix.clone();

                            self.fix = Job::start(&ctx, move || fix.apply(dry_run));
                        }

                        match fix.for_terminal() {
                            Some(command) => {
                                ui.label(theme::muted(
                                    "Or run this in a terminal, then check again:",
                                ));
                                ui.label(theme::path(&command));
                            }
                            None => {
                                ui.label(theme::muted(
                                    "Or install it from Microsoft, then check again:",
                                ));
                                ui.hyperlink(WEBVIEW2_DOWNLOAD);
                            }
                        }
                    }
                    None if cfg!(windows) => {
                        ui.label("Install the WebView2 Runtime from Microsoft, then check again:");
                        ui.hyperlink(WEBVIEW2_DOWNLOAD);
                    }
                    None => {
                        ui.label(
                            "Install WebKitGTK 4.1 with your distribution's package manager, \
                             then check again.",
                        );
                    }
                }

                if ui
                    .add_enabled(!self.fix.running(), egui::Button::new("Check again"))
                    .clicked()
                {
                    self.requirements = Job::start(ui.ctx(), requirements::check);
                }

                ui.label(theme::muted(
                    "You can continue now and install it later; Tinkerbench will not start until then.",
                ));
            }
        }

        match &self.fix {
            Job::Running(_) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Installing…");
                });
            }
            Job::Done(Err(e)) => {
                ui.label(theme::error(e));
            }
            Job::Done(Ok(log)) if self.dry_run => {
                ui.label(theme::path(log));
            }
            _ => {}
        }
    }

    fn game(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Warcraft III"));
        ui.label("Tinkerbench reads the units, abilities and art from your own game install.");

        if self.games.is_empty() {
            ui.label(theme::muted(
                "No install was found in the usual places. Choose the folder the game is installed in.",
            ));
        } else {
            ui.label(theme::muted("Found on this computer:"));

            for game in &self.games {
                let dir = text(Some(&game.dir));
                let mut summary = game.source.label().to_owned();

                if let Some(version) = &game.version {
                    summary.push_str(&format!(" · version {version}"));
                }

                if game.maps_dir.is_none() {
                    summary.push_str(" · no Maps folder yet");
                }

                ui.radio_value(&mut self.game_dir, dir.clone(), summary);
                ui.label(theme::path(&dir));
            }
        }

        folder_field(ui, "Game folder", &mut self.game_dir);

        match self.game_check.done() {
            _ if self.game_check.running() => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Reading the game data…");
                });
            }
            Some((dir, Ok(check))) if dir == self.game_dir.trim() => {
                let version = check
                    .version
                    .clone()
                    .unwrap_or_else(|| "unknown".to_owned());

                ui.label(theme::done(&format!(
                    "The game data reads (version {version})."
                )));

                let hd_art = check.hd_art;

                ui.add_enabled(
                    hd_art,
                    egui::Checkbox::new(&mut self.hd, "Use HD (Reforged) art values and icons"),
                );

                if !hd_art {
                    self.hd = false;
                    ui.label(theme::muted(
                        "HD art is not installed, so Tinkerbench shows the classic art.",
                    ));
                }
            }
            Some((dir, Err(e))) if dir == self.game_dir.trim() => {
                ui.label(theme::error(e));
            }
            _ => {}
        }
    }

    fn maps(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Maps folder"));
        ui.label(
            "Where your maps are: Open map starts here. Warcraft III creates this folder the \
             first time it runs.",
        );
        folder_field(ui, "Maps folder", &mut self.maps_dir);
        ui.label(theme::muted(
            "Leave it empty to choose each time. Test copies of maps are saved beside the map, \
             so Tinkerbench must be able to write here.",
        ));
    }

    fn test_maps(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Test maps"));
        ui.label(
            "Save and build test copy writes your Lua modules into a folder, then builds a \
             \"[DEV]\" copy of the map next to it, ready to play. Test maps must use Lua.",
        );
        ui.radio_value(
            &mut self.shared_lua,
            false,
            "A Lua folder next to each map (recommended)",
        );
        ui.label(theme::faint("For example My Map Lua beside My Map.w3x."));
        ui.radio_value(&mut self.shared_lua, true, "One Lua folder for all maps");

        if self.shared_lua {
            folder_field(ui, "Lua folder", &mut self.lua_folder);
        }
    }

    fn install_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Install"));

        let plan = self.plan();
        let lua = match &plan.choices.lua_folder {
            Some(folder) => folder.display().to_string(),
            None => "next to each map".to_owned(),
        };
        let maps = match &plan.choices.maps_dir {
            Some(folder) => folder.display().to_string(),
            None => "chosen each time".to_owned(),
        };

        for (name, value) in [
            ("Install in", plan.dir.display().to_string()),
            ("Warcraft III", plan.choices.game_dir.display().to_string()),
            ("Maps", maps),
            ("Lua modules", lua),
        ] {
            ui.label(theme::strong(name));
            ui.label(theme::path(&value));
        }

        if self.install_events.is_some() {
            ui.add_space(8.0);

            for step in &self.install_log {
                ui.label(theme::muted(&format!("{step}… done")));
            }

            match &self.installed {
                None => {
                    ui.spinner();
                }
                Some(Err(e)) => {
                    ui.label(theme::error(e));
                }
                Some(Ok(_)) => {}
            }
        }
    }

    fn finish_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Ready"));
        ui.label(theme::done("Tinkerbench is installed."));

        if cfg!(windows) {
            ui.label("Find it in the Start menu.");
        } else {
            ui.label("Find it in your applications menu, or run tinkerbench in a terminal.");
        }

        ui.label(theme::muted(
            "It opens your game directly. To update or remove it later, run this setup again.",
        ));
        ui.checkbox(&mut self.launch, "Start Tinkerbench now");
    }

    fn remove_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::heading("Remove"));

        let Some(record) = &self.existing else {
            ui.label("Tinkerbench Setup has not installed Tinkerbench for this user.");
            return;
        };

        ui.label(format!("Remove Tinkerbench {} from:", record.version));
        ui.label(theme::path(&record.dir.display().to_string()));
        ui.checkbox(
            &mut self.remove_settings,
            "Also remove Tinkerbench's settings and saved data",
        );
        ui.label(theme::muted("Your maps are never touched."));

        match &self.removal {
            Job::Running(_) => {
                ui.spinner();
            }
            Job::Done(Err(e)) => {
                ui.label(theme::error(e));
            }
            _ => {}
        }
    }
}

fn license(ui: &mut egui::Ui) {
    ui.label(theme::heading("License"));
    ui.label("Tinkerbench is free and open source, under the MIT License:");
    ScrollArea::vertical()
        .id_salt("license")
        .max_height(LICENSE_HEIGHT)
        .show(ui, |ui| ui.label(theme::path(payload::LICENSE)));
    ui.collapsing("Third-party notices", |ui| {
        ui.label(theme::muted(
            "Tinkerbench and this setup include the open-source software below.",
        ));

        let lines: Vec<&str> = payload::SETUP_NOTICES.lines().collect();
        let row_height = ui.text_style_height(&egui::TextStyle::Small);

        ScrollArea::vertical()
            .id_salt("notices")
            .max_height(NOTICES_HEIGHT)
            .show_rows(ui, row_height, lines.len(), |ui, rows| {
                for line in &lines[rows] {
                    ui.label(theme::path(line));
                }
            });
    });
}

fn removed(ui: &mut egui::Ui) {
    ui.label(theme::heading("Removed"));
    ui.label(theme::done("Tinkerbench is removed."));
}

/// A labelled path field with Browse….
fn folder_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    let name = ui.label(theme::strong(label));

    ui.horizontal(|ui| {
        let browse_width = 90.0;
        let field = TextEdit::singleline(value)
            .font(egui::TextStyle::Monospace)
            .desired_width(ui.available_width() - browse_width);

        ui.add(field).labelled_by(name.id);

        if ui.button("Browse…").clicked()
            && let Some(picked) = pick_folder(value)
        {
            *value = picked;
        }
    });
}

impl eframe::App for SetupApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.poll(&ctx);

        egui::Panel::left("steps")
            .exact_size(STEPS_WIDTH)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(theme::SURFACE_0)
                    .inner_margin(Margin::same(SIDE_MARGIN)),
            )
            .show(ui, |ui| self.step_list(ui));
        egui::Panel::bottom("navigation")
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(theme::SURFACE_0)
                    .inner_margin(Margin::same(NAV_MARGIN)),
            )
            .show(ui, |ui| self.navigation(ui));
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(theme::SURFACE_1)
                    .inner_margin(Margin::same(PAGE_MARGIN)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt("page")
                    .show(ui, |ui| self.page(ui));
            });

        if let Some(id) = ctx.memory(|m| m.focused())
            && let Some(focused) = ctx.read_response(id)
        {
            ui.painter().rect_stroke(
                focused.rect.expand(FOCUS_GAP),
                theme::RADIUS,
                Stroke::new(FOCUS_WIDTH, theme::FOCUS),
                StrokeKind::Outside,
            );
        }

        ui.painter().rect_stroke(
            ui.max_rect().shrink(FRAME_INSET),
            0.0,
            Stroke::new(1.0, theme::FRAME),
            StrokeKind::Inside,
        );
    }
}
