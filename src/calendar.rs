use gtk4::prelude::*;
use gtk4::{Button, Grid, Label, Orientation};
use gtk4::Box as GtkBox;
use chrono::{NaiveDate, Datelike, Duration};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::habit::HabitData;
use crate::storage::SecureStorage;

pub struct HabitCalendar {
    widget: GtkBox,
    habit_id: String,
    current_month: NaiveDate,
    habit_data: Rc<RefCell<HabitData>>,
    storage: SecureStorage,
    password: Rc<RefCell<Option<String>>>,
    on_change: Option<Rc<dyn Fn()>>,
    self_ref: Option<Weak<RefCell<Self>>>,
}

impl HabitCalendar {
    pub fn new(
        habit_id: String, 
        habit_data: Rc<RefCell<HabitData>>,
        storage: SecureStorage,
        password: Rc<RefCell<Option<String>>>,
        on_change: Option<Rc<dyn Fn()>>
    ) -> Rc<RefCell<Self>> {
        let widget = GtkBox::new(Orientation::Vertical, 5);
        let current_month = chrono::Utc::now().date_naive().with_day(1).unwrap();
        
        let calendar = Rc::new(RefCell::new(Self {
            widget,
            habit_id,
            current_month,
            habit_data,
            storage,
            password,
            on_change,
            self_ref: None,
        }));
        
        // Set self reference
        calendar.borrow_mut().self_ref = Some(Rc::downgrade(&calendar));
        
        calendar.borrow().build_calendar();
        calendar
    }
    
    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
    
    fn build_calendar(&self) {
        self.build_calendar_content();
    }
    
    fn navigate_month(&mut self, direction: i32) {
        
        if direction > 0 {
            // Next month
            if self.current_month.month() == 12 {
                self.current_month = NaiveDate::from_ymd_opt(
                    self.current_month.year() + 1, 1, 1
                ).unwrap();
            } else {
                self.current_month = NaiveDate::from_ymd_opt(
                    self.current_month.year(), 
                    self.current_month.month() + 1, 
                    1
                ).unwrap();
            }
        } else {
            // Previous month
            if self.current_month.month() == 1 {
                self.current_month = NaiveDate::from_ymd_opt(
                    self.current_month.year() - 1, 12, 1
                ).unwrap();
            } else {
                self.current_month = NaiveDate::from_ymd_opt(
                    self.current_month.year(), 
                    self.current_month.month() - 1, 
                    1
                ).unwrap();
            }
        }
        
        self.rebuild_calendar();
    }
    
    fn rebuild_calendar(&self) {
        // Clear existing children
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }
        
        // Rebuild with new month - similar to build_calendar but without setting up handlers again
        self.build_calendar_content();
    }
    
    fn build_calendar_content(&self) {
        // Header with month navigation
        let header = GtkBox::new(Orientation::Horizontal, 10);
        header.set_halign(gtk4::Align::Center);
        
        let prev_button = Button::with_label("◀");
        let month_label = Label::new(Some(&format!(
            "{} {}",
            self.current_month.format("%B"),
            self.current_month.year()
        )));
        let next_button = Button::with_label("▶");
        
        // Add click handlers for navigation
        if let Some(self_ref) = &self.self_ref {
            let calendar_prev = self_ref.clone();
            prev_button.connect_clicked(move |_| {
                if let Some(cal) = calendar_prev.upgrade() {
                    cal.borrow_mut().navigate_month(-1);
                }
            });
            
            let calendar_next = self_ref.clone();
            next_button.connect_clicked(move |_| {
                if let Some(cal) = calendar_next.upgrade() {
                    cal.borrow_mut().navigate_month(1);
                }
            });
        }
        
        header.append(&prev_button);
        header.append(&month_label);
        header.append(&next_button);
        
        self.widget.append(&header);
        
        // Add the calendar grid
        self.add_calendar_grid();
    }
    
    fn add_calendar_grid(&self) {
        // Calendar grid
        let grid = Grid::new();
        grid.set_row_spacing(2);
        grid.set_column_spacing(2);
        grid.set_halign(gtk4::Align::Center);
        
        // Day headers
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        for (i, day) in days.iter().enumerate() {
            let label = Label::new(Some(day));
            label.add_css_class("calendar-header");
            grid.attach(&label, i as i32, 0, 1, 1);
        }
        
        // Calculate first day of month and days in month
        let first_day = self.current_month;
        let last_day = if self.current_month.month() == 12 {
            NaiveDate::from_ymd_opt(self.current_month.year() + 1, 1, 1).unwrap() - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(self.current_month.year(), self.current_month.month() + 1, 1).unwrap() - Duration::days(1)
        };
        
        // Get first Monday of the calendar
        let mut current_date = first_day;
        while current_date.weekday().number_from_monday() != 1 {
            current_date = current_date - Duration::days(1);
        }
        
        // Fill calendar grid
        let mut row = 1;
        let mut col = 0;
        
        for _ in 0..42 { // 6 weeks max
            let day_button = self.create_day_button(current_date);
            grid.attach(&day_button, col, row, 1, 1);
            
            col += 1;
            if col >= 7 {
                col = 0;
                row += 1;
            }
            
            current_date = current_date + Duration::days(1);
            
            // Stop if we've gone past the current month and filled at least one full week
            if current_date > last_day && col == 0 {
                break;
            }
        }
        
        self.widget.append(&grid);
    }
    
    fn create_day_button(&self, date: NaiveDate) -> Button {
        let day_str = date.day().to_string();
        let button = Button::with_label(&day_str);
        button.set_size_request(30, 30);
        button.add_css_class("calendar-day-button");
        
        let is_current_month = date.month() == self.current_month.month();
        let is_completed = self.habit_data.borrow().is_completed_on_date(&self.habit_id, date);
        let is_today = date == chrono::Utc::now().date_naive();
        
        // Style the button based on state
        if !is_current_month {
            button.add_css_class("calendar-other-month");
            button.set_sensitive(false);
        } else if is_completed {
            button.add_css_class("calendar-completed");
        } else if is_today {
            button.add_css_class("calendar-today");
        }
        
        // Add click handler
        let habit_data_clone = self.habit_data.clone();
        let habit_id_clone = self.habit_id.clone();
        let storage_clone = self.storage.clone();
        let password_clone = self.password.clone();
        let on_change_clone = self.on_change.clone();
        
        button.connect_clicked(move |btn| {
            let is_completed = {
                let data = habit_data_clone.borrow();
                data.is_completed_on_date(&habit_id_clone, date)
            };
            
            // Toggle completion state
            {
                let mut data = habit_data_clone.borrow_mut();
                if is_completed {
                    data.unmark_completed(&habit_id_clone, date);
                } else {
                    data.mark_completed(&habit_id_clone, date, None);
                }
            }
            
            // Update button appearance immediately
            if is_completed {
                btn.remove_css_class("calendar-completed");
            } else {
                btn.add_css_class("calendar-completed");
            }
            
            // Save data after change
            if let Some(ref pass) = *password_clone.borrow() {
                if let Err(e) = storage_clone.save(&habit_data_clone.borrow(), pass) {
                    eprintln!("Failed to save data: {}", e);
                }
            }
            
            // Call the change callback to refresh main UI (streak counter)
            if let Some(ref callback) = on_change_clone {
                callback();
            }
        });
        
        button
    }
}