use std::fs::File;
use chrono::{Datelike, NaiveDate, ParseError};
//use std::io;
//use std::io::prelude::*;
use csv::Error;
//use std::env;
//use std::path::PathBuf;

#[derive(Debug, Clone)] // Add #[derive(Clone)] to enable cloning
pub struct Event {
    date: NaiveDate,
    description: String,
    primary_category: String,
    secondary_category: String,
}

pub enum DateComparison {
    Before,
    After,
    Exact,
}

// Function to read and process a CSV file
pub fn read_csv(file_path: &str) -> Result<Vec<Event>, Error> {
    let mut events = Vec::new();

    let file = File::open(&file_path)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.records() {
        let record = result?;
        //println!("{:?}", record); // Process each CSV record (a row)

        let date_str = record.get(0).unwrap_or_default();
        let description_str = record.get(1).unwrap_or_default();
        let category_str = record.get(2).unwrap_or_default();

        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(parsed_date) => {
                if let Some((first, second)) = parse_string(category_str) {
                    events.push(Event {
                        date: parsed_date,
                        description: description_str.to_string(),
                        primary_category: first,
                        secondary_category: second,
                    });
                } else {
                    println!("Invalid input category-format");
                }

            }
            Err(err) => {
                eprintln!("Error parsing date: {}", err);
            }
        }
    }

    Ok(events)
}

pub fn print_events(events: &Vec<Event>) {
    for event in events {
        let date = event.date.format("%Y-%m-%d").to_string();
        let category_string = if event.secondary_category.is_empty() {
            // If secondary category is empty, use primary category alone
            format!("{}", event.primary_category)
        } else {
            // If secondary category is present, format the primary and secondary categories
            format!("{}/{}", event.primary_category, event.secondary_category)
        };
        println!("{}: {}, {}", date, event.description, category_string)
    }
}

pub fn add_all(orig: &[Event], results: &mut Vec<Event>) {
    for event in orig {
        results.push(event.clone());
    }
}

pub fn filter_today(orig: &[Event], results: &mut Vec<Event>) {
    // Get today's date
    let today = chrono::Local::now().naive_local();

    // Iterate over the original events
    for event in orig {
        // Check if the event's date matches today's month and day
        if event.date.month() == today.month() && event.date.day() == today.day() {
            // Add the event to the results vector
            results.push(event.clone());
        }
    }
}

pub fn filter_by_date(
    orig: &[Event],
    results: &mut Vec<Event>,
    date_str: &str,
    comparison: DateComparison,
) -> Result<(), ParseError> {
    let given_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;

    for event in orig {
        match comparison {
            DateComparison::Before => {
                if event.date < given_date {
                    results.push(event.clone());
                }
            }
            DateComparison::After => {
                if event.date > given_date {
                    results.push(event.clone());
                }
            }
            DateComparison::Exact => {
                if event.date == given_date {
                    results.push(event.clone());
                }
            }
        }
    }

    Ok(())
}

fn parse_string(categories: &str) -> Option<(String, String)> {
    // Split the input string by the comma delimiter
    let mut parts = categories.split('/');

    // Attempt to extract the first part (if available and non-empty)
    let part1 = match parts.next()?.trim() {
        "" => return None, // Return None if the first part is empty
        s => s.to_string(),
    };

    // Attempt to extract the second part (if available and non-empty)
    let part2 = match parts.next() {
        Some(s) => s.trim().to_string(), // Trim and convert to String if part exists
        None => String::new(),
    };

    Some((part1, part2))
}

pub fn filter_by_category(orig: &[Event], results: &mut Vec<Event>, input: &str, excluded: bool) {
    let lower_input = input.to_lowercase();
    let categories: Vec<&str> = lower_input.split(',').map(|s| s.trim()).collect();

    for event in orig {
        let primary_starts_with_category = categories.iter().any(|&category| event.primary_category.starts_with(category));
        let secondary_starts_with_category = categories.iter().any(|&category| event.secondary_category.starts_with(category));

        // Determine if the event should be included based on inclusive/exclusive filtering
        let include_event = if excluded {
            !primary_starts_with_category && !secondary_starts_with_category
        } else {
            primary_starts_with_category || secondary_starts_with_category
        };

        if include_event {
            results.push(event.clone());
        }
    }
}

