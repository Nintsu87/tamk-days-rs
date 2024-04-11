extern crate csv;

// Re-export modules
pub mod utils;

// Public items (traits, structs, functions) that constitute the crate's API
pub use crate::utils::csv_utils::{read_csv, print_events, add_all, filter_today, filter_by_date, filter_by_category, DateComparison};