//! Minimal egui prototype for hdpassw.
//!
//! Unlocks the encrypted seed vault, then lists known sites, lets you
//! generate + copy a password for any of them, and add new site entries.
//! Pure Rust, no webview — this binary links directly against the same
//! crypto/session/store/manager code as the CLI via the `hdpassw` library.

use std::fs;
use std::path::PathBuf;

use eframe::egui;
use zeroize::Zeroizing;

use hdpassw::crypto::{self, encode::Charset};
use hdpassw::session::Session;
use hdpassw::store::{self, SiteStore};
use hdpassw::{clipboard, manager};

const CHARSETS: [Charset; 5] = [
    Charset::Alpha,
    Charset::Alphanumeric,
    Charset::Full,
    Charset::Pin,
    Charset::Hex,
];

/// Which screen of the "no vault yet" setup wizard is showing.
enum SetupStage {
    /// Choose between generating a new seed or restoring an existing one.
    Choice,
    /// Freshly generated seed phrase, shown once for the user to write down.
    ShowNew {
        phrase: Zeroizing<String>,
        written_down: bool,
    },
    /// Quiz the user on a handful of random words from a freshly generated
    /// phrase, to catch transcription mistakes before they're locked in.
    Verify {
        phrase: Zeroizing<String>,
        /// 0-indexed positions into the word list, ascending.
        positions: Vec<usize>,
        answers: Vec<String>,
        error: String,
    },
    /// Free-text entry for restoring an existing seed phrase.
    Restore { input: String, error: String },
    /// Set the vault password that will encrypt `phrase` on disk.
    SetPassword {
        phrase: Zeroizing<String>,
        pw1: Zeroizing<String>,
        pw2: Zeroizing<String>,
        /// The screen to return to on "Back" — whichever led here.
        previous: Box<SetupStage>,
    },
}

struct SetupState {
    stage: SetupStage,
}

/// Lay out a row of `count` equally-sized buttons and have it genuinely
/// centered in `ui`. Plain `ui.horizontal` shrink-wraps to the buttons'
/// actual width and anchors flush-left, ignoring the surrounding centered
/// layout — `allocate_ui_with_layout` with an *explicit* width is the only
/// way egui centers a block, so the row's exact width is computed up front.
fn centered_button_row(
    ui: &mut egui::Ui,
    button_size: egui::Vec2,
    count: usize,
    spacing: f32,
    add_contents: impl FnOnce(&mut egui::Ui, egui::Vec2),
) {
    let row_width = count as f32 * button_size.x + (count as f32 - 1.0) * spacing;
    ui.allocate_ui_with_layout(
        egui::vec2(row_width, ui.available_height()),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;
                add_contents(ui, button_size);
            });
        },
    );
}

/// BIP39 English words starting with `prefix`, or empty if `prefix` is too
/// short to be useful or already an exact word (nothing left to suggest).
fn word_suggestions(prefix: &str) -> Vec<&'static str> {
    if prefix.len() < 2 || bip39::Language::English.find_word(prefix).is_some() {
        return Vec::new();
    }
    bip39::Language::English
        .words_by_prefix(prefix)
        .iter()
        .copied()
        .take(6)
        .collect()
}

/// Render clickable suggestion chips for `prefix`. Returns the picked word,
/// if any, so the caller can write it back into its own text buffer.
fn suggestion_chips(ui: &mut egui::Ui, prefix: &str) -> Option<&'static str> {
    let suggestions = word_suggestions(prefix);
    if suggestions.is_empty() {
        return None;
    }
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        for word in suggestions {
            if ui.small_button(word).clicked() {
                picked = Some(word);
            }
        }
    });
    picked
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([460.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "hdpassw",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

struct App {
    /// Vault password entered by the user. Zeroed on drop and after use.
    password: Zeroizing<String>,
    /// Live session holding the derived master key, once unlocked.
    session: Option<Session>,
    /// Site metadata, loaded after unlock.
    sites: SiteStore,
    /// Where `sites` is persisted.
    db_path: PathBuf,
    /// Status line shown at the bottom of the window.
    status: String,
    /// Whether `status` represents an error (shown in red) or success (green).
    status_is_error: bool,

    /// First-run wizard state, active when no vault exists yet.
    setup: Option<SetupState>,

    /// In-progress password change for one site: (site, old_counter,
    /// new_counter, step). Step 0 = old password not yet copied,
    /// step 1 = old password copied, waiting to copy the new one and save.
    bump_in_progress: Option<(String, u32, u32, u8)>,

    /// "Add site" form state.
    show_add_form: bool,
    new_site: String,
    new_user: String,
    new_counter: String,
    new_length: String,
    new_charset: Charset,
    new_notes: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            password: Zeroizing::default(),
            session: None,
            sites: SiteStore::default(),
            db_path: store::default_db_path(),
            status: String::new(),
            status_is_error: false,
            setup: None,
            bump_in_progress: None,
            show_add_form: false,
            new_site: String::new(),
            new_user: String::new(),
            new_counter: "1".to_string(),
            new_length: "32".to_string(),
            new_charset: Charset::default(),
            new_notes: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);

            if self.session.is_none() {
                self.unlock_screen(ui);
            } else {
                self.main_screen(ui);
            }

            if !self.status.is_empty() {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                let color = if self.status_is_error {
                    egui::Color32::from_rgb(220, 80, 80)
                } else {
                    egui::Color32::from_rgb(90, 180, 100)
                };
                ui.label(egui::RichText::new(&self.status).color(color));
            }
        });
    }
}

impl App {
    fn set_status(&mut self, text: impl Into<String>, is_error: bool) {
        self.status = text.into();
        self.status_is_error = is_error;
    }

    fn unlock_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading(egui::RichText::new("hdpassw").size(32.0).strong());
            ui.label("Deterministic password manager");
            ui.add_space(32.0);

            let vault = store::vault_path();
            if !vault.exists() {
                self.setup_wizard(ui, &vault);
                return;
            }

            ui.set_max_width(260.0);
            ui.label("Vault password");
            let resp = ui.add(egui::TextEdit::singleline(&mut *self.password).password(true));
            ui.add_space(8.0);

            let unlock_clicked = ui.button("Unlock").clicked();

            if unlock_clicked
                || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.try_unlock(&vault);
            }
        });
    }

    /// First-run wizard: create a new seed phrase or restore an existing
    /// one, then encrypt it into a vault file — all without touching the CLI.
    fn setup_wizard(&mut self, ui: &mut egui::Ui, vault: &std::path::Path) {
        let setup = self.setup.get_or_insert(SetupState {
            stage: SetupStage::Choice,
        });

        let mut next_stage: Option<SetupStage> = None;
        let mut finish: Option<(Zeroizing<String>, String)> = None;

        // `allocate_ui_with_layout` reserves a fixed-width block and lets the
        // parent's centered layout position it — unlike `set_max_width`,
        // which keeps the left edge fixed and just shrinks the right side.
        ui.allocate_ui_with_layout(
            egui::vec2(360.0, ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.label("No seed vault found yet.");
                ui.add_space(16.0);

                match &mut setup.stage {
                    SetupStage::Choice => {
                        centered_button_row(ui, egui::vec2(170.0, 30.0), 2, 10.0, |ui, size| {
                            if ui
                                .add_sized(size, egui::Button::new("Create new seed"))
                                .clicked()
                            {
                                let phrase = Zeroizing::new(crypto::generate().to_string());
                                next_stage = Some(SetupStage::ShowNew {
                                    phrase,
                                    written_down: false,
                                });
                            }
                            if ui
                                .add_sized(size, egui::Button::new("Restore existing seed"))
                                .clicked()
                            {
                                next_stage = Some(SetupStage::Restore {
                                    input: String::new(),
                                    error: String::new(),
                                });
                            }
                        });
                    }
                    SetupStage::ShowNew {
                        phrase,
                        written_down,
                    } => {
                        let words: Vec<&str> = phrase.split_whitespace().collect();
                        ui.label(format!("Your seed phrase ({} words):", words.len()));
                        ui.add_space(8.0);

                        // `Frame`/`Grid` always shrink-wrap to their content and
                        // anchor to the left of whatever space they're given —
                        // they ignore the surrounding `Align::Center` layout.
                        // The only way egui actually centers a block is via
                        // `allocate_ui_with_layout` with an *explicit* size, so
                        // we fix the grid's column width and compute its exact
                        // total width up front to hand in here.
                        const GRID_COLS: usize = 3;
                        const COL_WIDTH: f32 = 95.0;
                        const COL_SPACING: f32 = 10.0;
                        let grid_width =
                            GRID_COLS as f32 * COL_WIDTH + (GRID_COLS as f32 - 1.0) * COL_SPACING;

                        ui.allocate_ui_with_layout(
                            egui::vec2(grid_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    egui::Grid::new("seed_words_grid")
                                        .num_columns(GRID_COLS)
                                        .min_col_width(COL_WIDTH)
                                        .spacing([COL_SPACING, 4.0])
                                        .show(ui, |ui| {
                                            for (i, chunk) in words.chunks(GRID_COLS).enumerate() {
                                                for (j, w) in chunk.iter().enumerate() {
                                                    ui.label(format!(
                                                        "{:2}. {}",
                                                        i * GRID_COLS + j + 1,
                                                        w
                                                    ));
                                                }
                                                ui.end_row();
                                            }
                                        });
                                });
                            },
                        );

                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(
                                "Write this down and store it offline. Never share it.",
                            )
                            .color(egui::Color32::from_rgb(230, 180, 60)),
                        );
                        ui.add_space(8.0);
                        ui.checkbox(written_down, "I have written this down");
                        ui.add_space(8.0);

                        centered_button_row(ui, egui::vec2(110.0, 30.0), 2, 10.0, |ui, size| {
                            if ui.add_sized(size, egui::Button::new("Back")).clicked() {
                                next_stage = Some(SetupStage::Choice);
                            }
                            ui.add_enabled_ui(*written_down, |ui| {
                                if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                    let word_count = words.len();
                                    let quiz_size = 6.min(word_count);
                                    let mut positions = rand::seq::index::sample(
                                        &mut rand::thread_rng(),
                                        word_count,
                                        quiz_size,
                                    )
                                    .into_vec();
                                    positions.sort_unstable();

                                    next_stage = Some(SetupStage::Verify {
                                        phrase: phrase.clone(),
                                        positions,
                                        answers: vec![String::new(); quiz_size],
                                        error: String::new(),
                                    });
                                }
                            });
                        });
                    }
                    SetupStage::Verify {
                        phrase,
                        positions,
                        answers,
                        error,
                    } => {
                        let words: Vec<&str> = phrase.split_whitespace().collect();
                        ui.label("Quick check — enter these words from your seed phrase:");
                        ui.add_space(10.0);

                        const LABEL_WIDTH: f32 = 90.0;
                        const INPUT_WIDTH: f32 = 110.0;
                        const SPACING: f32 = 10.0;
                        let grid_width = LABEL_WIDTH + SPACING + INPUT_WIDTH;

                        ui.allocate_ui_with_layout(
                            egui::vec2(grid_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    egui::Grid::new("verify_grid")
                                        .num_columns(2)
                                        .min_col_width(LABEL_WIDTH)
                                        .spacing([SPACING, 6.0])
                                        .show(ui, |ui| {
                                            for (slot, &word_idx) in positions.iter().enumerate() {
                                                ui.label(format!("Word {}:", word_idx + 1));
                                                ui.vertical(|ui| {
                                                    ui.add(
                                                        egui::TextEdit::singleline(
                                                            &mut answers[slot],
                                                        )
                                                        .desired_width(INPUT_WIDTH),
                                                    );
                                                    let lower = answers[slot].trim().to_lowercase();
                                                    if let Some(picked) =
                                                        suggestion_chips(ui, &lower)
                                                    {
                                                        answers[slot] = picked.to_string();
                                                    }
                                                });
                                                ui.end_row();
                                            }
                                        });
                                });
                            },
                        );

                        ui.add_space(8.0);
                        if !error.is_empty() {
                            ui.label(
                                egui::RichText::new(error.as_str())
                                    .color(egui::Color32::from_rgb(220, 80, 80)),
                            );
                            ui.add_space(6.0);
                        }

                        centered_button_row(ui, egui::vec2(110.0, 30.0), 2, 10.0, |ui, size| {
                            if ui.add_sized(size, egui::Button::new("Back")).clicked() {
                                next_stage = Some(SetupStage::ShowNew {
                                    phrase: phrase.clone(),
                                    written_down: true,
                                });
                            }
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                let all_correct = positions
                                    .iter()
                                    .zip(answers.iter())
                                    .all(|(&idx, ans)| ans.trim().to_lowercase() == words[idx]);

                                if all_correct {
                                    next_stage = Some(SetupStage::SetPassword {
                                        phrase: phrase.clone(),
                                        pw1: Zeroizing::default(),
                                        pw2: Zeroizing::default(),
                                        previous: Box::new(SetupStage::Verify {
                                            phrase: phrase.clone(),
                                            positions: positions.clone(),
                                            answers: answers.clone(),
                                            error: String::new(),
                                        }),
                                    });
                                } else {
                                    *error = "✗ One or more words don't match. Check your seed \
                                               phrase and try again."
                                        .to_string();
                                }
                            }
                        });
                    }
                    SetupStage::Restore { input, error } => {
                        ui.label("Enter all words separated by spaces:");
                        ui.add_space(6.0);
                        ui.add(egui::TextEdit::multiline(input).desired_rows(3));

                        // Suggest completions for whichever word is still
                        // being typed (the last one, as long as the user
                        // hasn't already finished it with a trailing space).
                        let ends_with_space = input.chars().last().is_some_and(char::is_whitespace);
                        if !ends_with_space {
                            let last_word = input.split_whitespace().last().unwrap_or("");
                            let lower = last_word.to_lowercase();
                            if let Some(picked) = suggestion_chips(ui, &lower) {
                                let mut words: Vec<String> =
                                    input.split_whitespace().map(str::to_lowercase).collect();
                                match words.last_mut() {
                                    Some(last) => *last = picked.to_string(),
                                    None => words.push(picked.to_string()),
                                }
                                *input = words.join(" ");
                                input.push(' ');
                            }
                        }
                        ui.add_space(8.0);

                        if !error.is_empty() {
                            ui.label(
                                egui::RichText::new(error.as_str())
                                    .color(egui::Color32::from_rgb(220, 80, 80)),
                            );
                            ui.add_space(6.0);
                        }

                        centered_button_row(ui, egui::vec2(110.0, 30.0), 2, 10.0, |ui, size| {
                            if ui.add_sized(size, egui::Button::new("Back")).clicked() {
                                next_stage = Some(SetupStage::Choice);
                            }
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                let normalized = input
                                    .split_whitespace()
                                    .map(|w| w.to_lowercase())
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                match crypto::parse(&normalized) {
                                    Ok(_) => {
                                        next_stage = Some(SetupStage::SetPassword {
                                            phrase: Zeroizing::new(normalized),
                                            pw1: Zeroizing::default(),
                                            pw2: Zeroizing::default(),
                                            previous: Box::new(SetupStage::Restore {
                                                input: input.clone(),
                                                error: String::new(),
                                            }),
                                        });
                                    }
                                    Err(e) => *error = format!("✗ {e}"),
                                }
                            }
                        });
                    }
                    SetupStage::SetPassword {
                        phrase,
                        pw1,
                        pw2,
                        previous,
                    } => {
                        ui.label("Choose a vault password:");
                        ui.add_space(6.0);
                        ui.label("Password");
                        ui.add(egui::TextEdit::singleline(&mut **pw1).password(true));
                        ui.add_space(4.0);
                        ui.label("Confirm password");
                        ui.add(egui::TextEdit::singleline(&mut **pw2).password(true));
                        ui.add_space(10.0);

                        let mut save_clicked = false;
                        centered_button_row(ui, egui::vec2(150.0, 30.0), 2, 10.0, |ui, size| {
                            if ui.add_sized(size, egui::Button::new("Back")).clicked() {
                                next_stage =
                                    Some(std::mem::replace(previous.as_mut(), SetupStage::Choice));
                            }
                            if ui
                                .add_sized(size, egui::Button::new("Save vault & unlock"))
                                .clicked()
                            {
                                save_clicked = true;
                            }
                        });

                        if save_clicked {
                            if pw1.is_empty() {
                                self.status = "✗ Password cannot be empty".to_string();
                                self.status_is_error = true;
                            } else if pw1 != pw2 {
                                self.status = "✗ Passwords do not match".to_string();
                                self.status_is_error = true;
                            } else {
                                finish = Some((phrase.clone(), pw1.to_string()));
                            }
                        }
                    }
                }
            },
        );

        if let Some(stage) = next_stage {
            setup.stage = stage;
        }

        if let Some((phrase, password)) = finish {
            self.complete_setup(vault, &phrase, &password);
        }
    }

    /// Encrypt `phrase` into the vault file at `vault`, then unlock a
    /// session directly so the user lands on the main screen.
    fn complete_setup(&mut self, vault: &std::path::Path, phrase: &str, password: &str) {
        let result = crypto::vault_seal(phrase, password).and_then(|data| {
            if let Some(parent) = vault.parent() {
                fs::create_dir_all(parent).map_err(hdpassw::error::Error::Io)?;
            }
            fs::write(vault, data).map_err(hdpassw::error::Error::Io)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(vault, fs::Permissions::from_mode(0o600))
                    .map_err(hdpassw::error::Error::Io)?;
            }

            Session::unlock(phrase, None)
        });

        match result {
            Ok(session) => {
                self.session = Some(session);
                self.sites = store::load(&self.db_path).unwrap_or_default();
                self.setup = None;
                self.set_status("✓ Vault created and unlocked", false);
            }
            Err(e) => self.set_status(format!("✗ {e}"), true),
        }
    }

    fn try_unlock(&mut self, vault: &std::path::Path) {
        let result = fs::read(vault)
            .map_err(hdpassw::error::Error::Io)
            .and_then(|data| crypto::vault_open(&data, &self.password))
            .and_then(|phrase| Session::unlock(&phrase, None));

        self.password = Zeroizing::default();

        match result {
            Ok(session) => {
                self.session = Some(session);
                self.sites = store::load(&self.db_path).unwrap_or_default();
                self.status.clear();
            }
            Err(e) => self.set_status(format!("✗ {e}"), true),
        }
    }

    fn main_screen(&mut self, ui: &mut egui::Ui) {
        let mut run_pwned_check = false;
        let mut run_rotate = false;

        ui.horizontal(|ui| {
            ui.heading("Sites");
            ui.label(
                egui::RichText::new(format!("rotation /{}", self.sites.rotation))
                    .color(egui::Color32::GRAY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Lock").clicked() {
                    self.session = None;
                    self.sites = SiteStore::default();
                    self.show_add_form = false;
                    self.status.clear();
                }
                let add_label = if self.show_add_form {
                    "Cancel"
                } else {
                    "+ Add site"
                };
                if ui.button(add_label).clicked() {
                    self.show_add_form = !self.show_add_form;
                }
                if ui
                    .add_enabled(!self.sites.sites.is_empty(), egui::Button::new("Test if pwned"))
                    .clicked()
                {
                    run_pwned_check = true;
                }
                if ui
                    .button("Rotate")
                    .on_hover_text(
                        "Bump the global rotation counter only — sites are flagged behind, \
                         not changed. Use \"Bump\" on a site to actually change its password.",
                    )
                    .clicked()
                {
                    run_rotate = true;
                }
            });
        });
        ui.add_space(8.0);

        if run_rotate {
            self.rotate_global();
        }

        if self.show_add_form {
            self.add_form(ui);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
        }

        if self.sites.sites.is_empty() {
            ui.label("No sites saved yet. Click \"+ Add site\" to create one.");
            return;
        }

        let mut to_copy: Option<(String, String, u32, usize, String)> = None;
        let mut start_bump: Option<(String, u32, u32)> = None;
        let mut advance_bump = false;
        let mut cancel_bump = false;
        let rotation = self.sites.rotation;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (site, record) in &self.sites.sites {
                let behind = record.counter < rotation;
                let in_progress = self
                    .bump_in_progress
                    .as_ref()
                    .filter(|(s, ..)| s == site)
                    .map(|(_, _, _, step)| *step);

                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(site).strong().size(15.0));
                                ui.label(
                                    egui::RichText::new(&record.user).color(egui::Color32::GRAY),
                                );
                                if behind {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "⚠ /{} → /{rotation}",
                                            record.counter
                                        ))
                                        .color(egui::Color32::ORANGE),
                                    );
                                }
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Generate & copy").clicked() {
                                        to_copy = Some((
                                            site.clone(),
                                            record.user.clone(),
                                            record.counter,
                                            record.length,
                                            record.charset.clone(),
                                        ));
                                    }
                                    match in_progress {
                                        Some(0) => {
                                            if ui.button("Copy old password").clicked() {
                                                advance_bump = true;
                                            }
                                            if ui.button("Cancel").clicked() {
                                                cancel_bump = true;
                                            }
                                        }
                                        Some(_) => {
                                            if ui.button("Copy new password & save").clicked() {
                                                advance_bump = true;
                                            }
                                            if ui.button("Cancel").clicked() {
                                                cancel_bump = true;
                                            }
                                        }
                                        None if behind => {
                                            if ui.button("Bump").clicked() {
                                                let new = if record.counter < rotation {
                                                    rotation
                                                } else {
                                                    record.counter + 1
                                                };
                                                start_bump =
                                                    Some((site.clone(), record.counter, new));
                                            }
                                        }
                                        None => {}
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
            }
        });

        if let Some((site, user, counter, length, charset_str)) = to_copy {
            let session = self.session.as_ref().expect("unlocked");
            let result = generate_and_copy(session, &site, &user, counter, length, &charset_str);
            match result {
                Ok(()) => self.set_status(
                    format!("✓ Copied password for {site} — clears in 30s"),
                    false,
                ),
                Err(e) => self.set_status(format!("✗ {e}"), true),
            }
        }

        if let Some((site, old, new)) = start_bump {
            self.bump_in_progress = Some((site, old, new, 0));
        }

        if advance_bump {
            self.advance_bump();
        }

        if cancel_bump {
            self.bump_in_progress = None;
            self.set_status("Bump cancelled — nothing saved", false);
        }

        if run_pwned_check {
            self.check_all_pwned();
        }
    }

    /// Advances an in-progress password change one step.
    ///
    /// Step 0 → 1: copies the OLD password (current counter) so the user
    /// can log in and start the change on the site.
    /// Step 1 → done: copies the NEW password (target counter) and only
    /// then persists the bumped counter and recomputed verifier to disk.
    /// If either step fails, `bump_in_progress` is left as-is so the user
    /// can retry without losing their place.
    fn advance_bump(&mut self) {
        let Some((site, old, new, step)) = self.bump_in_progress.clone() else {
            return;
        };
        let Some(record) = self.sites.sites.get(&site).cloned() else {
            self.bump_in_progress = None;
            return;
        };
        let charset = match Charset::from_str(&record.charset) {
            Ok(c) => c,
            Err(e) => return self.set_status(format!("✗ {e}"), true),
        };

        if step == 0 {
            let password_result = {
                let session = self.session.as_ref().expect("unlocked");
                manager::generate_password(session, &site, &record.user, old, record.length, charset)
            };
            let password = match password_result {
                Ok(p) => Zeroizing::new(p),
                Err(e) => return self.set_status(format!("✗ {e}"), true),
            };
            if let Err(e) = clipboard::copy_and_clear(&password, 30) {
                return self.set_status(format!("✗ {e}"), true);
            }
            self.bump_in_progress = Some((site.clone(), old, new, 1));
            self.set_status(
                format!(
                    "✓ Copied OLD password for {site} — log in, then click \"Copy new password & save\""
                ),
                false,
            );
            return;
        }

        let new_password_result = {
            let session = self.session.as_ref().expect("unlocked");
            manager::generate_password(session, &site, &record.user, new, record.length, charset)
        };
        let new_password = match new_password_result {
            Ok(p) => Zeroizing::new(p),
            Err(e) => return self.set_status(format!("✗ {e}"), true),
        };
        if let Err(e) = clipboard::copy_and_clear(&new_password, 30) {
            return self.set_status(format!("✗ {e}"), true);
        }

        let upsert_result = {
            let session = self.session.as_ref().expect("unlocked");
            manager::upsert_site(
                &mut self.sites,
                session,
                &site,
                &record.user,
                new,
                record.length,
                charset,
                record.notes.clone(),
            )
        };
        if let Err(e) = upsert_result {
            return self.set_status(format!("✗ {e}"), true);
        }
        if let Err(e) = store::save(&self.db_path, &self.sites) {
            return self.set_status(format!("✗ {e}"), true);
        }

        self.bump_in_progress = None;
        self.set_status(
            format!("✓ Saved: {site} is now at /{new} — new password copied, paste it on the site"),
            false,
        );
    }

    /// Bump the global rotation counter only — no site's own counter or
    /// password is touched. Sites whose counter is now behind are flagged
    /// in the list; use the per-site "Bump" button to actually change one.
    fn rotate_global(&mut self) {
        let old = self.sites.rotation;
        let new = old + 1;
        self.sites.rotation = new;
        self.sites.rotation_since = chrono::Local::now().date_naive();

        if let Err(e) = store::save(&self.db_path, &self.sites) {
            self.sites.rotation = old;
            return self.set_status(format!("✗ {e}"), true);
        }

        let stale = self.sites.sites.values().filter(|r| r.counter < new).count();
        self.set_status(
            if stale == 0 {
                format!("✓ Rotation /{old} → /{new} — every site is already caught up")
            } else {
                format!(
                    "✓ Rotation /{old} → /{new} — {stale} site(s) now behind, use \"Bump\" to change them"
                )
            },
            false,
        );
    }

    /// Derive every stored site's password and check it against Have I
    /// Been Pwned, then summarize results in the status line. Passwords
    /// are held only long enough to hash and check, never displayed.
    fn check_all_pwned(&mut self) {
        let session = self.session.as_ref().expect("unlocked");
        let sites: Vec<_> = self
            .sites
            .sites
            .iter()
            .map(|(site, record)| {
                (
                    site.clone(),
                    record.user.clone(),
                    record.counter,
                    record.length,
                    record.charset.clone(),
                )
            })
            .collect();

        let mut breached = Vec::new();
        for (site, user, counter, length, charset_str) in &sites {
            match check_pwned(session, site, user, *counter, *length, charset_str) {
                Ok(hdpassw::pwned::PwnedStatus::Safe) => {}
                Ok(hdpassw::pwned::PwnedStatus::Pwned(count)) => {
                    breached.push(format!("{site} ({count})"));
                }
                Err(e) => {
                    self.set_status(format!("✗ {e}"), true);
                    return;
                }
            }
        }

        if breached.is_empty() {
            self.set_status(
                format!("✓ Checked {} site(s) — none found in known breaches", sites.len()),
                false,
            );
        } else {
            self.set_status(
                format!(
                    "⚠ {} of {} password(s) found in known breaches: {}",
                    breached.len(),
                    sites.len(),
                    breached.join(", ")
                ),
                true,
            );
        }
    }

    /// The "add new site" form. Lets you register a site so a password can
    /// be generated for it (and re-generated identically later).
    fn add_form(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::symmetric(10, 10))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("New site").strong());
                ui.add_space(6.0);

                egui::Grid::new("add_site_grid")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Site");
                        ui.text_edit_singleline(&mut self.new_site);
                        ui.end_row();

                        ui.label("User");
                        ui.text_edit_singleline(&mut self.new_user);
                        ui.end_row();

                        ui.label("Counter");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_counter).desired_width(60.0),
                        );
                        ui.end_row();

                        ui.label("Length");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_length).desired_width(60.0),
                        );
                        ui.end_row();

                        ui.label("Charset");
                        egui::ComboBox::from_id_salt("charset")
                            .selected_text(self.new_charset.as_str())
                            .show_ui(ui, |ui| {
                                for charset in CHARSETS {
                                    ui.selectable_value(
                                        &mut self.new_charset,
                                        charset,
                                        charset.as_str(),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label("Notes");
                        ui.text_edit_singleline(&mut self.new_notes);
                        ui.end_row();
                    });

                if self.new_length.trim() != "32" || self.new_charset != Charset::default() {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(
                            "⚠ Length and charset feed into the derived password, but \
                             aren't part of your seed phrase. If you change them from \
                             the defaults, you'll need to remember these exact settings \
                             (or keep this sites file) to regenerate this password later.",
                        )
                        .color(egui::Color32::from_rgb(230, 180, 60))
                        .small(),
                    );
                }

                ui.add_space(8.0);
                if ui.button("Save").clicked() {
                    self.save_new_site();
                }
            });
    }

    fn save_new_site(&mut self) {
        let site = self.new_site.trim().to_string();
        let user = self.new_user.trim().to_string();

        if site.is_empty() {
            self.set_status("✗ Site name cannot be empty", true);
            return;
        }

        let counter: u32 = match self.new_counter.trim().parse() {
            Ok(n) => n,
            Err(_) => return self.set_status("✗ Counter must be a positive integer", true),
        };
        let length: usize = match self.new_length.trim().parse() {
            Ok(n) => n,
            Err(_) => return self.set_status("✗ Length must be a positive integer", true),
        };
        let notes = {
            let n = self.new_notes.trim();
            if n.is_empty() {
                None
            } else {
                Some(n.to_string())
            }
        };

        let session = self.session.as_ref().expect("unlocked");
        let result = manager::upsert_site(
            &mut self.sites,
            session,
            &site,
            &user,
            counter,
            length,
            self.new_charset,
            notes,
        )
        .and_then(|()| store::save(&self.db_path, &self.sites));

        match result {
            Ok(()) => {
                self.set_status(format!("✓ Added {site}"), false);
                self.new_site.clear();
                self.new_user.clear();
                self.new_counter = "1".to_string();
                self.new_length = "32".to_string();
                self.new_charset = Charset::default();
                self.new_notes.clear();
                self.show_add_form = false;
            }
            Err(e) => self.set_status(format!("✗ {e}"), true),
        }
    }
}

fn generate_and_copy(
    session: &Session,
    site: &str,
    user: &str,
    counter: u32,
    length: usize,
    charset_str: &str,
) -> hdpassw::error::Result<()> {
    let charset = Charset::from_str(charset_str)?;
    let password = manager::generate_password(session, site, user, counter, length, charset)?;
    clipboard::copy_and_clear(&password, 30)
}

/// Derive the password for `site` and check it against the Have I Been
/// Pwned range API. The password never leaves this function — only a
/// 5-character SHA-1 prefix is sent over the network.
fn check_pwned(
    session: &Session,
    site: &str,
    user: &str,
    counter: u32,
    length: usize,
    charset_str: &str,
) -> hdpassw::error::Result<hdpassw::pwned::PwnedStatus> {
    let charset = Charset::from_str(charset_str)?;
    let password = Zeroizing::new(manager::generate_password(
        session, site, user, counter, length, charset,
    )?);
    hdpassw::pwned::check(&password)
}
