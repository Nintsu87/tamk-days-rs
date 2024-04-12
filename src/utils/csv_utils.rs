use std::fs::{File, OpenOptions};
use std::error::Error as StdError;
use csv::{Error, WriterBuilder};
use chrono::{NaiveDate, ParseError};
use std::io::{self, Write};
//use std::io::prelude::*;

//use std::env;
//use std::path::PathBuf;

#[derive(Debug, Clone)] // enable cloning
#[derive(PartialEq)]    // enable comparison
pub struct Event {
    date: NaiveDate,
    description: String,
    primary_category: String,
    secondary_category: String,
}

impl Event {
    pub fn new(date: NaiveDate, description: String, primary_category: String, secondary_category: String) -> Self {
        Event {
            date,
            description,
            primary_category,
            secondary_category,
        }
    }

    pub fn test_date(date: &str) -> Result<NaiveDate, ParseError> {
        NaiveDate::parse_from_str(date, "%Y-%m-%d")
    }
    fn format_category(&self, format:StringFormat) -> String {
        match format {
            StringFormat::Print => {
                match (self.primary_category.is_empty(), self.secondary_category.is_empty()) {
                    (true, true) => "/".to_string(),
                    (false, true) => self.primary_category.clone(),
                    _ => format!("{}/{}", self.primary_category, self.secondary_category),
                }
            }
            StringFormat::Csv => {
                match (self.primary_category.is_empty(), self.secondary_category.is_empty()) {
                    (true, true) => String::new(),
                    (false, true) => self.primary_category.clone(),
                    _ => format!("{}/{}", self.primary_category, self.secondary_category),
                }
            }
        }
    }
    pub fn format_to_string(&self, format: StringFormat) -> String {
        let date = self.date.format("%Y-%m-%d").to_string();
        let description_string = if self.description.is_empty() {
            String::new()
        } else {
            self.description.clone()
        };
        let category_string = self.format_category(format.clone());
        match format {
            StringFormat::Print => {
                format!("{}: {}, {}", date, description_string, category_string)
            }
            StringFormat::Csv => {
                format!("{},{},{}", date, description_string, category_string)
            }
        }
    }
}
pub enum DateComparison {
    Before,
    After,
    Exact,
    Today,
    All
}

#[derive(Debug, Clone)]
pub enum StringFormat {
    Print,
    Csv,
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

        match Event::test_date(date_str) {
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
        println!("{}", event.format_to_string(StringFormat::Print));
    }
}

pub fn delete_events(filepath: &str, orig: &[Event], events_to_delete: &[Event]) -> Result<(),Box<dyn StdError>> {

    let remaining_events: Vec<&Event> = orig.iter().filter(|event| !events_to_delete.contains(event)).collect();
    let mut wtr = WriterBuilder::new().from_path(filepath)?;
    for event in remaining_events {
        wtr.write_record(&[
            &event.date.format("%Y-%m-%d").to_string(),
            &event.description,
            &event.format_category(StringFormat::Csv),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn filter_by_date(
    orig: &[Event],
    results: &mut Vec<Event>,
    date_str: &str,
    comparison: DateComparison,
) -> Result<(), ParseError> {
    let given_date = if date_str.is_empty() {
        chrono::Local::now().naive_local().date()
    } else {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?
    };

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
            DateComparison::Exact | DateComparison::Today => {
                if event.date == given_date {
                    results.push(event.clone());
                }
            }
            DateComparison::All => {
                results.push(event.clone());
            }
        }
    }

    Ok(())
}

fn parse_string(categories: &str) -> Option<(String, String)> {
    // Split the input string by the comma delimiter
    let mut parts = categories.split('/');

    // Attempt to extract the first part (if available and non-empty)
    let part1 = match parts.next() {
        Some(s) => s.trim().to_string(), // Trim and convert to String if part exists
        None => String::new(),
    };

    // Attempt to extract the second part (if available and non-empty)
    let part2 = match parts.next() {
        Some(s) => s.trim().to_string(), // Trim and convert to String if part exists
        None => String::new(),
    };

    Some((part1, part2))
}

pub fn filter_by_string(orig: &[Event], results: &mut Vec<Event>, input: &str, excluded: bool, category: bool) {
    let lower_input = input.to_lowercase();

    let categories: Vec<&str> = lower_input.split(',').map(|s| s.trim()).collect();

    for event in orig {
        let include_event: bool;
        if category {
            let primary_starts_with_category = categories.iter().any(|&category| event.primary_category.starts_with(category));
            let secondary_starts_with_category = categories.iter().any(|&category| event.secondary_category.starts_with(category));

            // Determine if the event should be included based on inclusive/exclusive filtering
            include_event = if excluded {
                !primary_starts_with_category && !secondary_starts_with_category
            } else {
                primary_starts_with_category || secondary_starts_with_category
            };
        } else {
            include_event = event.description.to_lowercase().starts_with(&lower_input);
        }
        if include_event {
            results.push(event.clone());
        }
    }
}

pub fn open_file(filepath: &str) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath)
}

pub fn append_to_csv(file: &mut File, new_row: String) -> io::Result<()> {
    file.write_all(new_row.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}
