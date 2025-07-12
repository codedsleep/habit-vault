use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, Toast, ToastOverlay, StyleManager};
use gtk4::{Button, Entry, Label, ListBox, ScrolledWindow, Orientation, MessageDialog, Dialog, DialogFlags, ResponseType, CssProvider, Switch, FileChooserDialog, FileChooserAction, FileFilter};
use gtk4::Box as GtkBox;
use gtk4::glib;
use crate::habit::{Habit, HabitData};
use crate::storage::SecureStorage;
use crate::calendar::HabitCalendar;
use std::rc::Rc;
use std::cell::RefCell;
use chrono::Utc;

pub struct HabitApp {
    window: ApplicationWindow,
    storage: SecureStorage,
    habit_data: Rc<RefCell<HabitData>>,
    password: Rc<RefCell<Option<String>>>,
    habit_list: ListBox,
    toast_overlay: ToastOverlay,
    style_manager: StyleManager,
    add_button: Button,
    settings_button: Button,
}

fn get_streak_emoji(streak: u32) -> &'static str {
    match streak {
        0..=2 => "😞",  // Unhappy face for 0-2 days
        3..=6 => "😊",  // Happy face for 3-6 days
        _ => "🔥",      // Flame for 7+ days
    }
}

impl HabitApp {
    pub fn new(app: &Application) -> Result<Self, std::boxed::Box<dyn std::error::Error>> {
        let storage = SecureStorage::new()?;
        
        let window = ApplicationWindow::builder()
            .application(app)
            .title("HabitVault")
            .default_width(800)
            .default_height(600)
            .build();

        let header_bar = HeaderBar::new();
        header_bar.set_title_widget(Some(&Label::new(Some("HabitVault"))));
        
        let add_button = Button::with_label("Add Habit");
        add_button.set_sensitive(false); // Disable until authenticated
        let settings_button = Button::with_label("⚙️");
        settings_button.set_tooltip_text(Some("Settings"));
        settings_button.add_css_class("header-icon-button");
        settings_button.set_sensitive(true); // Always enable settings access
        header_bar.pack_end(&settings_button);
        header_bar.pack_end(&add_button);
        
        // For AdwApplicationWindow, use set_content instead of set_titlebar
        // The header_bar will be added to the main content structure

        // Load custom CSS
        let css_provider = CssProvider::new();
        css_provider.load_from_data(include_str!("style.css"));
        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&window),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let toast_overlay = ToastOverlay::new();
        let content_box = GtkBox::new(Orientation::Vertical, 0);
        
        // Add header bar to content
        content_box.append(&header_bar);
        
        let main_box = GtkBox::new(Orientation::Vertical, 10);
        main_box.set_margin_top(10);
        main_box.set_margin_bottom(10);
        main_box.set_margin_start(10);
        main_box.set_margin_end(10);

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_vexpand(true);
        
        let habit_list = ListBox::new();
        habit_list.set_selection_mode(gtk4::SelectionMode::None);
        scrolled_window.set_child(Some(&habit_list));
        
        main_box.append(&scrolled_window);
        content_box.append(&main_box);
        toast_overlay.set_child(Some(&content_box));
        window.set_content(Some(&toast_overlay));

        let style_manager = StyleManager::default();

        let app = Self {
            window,
            storage,
            habit_data: Rc::new(RefCell::new(HabitData::new())),
            password: Rc::new(RefCell::new(None)),
            habit_list,
            toast_overlay,
            style_manager,
            add_button: add_button.clone(),
            settings_button: settings_button.clone(),
        };

        app.setup_events(add_button, settings_button);
        app.authenticate_user()?;
        
        // Fallback: enable add button after a short delay if authentication doesn't complete
        let add_button_fallback = app.add_button.clone();
        glib::timeout_add_seconds_local(3, move || {
            add_button_fallback.set_sensitive(true);
            glib::ControlFlow::Break
        });
        
        Ok(app)
    }

    fn setup_events(&self, add_button: Button, settings_button: Button) {
        let habit_data = self.habit_data.clone();
        let storage = self.storage.clone();
        let password = self.password.clone();
        let habit_list = self.habit_list.clone();
        let toast_overlay = self.toast_overlay.clone();
        
        add_button.connect_clicked(move |_| {
            Self::show_add_habit_dialog(&habit_data, storage.clone(), &password, &habit_list, &toast_overlay);
        });

        let storage_clone = self.storage.clone();
        let password_clone = self.password.clone();
        let habit_data_clone = self.habit_data.clone();
        let habit_list_clone = self.habit_list.clone();
        let toast_overlay_clone = self.toast_overlay.clone();
        
        let style_manager_clone = self.style_manager.clone();
        settings_button.connect_clicked(move |_| {
            Self::show_settings_dialog(&storage_clone, &password_clone, &habit_data_clone, &habit_list_clone, &toast_overlay_clone, &style_manager_clone);
        });
    }

    fn authenticate_user(&self) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
        if !self.storage.exists() {
            self.show_password_setup_dialog()?;
        } else {
            self.show_password_entry_dialog()?;
        }
        Ok(())
    }

    fn show_password_setup_dialog(&self) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
        let dialog = MessageDialog::new(
            Some(&self.window),
            DialogFlags::MODAL,
            gtk4::MessageType::Question,
            gtk4::ButtonsType::OkCancel,
            "Set up encryption password for your habit data:",
        );

        let content_area = dialog.content_area();
        let entry = Entry::new();
        entry.set_visibility(false);
        entry.set_placeholder_text(Some("Enter password"));
        content_area.append(&entry);

        let password = self.password.clone();
        let storage = self.storage.clone();
        let habit_data = self.habit_data.clone();
        
        // Add Enter key support
        let dialog_clone = dialog.clone();
        entry.connect_activate(move |entry| {
            let pass = entry.text().to_string();
            if !pass.is_empty() {
                dialog_clone.response(ResponseType::Ok);
            }
        });
        
        let add_button = self.add_button.clone();
        let settings_button = self.settings_button.clone();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                let pass = entry.text().to_string();
                if !pass.is_empty() {
                    password.replace(Some(pass.clone()));
                    
                    if let Err(e) = storage.save(&habit_data.borrow(), &pass) {
                        eprintln!("Failed to save initial data: {}", e);
                    }
                    
                    // Enable UI after successful authentication
                    add_button.set_sensitive(true);
                    settings_button.set_sensitive(true);
                }
            }
            dialog.close();
        });

        dialog.show();
        Ok(())
    }

    fn show_password_entry_dialog(&self) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
        let dialog = MessageDialog::new(
            Some(&self.window),
            DialogFlags::MODAL,
            gtk4::MessageType::Question,
            gtk4::ButtonsType::OkCancel,
            "Enter your password to access habit data:",
        );

        let content_area = dialog.content_area();
        let entry = Entry::new();
        entry.set_visibility(false);
        entry.set_placeholder_text(Some("Enter password"));
        content_area.append(&entry);

        let password = self.password.clone();
        let storage = self.storage.clone();
        let habit_data = self.habit_data.clone();
        let habit_list = self.habit_list.clone();
        
        // Add Enter key support
        let dialog_clone = dialog.clone();
        entry.connect_activate(move |entry| {
            let pass = entry.text().to_string();
            if !pass.is_empty() {
                dialog_clone.response(ResponseType::Ok);
            }
        });
        
        let add_button = self.add_button.clone();
        let settings_button = self.settings_button.clone();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                let pass = entry.text().to_string();
                if !pass.is_empty() {
                    match storage.load(&pass) {
                        Ok(data) => {
                            password.replace(Some(pass));
                            habit_data.replace(data);
                            Self::refresh_habit_list(&habit_list, &habit_data, &storage, &password);
                            
                            // Enable UI after successful authentication
                            add_button.set_sensitive(true);
                            settings_button.set_sensitive(true);
                        }
                        Err(_) => {
                            eprintln!("Failed to decrypt data. Wrong password?");
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.show();
        Ok(())
    }

    fn show_settings_dialog(
        storage: &SecureStorage,
        password: &Rc<RefCell<Option<String>>>,
        habit_data: &Rc<RefCell<HabitData>>,
        habit_list: &ListBox,
        toast_overlay: &ToastOverlay,
        style_manager: &StyleManager,
    ) {
        let dialog = MessageDialog::new(
            None::<&ApplicationWindow>,
            DialogFlags::MODAL,
            gtk4::MessageType::Other,
            gtk4::ButtonsType::Close,
            "Settings",
        );

        let content_area = dialog.content_area();
        let settings_box = GtkBox::new(Orientation::Vertical, 10);
        settings_box.set_margin_start(20);
        settings_box.set_margin_end(20);
        settings_box.set_margin_top(10);
        settings_box.set_margin_bottom(10);
        
        // Theme toggle section
        let theme_section = GtkBox::new(Orientation::Horizontal, 10);
        theme_section.append(&Label::new(Some("Dark Mode")));
        
        let theme_switch = Switch::new();
        theme_switch.set_active(style_manager.color_scheme() == libadwaita::ColorScheme::ForceDark);
        theme_section.append(&theme_switch);
        
        settings_box.append(&theme_section);
        
        // Add separator
        let separator = gtk4::Separator::new(Orientation::Horizontal);
        separator.set_margin_top(10);
        separator.set_margin_bottom(10);
        settings_box.append(&separator);
        
        // Password change section
        let password_section = GtkBox::new(Orientation::Vertical, 5);
        password_section.append(&Label::new(Some("Change Encryption Password")));
        
        let current_password_entry = Entry::new();
        current_password_entry.set_visibility(false);
        current_password_entry.set_placeholder_text(Some("Current password"));
        
        let new_password_entry = Entry::new();
        new_password_entry.set_visibility(false);
        new_password_entry.set_placeholder_text(Some("New password"));
        
        let confirm_password_entry = Entry::new();
        confirm_password_entry.set_visibility(false);
        confirm_password_entry.set_placeholder_text(Some("Confirm new password"));
        
        let change_password_button = Button::with_label("Change Password");
        change_password_button.add_css_class("suggested-action");
        
        password_section.append(&current_password_entry);
        password_section.append(&new_password_entry);
        password_section.append(&confirm_password_entry);
        password_section.append(&change_password_button);
        
        settings_box.append(&password_section);
        
        // Add separator
        let separator2 = gtk4::Separator::new(Orientation::Horizontal);
        separator2.set_margin_top(10);
        separator2.set_margin_bottom(10);
        settings_box.append(&separator2);
        
        // Backup and restore section
        let backup_section = GtkBox::new(Orientation::Vertical, 5);
        backup_section.append(&Label::new(Some("Backup & Restore")));
        
        let backup_restore_box = GtkBox::new(Orientation::Horizontal, 10);
        
        let export_button = Button::with_label("📤 Export Backup");
        export_button.set_tooltip_text(Some("Export your habits to a backup file"));
        export_button.add_css_class("suggested-action");
        
        let import_button = Button::with_label("📥 Import Backup");
        import_button.set_tooltip_text(Some("Import habits from a backup file"));
        import_button.add_css_class("destructive-action");
        
        backup_restore_box.append(&export_button);
        backup_restore_box.append(&import_button);
        
        backup_section.append(&backup_restore_box);
        settings_box.append(&backup_section);
        
        // Add separator
        let separator3 = gtk4::Separator::new(Orientation::Horizontal);
        separator3.set_margin_top(10);
        separator3.set_margin_bottom(10);
        settings_box.append(&separator3);
        
        // Delete all data section
        let delete_section = GtkBox::new(Orientation::Vertical, 5);
        delete_section.append(&Label::new(Some("Danger Zone")));
        
        let delete_button = Button::with_label("🗑️ Delete All Data");
        delete_button.set_tooltip_text(Some("Delete all habits and data, then setup new password"));
        delete_button.add_css_class("destructive-action");
        
        delete_section.append(&delete_button);
        settings_box.append(&delete_section);
        
        content_area.append(&settings_box);

        // Theme switch event handler
        let style_manager_clone = style_manager.clone();
        theme_switch.connect_active_notify(move |switch| {
            if switch.is_active() {
                style_manager_clone.set_color_scheme(libadwaita::ColorScheme::ForceDark);
            } else {
                style_manager_clone.set_color_scheme(libadwaita::ColorScheme::ForceLight);
            }
        });

        let storage_clone = storage.clone();
        let password_clone = password.clone();
        let habit_data_clone = habit_data.clone();
        let toast_overlay_clone = toast_overlay.clone();
        
        change_password_button.connect_clicked(move |_| {
            let current_pass = current_password_entry.text().to_string();
            let new_pass = new_password_entry.text().to_string();
            let confirm_pass = confirm_password_entry.text().to_string();
            
            if current_pass.is_empty() || new_pass.is_empty() || confirm_pass.is_empty() {
                let error_toast = Toast::new("Please fill in all password fields");
                toast_overlay_clone.add_toast(error_toast);
                return;
            }
            
            if new_pass != confirm_pass {
                let error_toast = Toast::new("New passwords do not match");
                toast_overlay_clone.add_toast(error_toast);
                return;
            }
            
            // Verify current password by trying to load data
            match storage_clone.load(&current_pass) {
                Ok(_) => {
                    // Current password is correct, now save with new password
                    match storage_clone.save(&habit_data_clone.borrow(), &new_pass) {
                        Ok(_) => {
                            password_clone.replace(Some(new_pass));
                            let success_toast = Toast::new("Password changed successfully");
                            toast_overlay_clone.add_toast(success_toast);
                            
                            // Clear the input fields
                            current_password_entry.set_text("");
                            new_password_entry.set_text("");
                            confirm_password_entry.set_text("");
                        }
                        Err(e) => {
                            let error_toast = Toast::new(&format!("Failed to save with new password: {}", e));
                            toast_overlay_clone.add_toast(error_toast);
                        }
                    }
                }
                Err(_) => {
                    let error_toast = Toast::new("Current password is incorrect");
                    toast_overlay_clone.add_toast(error_toast);
                }
            }
        });

        // Export backup button event handler
        let storage_export = storage.clone();
        let password_export = password.clone();
        let toast_overlay_export = toast_overlay.clone();
        
        export_button.connect_clicked(move |_| {
            let password_dialog = MessageDialog::new(
                None::<&ApplicationWindow>,
                DialogFlags::MODAL,
                gtk4::MessageType::Question,
                gtk4::ButtonsType::OkCancel,
                "Enter a password to encrypt your backup file:",
            );

            let content_area = password_dialog.content_area();
            let backup_password_entry = Entry::new();
            backup_password_entry.set_visibility(false);
            backup_password_entry.set_placeholder_text(Some("Backup password"));
            content_area.append(&backup_password_entry);

            let storage_export_inner = storage_export.clone();
            let password_export_inner = password_export.clone();
            let toast_overlay_export_inner = toast_overlay_export.clone();

            password_dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Ok {
                    let backup_password = backup_password_entry.text().to_string();
                    if !backup_password.is_empty() {
                        let file_chooser = FileChooserDialog::new(
                            Some("Export Habits Backup"),
                            None::<&ApplicationWindow>,
                            FileChooserAction::Save,
                            &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)]
                        );
                        
                        file_chooser.set_current_name("habits_backup.encrypted");
                        
                        let filter = FileFilter::new();
                        filter.add_pattern("*.encrypted");
                        filter.set_name(Some("Encrypted backup files"));
                        file_chooser.add_filter(&filter);
                        
                        let storage_export_inner2 = storage_export_inner.clone();
                        let password_export_inner2 = password_export_inner.clone();
                        let toast_overlay_export_inner2 = toast_overlay_export_inner.clone();
                        
                        file_chooser.connect_response(move |dialog, response| {
                            if response == ResponseType::Accept {
                                if let Some(file) = dialog.file() {
                                    if let Some(path) = file.path() {
                                        if let Some(ref current_pass) = *password_export_inner2.borrow() {
                                            match storage_export_inner2.export_backup(current_pass, &backup_password, &path) {
                                                Ok(_) => {
                                                    let success_toast = Toast::new("Encrypted backup exported successfully!");
                                                    toast_overlay_export_inner2.add_toast(success_toast);
                                                }
                                                Err(e) => {
                                                    let error_toast = Toast::new(&format!("Failed to export backup: {}", e));
                                                    toast_overlay_export_inner2.add_toast(error_toast);
                                                }
                                            }
                                        } else {
                                            let error_toast = Toast::new("No password available for export");
                                            toast_overlay_export_inner2.add_toast(error_toast);
                                        }
                                    }
                                }
                            }
                            dialog.close();
                        });
                        
                        file_chooser.show();
                    } else {
                        let error_toast = Toast::new("Backup password cannot be empty");
                        toast_overlay_export_inner.add_toast(error_toast);
                    }
                }
                dialog.close();
            });

            password_dialog.show();
        });

        // Import backup button event handler
        let storage_import = storage.clone();
        let password_import = password.clone();
        let habit_data_import = habit_data.clone();
        let habit_list_import = habit_list.clone();
        let toast_overlay_import = toast_overlay.clone();
        
        import_button.connect_clicked(move |_| {
            let confirmation_dialog = MessageDialog::new(
                None::<&ApplicationWindow>,
                DialogFlags::MODAL,
                gtk4::MessageType::Warning,
                gtk4::ButtonsType::YesNo,
                "Warning: Importing a backup will replace all current habit data. This action cannot be undone. Do you want to continue?"
            );
            
            let storage_import_inner = storage_import.clone();
            let password_import_inner = password_import.clone();
            let habit_data_import_inner = habit_data_import.clone();
            let habit_list_import_inner = habit_list_import.clone();
            let toast_overlay_import_inner = toast_overlay_import.clone();
            
            confirmation_dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Yes {
                    let file_chooser = FileChooserDialog::new(
                        Some("Import Habits Backup"),
                        None::<&ApplicationWindow>,
                        FileChooserAction::Open,
                        &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)]
                    );
                    
                    let filter = FileFilter::new();
                    filter.add_pattern("*.encrypted");
                    filter.set_name(Some("Encrypted backup files"));
                    file_chooser.add_filter(&filter);
                    
                    let storage_import_inner2 = storage_import_inner.clone();
                    let password_import_inner2 = password_import_inner.clone();
                    let habit_data_import_inner2 = habit_data_import_inner.clone();
                    let habit_list_import_inner2 = habit_list_import_inner.clone();
                    let toast_overlay_import_inner2 = toast_overlay_import_inner.clone();
                    
                    file_chooser.connect_response(move |dialog, response| {
                        if response == ResponseType::Accept {
                            if let Some(file) = dialog.file() {
                                if let Some(path) = file.path() {
                                    let password_dialog = MessageDialog::new(
                                        None::<&ApplicationWindow>,
                                        DialogFlags::MODAL,
                                        gtk4::MessageType::Question,
                                        gtk4::ButtonsType::OkCancel,
                                        "Enter the password for this backup file:",
                                    );

                                    let content_area = password_dialog.content_area();
                                    let backup_password_entry = Entry::new();
                                    backup_password_entry.set_visibility(false);
                                    backup_password_entry.set_placeholder_text(Some("Backup password"));
                                    content_area.append(&backup_password_entry);

                                    let storage_import_inner3 = storage_import_inner2.clone();
                                    let password_import_inner3 = password_import_inner2.clone();
                                    let habit_data_import_inner3 = habit_data_import_inner2.clone();
                                    let habit_list_import_inner3 = habit_list_import_inner2.clone();
                                    let toast_overlay_import_inner3 = toast_overlay_import_inner2.clone();
                                    let path_clone = path.clone();

                                    password_dialog.connect_response(move |dialog, response| {
                                        if response == ResponseType::Ok {
                                            let backup_password = backup_password_entry.text().to_string();
                                            if !backup_password.is_empty() {
                                                if let Some(ref current_pass) = *password_import_inner3.borrow() {
                                                    match storage_import_inner3.import_backup(&path_clone, &backup_password, current_pass) {
                                                        Ok(_) => {
                                                            match storage_import_inner3.load(current_pass) {
                                                                Ok(new_data) => {
                                                                    habit_data_import_inner3.replace(new_data);
                                                                    Self::refresh_habit_list(&habit_list_import_inner3, &habit_data_import_inner3, &storage_import_inner3, &password_import_inner3);
                                                                    let success_toast = Toast::new("Encrypted backup imported successfully!");
                                                                    toast_overlay_import_inner3.add_toast(success_toast);
                                                                }
                                                                Err(e) => {
                                                                    let error_toast = Toast::new(&format!("Failed to reload data after import: {}", e));
                                                                    toast_overlay_import_inner3.add_toast(error_toast);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let error_toast = Toast::new(&format!("Failed to import backup: {}", e));
                                                            toast_overlay_import_inner3.add_toast(error_toast);
                                                        }
                                                    }
                                                } else {
                                                    let error_toast = Toast::new("No password available for import");
                                                    toast_overlay_import_inner3.add_toast(error_toast);
                                                }
                                            } else {
                                                let error_toast = Toast::new("Backup password cannot be empty");
                                                toast_overlay_import_inner3.add_toast(error_toast);
                                            }
                                        }
                                        dialog.close();
                                    });

                                    password_dialog.show();
                                }
                            }
                        }
                        dialog.close();
                    });
                    
                    file_chooser.show();
                }
                dialog.close();
            });
            
            confirmation_dialog.show();
        });

        // Delete all data button event handler
        let storage_delete = storage.clone();
        let password_delete = password.clone();
        let habit_data_delete = habit_data.clone();
        let habit_list_delete = habit_list.clone();
        let toast_overlay_delete = toast_overlay.clone();
        
        delete_button.connect_clicked(move |_| {
            let confirmation_dialog = MessageDialog::new(
                None::<&ApplicationWindow>,
                DialogFlags::MODAL,
                gtk4::MessageType::Warning,
                gtk4::ButtonsType::YesNo,
                "Warning: This will permanently delete ALL your habit data and cannot be undone. You will need to set up a new password. Are you sure you want to continue?"
            );
            
            let storage_delete_inner = storage_delete.clone();
            let password_delete_inner = password_delete.clone();
            let habit_data_delete_inner = habit_data_delete.clone();
            let habit_list_delete_inner = habit_list_delete.clone();
            let toast_overlay_delete_inner = toast_overlay_delete.clone();
            
            confirmation_dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Yes {
                    // Delete all data
                    if let Err(e) = storage_delete_inner.delete_all_data() {
                        let error_toast = Toast::new(&format!("Failed to delete data: {}", e));
                        toast_overlay_delete_inner.add_toast(error_toast);
                    } else {
                        // Clear in-memory data
                        habit_data_delete_inner.replace(HabitData::new());
                        password_delete_inner.replace(None);
                        
                        // Clear the habit list
                        while let Some(child) = habit_list_delete_inner.first_child() {
                            habit_list_delete_inner.remove(&child);
                        }
                        
                        // Show success message and prompt for new password setup
                        let success_toast = Toast::new("All data deleted successfully. Please set up a new password.");
                        toast_overlay_delete_inner.add_toast(success_toast);
                        
                        // Trigger password setup dialog
                        let storage_for_dialog = storage_delete_inner.clone();
                        let password_for_dialog = password_delete_inner.clone();
                        let habit_data_for_dialog = habit_data_delete_inner.clone();
                        glib::idle_add_local_once(move || {
                            let _ = Self::show_password_setup_dialog_static(&storage_for_dialog, &password_for_dialog, &habit_data_for_dialog);
                        });
                    }
                }
                dialog.close();
            });
            
            confirmation_dialog.show();
        });

        // Handle dialog close button
        dialog.connect_response(|dialog, _response| {
            dialog.close();
        });

        dialog.show();
    }

    fn show_add_habit_dialog(
        habit_data: &Rc<RefCell<HabitData>>,
        storage: SecureStorage,
        password: &Rc<RefCell<Option<String>>>,
        habit_list: &ListBox,
        toast_overlay: &ToastOverlay,
    ) {
        let dialog = MessageDialog::new(
            None::<&ApplicationWindow>,
            DialogFlags::MODAL,
            gtk4::MessageType::Question,
            gtk4::ButtonsType::OkCancel,
            "Add New Habit",
        );

        let content_area = dialog.content_area();
        let vbox = GtkBox::new(Orientation::Vertical, 5);
        
        let name_entry = Entry::new();
        name_entry.set_placeholder_text(Some("Habit name"));
        
        let desc_entry = Entry::new();
        desc_entry.set_placeholder_text(Some("Description"));
        
        vbox.append(&Label::new(Some("Name:")));
        vbox.append(&name_entry);
        vbox.append(&Label::new(Some("Description:")));
        vbox.append(&desc_entry);
        
        content_area.append(&vbox);

        let habit_data_clone = habit_data.clone();
        let storage_clone = storage;
        let password_clone = password.clone();
        let habit_list_clone = habit_list.clone();
        let toast_overlay_clone = toast_overlay.clone();
        
        // Add Enter key support for both entries
        let dialog_clone1 = dialog.clone();
        let name_entry_clone = name_entry.clone();
        name_entry.connect_activate(move |entry| {
            let name = entry.text().to_string();
            if !name.is_empty() {
                dialog_clone1.response(ResponseType::Ok);
            }
        });
        
        let dialog_clone2 = dialog.clone();
        desc_entry.connect_activate(move |_entry| {
            let name = name_entry_clone.text().to_string();
            if !name.is_empty() {
                dialog_clone2.response(ResponseType::Ok);
            }
        });
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                let name = name_entry.text().to_string();
                let description = desc_entry.text().to_string();
                
                if !name.is_empty() {
                    // Check if password is available before allowing habit creation
                    if let Some(ref pass) = *password_clone.borrow() {
                        let habit = Habit {
                            id: format!("habit_{}", Utc::now().timestamp()),
                            name: name.clone(),
                            description,
                            created_at: Utc::now(),
                            target_days_per_week: 7,
                            streak: 0,
                            longest_streak: 0,
                        };
                        
                        habit_data_clone.borrow_mut().add_habit(habit);
                        
                        if let Err(e) = storage_clone.save(&habit_data_clone.borrow(), pass) {
                            eprintln!("Failed to save data: {}", e);
                        }
                        
                        Self::refresh_habit_list(&habit_list_clone, &habit_data_clone, &storage_clone, &password_clone);
                        
                        let toast = Toast::new(&format!("Added habit: {}", name));
                        toast_overlay_clone.add_toast(toast);
                    } else {
                        let error_toast = Toast::new("Password required to add habits");
                        toast_overlay_clone.add_toast(error_toast);
                    }
                }
            }
            dialog.close();
        });

        dialog.show();
    }
    
    fn show_delete_confirmation(
        habit_id: &str,
        habit_name: &str,
        habit_data: &Rc<RefCell<HabitData>>,
        habit_list: &ListBox,
        storage: &SecureStorage,
        password: &Rc<RefCell<Option<String>>>
    ) {
        println!("Showing delete confirmation for habit: {}", habit_name);
        let dialog = MessageDialog::new(
            None::<&ApplicationWindow>,
            DialogFlags::MODAL,
            gtk4::MessageType::Warning,
            gtk4::ButtonsType::YesNo,
            &format!("Are you sure you want to delete the habit '{}'?\n\nThis will permanently remove all completion data for this habit.", habit_name),
        );
        
        let habit_id = habit_id.to_string();
        let habit_data = habit_data.clone();
        let habit_list = habit_list.clone();
        let storage = storage.clone();
        let password = password.clone();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Yes {
                // Delete the habit
                habit_data.borrow_mut().remove_habit(&habit_id);
                
                // Save the updated data
                if let Some(ref pass) = *password.borrow() {
                    if let Err(e) = storage.save(&habit_data.borrow(), pass) {
                        eprintln!("Failed to save data after deletion: {}", e);
                    }
                }
                
                // Refresh the habit list
                Self::refresh_habit_list(&habit_list, &habit_data, &storage, &password);
            }
            dialog.close();
        });
        
        dialog.show();
    }

    fn show_edit_dialog(
        habit_id: &str,
        current_name: &str,
        current_description: &str,
        habit_data: &Rc<RefCell<HabitData>>,
        habit_list: &ListBox,
        storage: &SecureStorage,
        password: &Rc<RefCell<Option<String>>>
    ) {
        let dialog = Dialog::new();
        dialog.set_title(Some("Edit Habit"));
        dialog.set_modal(true);
        dialog.set_default_size(400, 300);
        
        // Create content area
        let content_area = dialog.content_area();
        let main_box = GtkBox::new(Orientation::Vertical, 10);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);
        
        // Name field
        let name_label = Label::new(Some("Habit Name:"));
        name_label.set_halign(gtk4::Align::Start);
        let name_entry = Entry::new();
        name_entry.set_text(current_name);
        name_entry.set_placeholder_text(Some("Enter habit name"));
        
        // Description field
        let desc_label = Label::new(Some("Description:"));
        desc_label.set_halign(gtk4::Align::Start);
        let desc_entry = Entry::new();
        desc_entry.set_text(current_description);
        desc_entry.set_placeholder_text(Some("Enter habit description"));
        
        main_box.append(&name_label);
        main_box.append(&name_entry);
        main_box.append(&desc_label);
        main_box.append(&desc_entry);
        
        content_area.append(&main_box);
        
        // Add buttons
        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.add_button("Save", ResponseType::Accept);
        
        let habit_id = habit_id.to_string();
        let habit_data = habit_data.clone();
        let habit_list = habit_list.clone();
        let storage = storage.clone();
        let password = password.clone();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let new_name = name_entry.text().to_string();
                let new_description = desc_entry.text().to_string();
                
                if !new_name.trim().is_empty() {
                    // Update the habit
                    habit_data.borrow_mut().update_habit(&habit_id, &new_name, &new_description);
                    
                    // Save the updated data
                    if let Some(ref pass) = *password.borrow() {
                        if let Err(e) = storage.save(&habit_data.borrow(), pass) {
                            eprintln!("Failed to save data after edit: {}", e);
                        }
                    }
                    
                    // Refresh the habit list
                    Self::refresh_habit_list(&habit_list, &habit_data, &storage, &password);
                }
            }
            dialog.close();
        });
        
        dialog.show();
    }

    fn refresh_habit_list(
        habit_list: &ListBox, 
        habit_data: &Rc<RefCell<HabitData>>,
        storage: &SecureStorage,
        password: &Rc<RefCell<Option<String>>>
    ) {
        while let Some(child) = habit_list.first_child() {
            habit_list.remove(&child);
        }

        for habit in &habit_data.borrow().habits {
            // Create main container for habit
            let main_box = GtkBox::new(Orientation::Vertical, 5);
            main_box.add_css_class("habit-container");
            
            // Create header row with habit info using a simple horizontal box
            let header_row = GtkBox::new(Orientation::Horizontal, 4);
            header_row.add_css_class("habit-header");
            
            // Create habit info section
            let habit_info_box = GtkBox::new(Orientation::Vertical, 5);
            habit_info_box.set_hexpand(true);
            habit_info_box.set_halign(gtk4::Align::Start);
            
            // Create clickable title button
            let title_button = Button::new();
            title_button.set_label(&format!("{}\n{}", habit.name, habit.description));
            title_button.add_css_class("flat");
            title_button.add_css_class("habit-title");
            title_button.set_hexpand(true);
            title_button.set_halign(gtk4::Align::Start);
            
            // Create prominent streak display
            let streak_box = GtkBox::new(Orientation::Horizontal, 5);
            streak_box.set_halign(gtk4::Align::Start);
            
            let streak_label = Label::new(Some(get_streak_emoji(habit.streak)));
            streak_label.add_css_class("streak-icon");
            
            let streak_number = Label::new(Some(&format!("{}", habit.streak)));
            streak_number.add_css_class("streak-number");
            if habit.streak >= 7 {
                streak_number.add_css_class("high-streak");
            }
            
            let streak_text = Label::new(Some(if habit.streak == 1 { "day streak" } else { "day streak" }));
            streak_text.add_css_class("streak-text");
            
            streak_box.append(&streak_label);
            streak_box.append(&streak_number);
            streak_box.append(&streak_text);
            
            habit_info_box.append(&title_button);
            habit_info_box.append(&streak_box);
            
            let complete_button = Button::with_label("✅");
            complete_button.set_tooltip_text(Some("Mark as completed today"));
            complete_button.set_has_tooltip(true);
            complete_button.add_css_class("suggested-action");
            complete_button.add_css_class("compact-button");
            complete_button.set_size_request(40, 40);
            complete_button.set_hexpand(false);
            complete_button.set_vexpand(false);
            complete_button.set_halign(gtk4::Align::Center);
            complete_button.set_valign(gtk4::Align::Center);
            
            let edit_button = Button::with_label("✏️");
            edit_button.set_tooltip_text(Some("Edit this habit's name and description"));
            edit_button.set_has_tooltip(true);
            edit_button.add_css_class("compact-button");
            edit_button.set_size_request(40, 40);
            edit_button.set_hexpand(false);
            edit_button.set_vexpand(false);
            edit_button.set_halign(gtk4::Align::Center);
            edit_button.set_valign(gtk4::Align::Center);
            
            let delete_button = Button::with_label("🗑️");
            delete_button.set_tooltip_text(Some("Delete this habit permanently"));
            delete_button.set_has_tooltip(true);
            delete_button.add_css_class("compact-button");
            delete_button.set_size_request(40, 40);
            delete_button.set_hexpand(false);
            delete_button.set_vexpand(false);
            delete_button.set_halign(gtk4::Align::Center);
            delete_button.set_valign(gtk4::Align::Center);
            
            // Add widgets to header row
            header_row.append(&habit_info_box);
            header_row.append(&complete_button);
            header_row.append(&edit_button);
            header_row.append(&delete_button);
            
            // Create calendar view (initially hidden)
            let habit_id_for_callback = habit.id.clone();
            let habit_data_for_callback = habit_data.clone();
            let title_button_for_callback = title_button.clone();
            let streak_number_for_callback = streak_number.clone();
            let streak_label_for_callback = streak_label.clone();
            let habit_name_for_callback = habit.name.clone();
            let habit_description_for_callback = habit.description.clone();
            
            let on_change_callback = Rc::new(move || {
                // Update just the streak display for this specific habit
                if let Some(updated_habit) = habit_data_for_callback.borrow().habits.iter()
                    .find(|h| h.id == habit_id_for_callback) {
                    title_button_for_callback.set_label(&format!("{}\n{}", 
                        habit_name_for_callback,
                        habit_description_for_callback
                    ));
                    streak_number_for_callback.set_text(&format!("{}", updated_habit.streak));
                    
                    // Update streak emoji based on current streak
                    streak_label_for_callback.set_text(get_streak_emoji(updated_habit.streak));
                    
                    // Update streak styling based on current streak
                    if updated_habit.streak >= 7 {
                        streak_number_for_callback.add_css_class("high-streak");
                    } else {
                        streak_number_for_callback.remove_css_class("high-streak");
                    }
                }
            });
            
            let calendar = HabitCalendar::new(
                habit.id.clone(), 
                habit_data.clone(),
                storage.clone(),
                password.clone(),
                Some(on_change_callback)
            );
            
            let calendar_widget = {
                let borrowed_calendar = calendar.borrow();
                borrowed_calendar.widget().clone()
            };
            calendar_widget.set_visible(false); // Initially hidden
            
            // Store the calendar reference in the widget to keep it alive
            unsafe {
                calendar_widget.set_data("habit_calendar", calendar);
            }
            
            main_box.append(&header_row);
            main_box.append(&calendar_widget);
            
            // Make title button clickable to toggle calendar
            let calendar_widget_clone = calendar_widget.clone();
            let main_box_clone = main_box.clone();
            title_button.connect_clicked(move |_| {
                let is_visible = calendar_widget_clone.is_visible();
                calendar_widget_clone.set_visible(!is_visible);
                
                // Toggle expanded state visual indication
                if is_visible {
                    main_box_clone.remove_css_class("expanded");
                } else {
                    main_box_clone.add_css_class("expanded");
                }
            });
            
            // Add click handler for delete button
            let habit_id_delete = habit.id.clone();
            let habit_name_delete = habit.name.clone();
            let habit_data_delete = habit_data.clone();
            let habit_list_delete = habit_list.clone();
            let storage_delete = storage.clone();
            let password_delete = password.clone();
            
            delete_button.connect_clicked(move |_| {
                println!("Delete button clicked for habit: {}", habit_name_delete);
                Self::show_delete_confirmation(
                    &habit_id_delete,
                    &habit_name_delete,
                    &habit_data_delete,
                    &habit_list_delete,
                    &storage_delete,
                    &password_delete
                );
            });
            
            // Add click handler for edit button
            let habit_id_edit = habit.id.clone();
            let habit_name_edit = habit.name.clone();
            let habit_description_edit = habit.description.clone();
            let habit_data_edit = habit_data.clone();
            let habit_list_edit = habit_list.clone();
            let storage_edit = storage.clone();
            let password_edit = password.clone();
            
            edit_button.connect_clicked(move |_| {
                Self::show_edit_dialog(
                    &habit_id_edit,
                    &habit_name_edit,
                    &habit_description_edit,
                    &habit_data_edit,
                    &habit_list_edit,
                    &storage_edit,
                    &password_edit
                );
            });
            
            // Add click handler for today button
            let habit_id = habit.id.clone();
            let habit_data_clone = habit_data.clone();
            let habit_list_clone = habit_list.clone();
            let habit_data_refresh = habit_data.clone();
            let storage_clone = storage.clone();
            let password_clone = password.clone();
            
            complete_button.connect_clicked(move |_| {
                let today = Utc::now().date_naive();
                let is_completed = habit_data_clone.borrow().is_completed_on_date(&habit_id, today);
                
                if is_completed {
                    habit_data_clone.borrow_mut().unmark_completed(&habit_id, today);
                } else {
                    habit_data_clone.borrow_mut().mark_completed(&habit_id, today, None);
                }
                
                // Save data after change
                if let Some(ref pass) = *password_clone.borrow() {
                    if let Err(e) = storage_clone.save(&habit_data_clone.borrow(), pass) {
                        eprintln!("Failed to save data: {}", e);
                    }
                }
                
                // Refresh the entire list to update streak display
                Self::refresh_habit_list(&habit_list_clone, &habit_data_refresh, &storage_clone, &password_clone);
            });
            
            habit_list.append(&main_box);
        }
    }

    pub fn show(&self) {
        self.window.present();
    }
    
    
    fn show_password_setup_dialog_static(
        storage: &SecureStorage,
        password: &Rc<RefCell<Option<String>>>,
        habit_data: &Rc<RefCell<HabitData>>
    ) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
        let dialog = MessageDialog::new(
            None::<&ApplicationWindow>,
            DialogFlags::MODAL,
            gtk4::MessageType::Question,
            gtk4::ButtonsType::OkCancel,
            "Set up new encryption password for your habit data:",
        );

        let content_area = dialog.content_area();
        let entry = Entry::new();
        entry.set_visibility(false);
        entry.set_placeholder_text(Some("Enter new password"));
        content_area.append(&entry);

        let password_clone = password.clone();
        let storage_clone = storage.clone();
        let habit_data_clone = habit_data.clone();
        
        // Add Enter key support
        let dialog_clone = dialog.clone();
        entry.connect_activate(move |entry| {
            let pass = entry.text().to_string();
            if !pass.is_empty() {
                dialog_clone.response(ResponseType::Ok);
            }
        });
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                let pass = entry.text().to_string();
                if !pass.is_empty() {
                    password_clone.replace(Some(pass.clone()));
                    
                    if let Err(e) = storage_clone.save(&habit_data_clone.borrow(), &pass) {
                        eprintln!("Failed to save initial data: {}", e);
                    }
                }
            }
            dialog.close();
        });

        dialog.show();
        Ok(())
    }
}