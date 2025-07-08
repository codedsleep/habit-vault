use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub target_days_per_week: u8,
    pub streak: u32,
    pub longest_streak: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitCompletion {
    pub habit_id: String,
    pub date: NaiveDate,
    pub completed_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HabitData {
    pub habits: Vec<Habit>,
    pub completions: Vec<HabitCompletion>,
}

impl HabitData {
    pub fn new() -> Self {
        Self {
            habits: Vec::new(),
            completions: Vec::new(),
        }
    }
    
    pub fn add_habit(&mut self, habit: Habit) {
        self.habits.push(habit);
    }
    
    pub fn remove_habit(&mut self, habit_id: &str) {
        self.habits.retain(|h| h.id != habit_id);
        self.completions.retain(|c| c.habit_id != habit_id);
    }
    
    pub fn update_habit(&mut self, habit_id: &str, new_name: &str, new_description: &str) {
        if let Some(habit) = self.habits.iter_mut().find(|h| h.id == habit_id) {
            habit.name = new_name.to_string();
            habit.description = new_description.to_string();
        }
    }
    
    pub fn mark_completed(&mut self, habit_id: &str, date: NaiveDate, notes: Option<String>) {
        if self.is_completed_on_date(habit_id, date) {
            return;
        }
        
        let completion = HabitCompletion {
            habit_id: habit_id.to_string(),
            date,
            completed_at: Utc::now(),
            notes,
        };
        
        self.completions.push(completion);
        self.update_streak(habit_id);
    }
    
    pub fn unmark_completed(&mut self, habit_id: &str, date: NaiveDate) {
        self.completions.retain(|c| !(c.habit_id == habit_id && c.date == date));
        self.update_streak(habit_id);
    }
    
    pub fn is_completed_on_date(&self, habit_id: &str, date: NaiveDate) -> bool {
        self.completions.iter().any(|c| c.habit_id == habit_id && c.date == date)
    }
    
    pub fn get_habit_by_id(&self, habit_id: &str) -> Option<&Habit> {
        self.habits.iter().find(|h| h.id == habit_id)
    }
    
    pub fn get_habit_by_id_mut(&mut self, habit_id: &str) -> Option<&mut Habit> {
        self.habits.iter_mut().find(|h| h.id == habit_id)
    }
    
    fn update_streak(&mut self, habit_id: &str) {
        let mut streak = 0;
        let today = Utc::now().date_naive();
        
        // Start counting from today and go backwards
        // Only count consecutive days including today
        if self.is_completed_on_date(habit_id, today) {
            streak = 1;
            
            // Count consecutive days backwards from yesterday
            for days_back in 1.. {
                let check_date = today - chrono::Duration::days(days_back);
                if self.is_completed_on_date(habit_id, check_date) {
                    streak += 1;
                } else {
                    break;
                }
            }
        }
        
        if let Some(habit) = self.get_habit_by_id_mut(habit_id) {
            habit.streak = streak;
            if streak > habit.longest_streak {
                habit.longest_streak = streak;
            }
        }
    }
    
    pub fn get_completions_for_habit(&self, habit_id: &str) -> Vec<&HabitCompletion> {
        self.completions.iter()
            .filter(|c| c.habit_id == habit_id)
            .collect()
    }
}