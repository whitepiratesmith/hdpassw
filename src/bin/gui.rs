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
                ui.label("No seed vault found.");
                ui.label("Run `hdpassw seed new` (or `seed restore`) on the command line first.");
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
        ui.horizontal(|ui| {
            ui.heading("Sites");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Lock").clicked() {
                    self.session = None;
                    self.sites = SiteStore::default();
                    self.show_add_form = false;
                    self.status.clear();
                }
                let add_label = if self.show_add_form { "Cancel" } else { "+ Add site" };
                if ui.button(add_label).clicked() {
                    self.show_add_form = !self.show_add_form;
                }
            });
        });
        ui.add_space(8.0);

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

        let session = self.session.as_ref().expect("unlocked");
        let mut to_copy: Option<(String, String, u32, usize, String)> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (site, record) in &self.sites.sites {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(site).strong().size(15.0));
                                ui.label(
                                    egui::RichText::new(&record.user)
                                        .color(egui::Color32::GRAY),
                                );
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
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
            }
        });

        if let Some((site, user, counter, length, charset_str)) = to_copy {
            match generate_and_copy(session, &site, &user, counter, length, &charset_str) {
                Ok(()) => self.set_status(format!("✓ Copied password for {site} — clears in 30s"), false),
                Err(e) => self.set_status(format!("✗ {e}"), true),
            }
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
            if n.is_empty() { None } else { Some(n.to_string()) }
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
