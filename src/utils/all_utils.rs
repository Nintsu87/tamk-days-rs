use std::fs::{File, OpenOptions};
use std::error::Error as StdError;
use std::io::{self, Write};
use csv::{Error, WriterBuilder};
use chrono::{NaiveDate, ParseError};
use regex::Regex;

// open cloning, equal_to comparison and ordering
// note: uses date compare&ordering automaticly
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    date: NaiveDate,
    description: String,
    primary_category: String,
    secondary_category: String,
}

impl Event {
    // create new Event, "constructor"
    pub fn new(date: NaiveDate, description: String, primary_category: String, secondary_category: String) -> Self {
        Event {
            date,
            description,
            primary_category,
            secondary_category,
        }
    }

    // use Event to test if date is in correct range
    pub fn test_date(date: &str) -> Result<NaiveDate, ParseError> {
            NaiveDate::parse_from_str(date, "%Y-%m-%d")
    }

    // format category in spesific string for csv and printing
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

    // format Event to proper string for csv and print
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

// Used in filter_by_date()
pub enum DateComparison {
    Before,
    After,
    Exact,
    Today,
    All
}

// used in Event::format_to_string() and Event::format_category()
#[derive(Debug, Clone)]
pub enum StringFormat {
    Print,
    Csv,
}

// read csv and create vector from rows
pub fn read_csv(file_path: &str) -> Result<Vec<Event>, Error> {
    let mut events = Vec::new();

    // safe open file if no error
    let file = File::open(&file_path)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.records() {
        // go through readings, note about error-lines, but still continue
        let record = match result {
            Ok(record) => record,
            Err(err) => {
                eprintln!("Error reading CSV record: {}", err);
                continue;
            }
        };

        // get variables
        let date_str = record.get(0).unwrap_or_default();
        let description_str = record.get(1).unwrap_or_default();
        let category_str = record.get(2).unwrap_or_default();

        // if date is not in correct form, note about error-line, but still continue
        let parsed_date = match Event::test_date(date_str) {
            Ok(date) => date,
            Err(err) => {
                eprintln!("Error parsing date: {}", err);
                continue;
            }
        };

        // if categorys are not in correct form, note about error-line, but still continue
        let (primary, secondary) = match parse_string(category_str, '/') {
            Ok((primary, secondary)) => (primary, secondary),
            Err(err) => {
                eprintln!("Invalid input category format: {}", err);
                continue; // Skip to the next record on error
            }
        };

        // create event and push it in the event vector
        let event = Event::new(parsed_date, description_str.to_string(), primary, secondary);
        events.push(event);
    }
    Ok(events)
}

// print all given vector events in order from oldest to latest
pub fn print_events(events: &mut Vec<Event>) {
    events.sort();
    for event in events {
        println!("{}", event.format_to_string(StringFormat::Print));
    }
}

// delete events by writing the file over with not deleted events
pub fn delete_events(filepath: &str, orig: &[Event], events_to_delete: &[Event]) -> Result<(),Box<dyn StdError>> {
    // filter all events that arent in the delete-vector
    let remaining_events: Vec<&Event> = orig.iter().filter(|event| !events_to_delete.contains(event)).collect();
    // reset the file and write the header in it while testing for errors
    let mut wtr = WriterBuilder::new().from_path(filepath)?;
    wtr.write_record(&["date", "description", "category"])?;

    // write every event row by row to the file
    for event in remaining_events {
        wtr.write_record(&[
            &event.date.format("%Y-%m-%d").to_string(),
            &event.description,
            &event.format_category(StringFormat::Csv),
        ])?;
    }

    // make sure all rows are written in the file
    wtr.flush()?;
    Ok(())
}


// add filtered Events to results vector
pub fn filter_by_date(
    orig: &[Event],
    results: &mut Vec<Event>,
    date_str: &str,
    comparison: DateComparison,
) -> Result<(), ParseError> {
    // if no given date, use todays date instead
    let given_date = if date_str.is_empty() {
        chrono::Local::now().naive_local().date()
    } else {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?
    };

    // create temp vector for gathering filtered events and filter by given comparison
    let mut temp_results: Vec<Event> = Vec::new();
    for event in orig {
        match comparison {
            DateComparison::Before => {
                if event.date < given_date {
                    temp_results.push(event.clone());
                }
            }
            DateComparison::After => {
                if event.date > given_date {
                    temp_results.push(event.clone());
                }
            }
            DateComparison::Exact | DateComparison::Today => {
                if event.date == given_date {
                    temp_results.push(event.clone());
                }
            }
            DateComparison::All => {
                temp_results.push(event.clone());
            }
        }
    }

    // filter out the dublicates
    for event in temp_results {
        if !results.contains(&event) {
            results.push(event);
        }
    }

    Ok(())
}

// split the string in 2 parts
pub fn parse_string(categories: &str, splitter: char) -> Result<(String, String), String> {
    // split the given string using splitter
    let mut parts = categories.split(splitter);

    // create strings from split or empty string
    let part1 = parts.next().unwrap_or_default().trim().to_string();
    let part2 = parts.next().unwrap_or_default().trim().to_string();

    // if more than 2 split results, result error
    if parts.next().is_some() {
        return Err("Too many parts in category for the split".to_string());
    }
    Ok((part1, part2))
}

// filter all events by category or description
pub fn filter_by_string(orig: &[Event], results: &mut Vec<Event>, input: &str, excluded: bool, category: bool) {
    let lower_input = input.to_lowercase();
    // split given category_string to categories
    let categories: Vec<&str> = lower_input.split(',').map(|s| s.trim()).collect();
    // go through events in orig
    for event in orig {
        let include_event: bool;
        // if string is category
        if category {
            // compare the categories to event categies
            let primary_matches = categories.iter().any(|&category| {
                event.primary_category.to_lowercase().starts_with(category)

            });
            let secondary_matches = categories.iter().any(|&category| {
                event.secondary_category.to_lowercase().starts_with(category)

            });

            // create boolean depending if excluded is active
            include_event = if excluded {
                !(primary_matches || secondary_matches)
            } else {
                primary_matches || secondary_matches
            };
        // must be description if not category
        } else {
            // create boolean depending if excluded is active

            include_event = event.description.to_lowercase().starts_with(&lower_input)
        }
        // add event in result list if its not added already
        if include_event && !results.contains(event) {
            results.push(event.clone());
        }
    }
}

// validate to match "YYYY-mm-dd" format (NaiveDate accepts 2023-3-3 instead of 2023-03-03)
pub fn validate_date_format(date_str: &str) -> bool {
    // use regular expression pattern to match "YYYY-mm-dd"
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

    // check if a match
    re.is_match(date_str)
}

// simple open file for append
pub fn open_file_for_append(filepath: &str) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath)
}

// simple append to given file. Meant to use with open_file_for_append()
pub fn append_to_csv(file: &mut File, new_row: String) -> io::Result<()> {
    file.write_all(new_row.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // import necessary items for testing
    use super::*;

    //
    //  parse_string() tests:
    //

    #[test]
    fn test_parse_string_valid_input() {
        // test input with "2 parameters"
        let input = "first,second";
        let result = parse_string(input, ',');
        assert_eq!(result, Ok(("first".to_string(), "second".to_string())));
    }

    #[test]
    fn test_parse_string_single_part() {
        // test input with "1 parameter"
        let input = "only";
        let result = parse_string(input, ',');
        assert_eq!(result, Ok(("only".to_string(), "".to_string())));
    }

    #[test]
    fn test_parse_string_empty_input() {
        // test input with empty string
        let input = "";
        let result = parse_string(input, ',');
        assert_eq!(result, Ok(("".to_string(), "".to_string())));
    }

    #[test]
    fn test_parse_string_too_many_parts() {
        // test input with "too many parameters"
        let input = "first,second,third";
        let result = parse_string(input, ',');
        assert!(result.is_err()); // Expecting an error
        assert_eq!(result.unwrap_err(), "Too many parts in category for the split".to_string());
    }

    //
    //  Event tests:
    //

    #[test]
    fn test_event_formatting_print() {

        let date = NaiveDate::from_ymd_opt(2024, 4, 15).expect("Valid date");
        let description = "test shananigans".to_string();
        let primary_category = "testprimary".to_string();
        let secondary_category = "testsecondary".to_string();

        let event = Event::new(date, description.clone(), primary_category.clone(), secondary_category.clone());

        let formatted_string = event.format_to_string(StringFormat::Print);

        let expected_string = format!("{}: {}, {}/{}", date.format("%Y-%m-%d"), description, primary_category, secondary_category);

        assert_eq!(formatted_string, expected_string);
    }

    #[test]
    fn test_event_formatting_csv() {
        let date = NaiveDate::from_ymd_opt(2024, 4, 15).expect("Valid date");
        let description = "test shananigans".to_string();
        let primary_category = "testprimary".to_string();
        let secondary_category = "testsecondary".to_string();

        let event = Event::new(date, description.clone(), primary_category.clone(), secondary_category.clone());

        let formatted_string = event.format_to_string(StringFormat::Csv);

        let expected_string = format!("{},{},{}", date.format("%Y-%m-%d"), description, format!("{}/{}", primary_category, secondary_category));

        assert_eq!(formatted_string, expected_string);
    }

    //
    //  validate_date_format() tests:
    //

    #[test]
    fn test_date_format() {
        // test valid formats
        assert!(validate_date_format("2022-06-05")); // valid
        assert!(!validate_date_format("2022-6-05")); // month missing 0
        assert!(!validate_date_format("2024-06-5")); // day missing 0
    }

    //
    // helper functions for filter_by_string and filter_by_date() functions
    //

    fn get_today() -> NaiveDate {
        chrono::Local::now().naive_local().date()
    }

    fn create_test_events() -> Vec<Event> {
        vec![
            Event::new(NaiveDate::from_ymd_opt(2022, 4, 1).expect("Valid date"), "event1".to_string(), "work".to_string(), "".to_string()),
            Event::new(NaiveDate::from_ymd_opt(2022, 4, 15).expect("Valid date"), "event2".to_string(), "study".to_string(), "homework".to_string()),
            Event::new(get_today(), "event3".to_string(), "exercise".to_string(), "running".to_string()),
        ]
    }

    //
    // filter_by_string() tests
    //

    #[test]
    fn test_filter_by_string_include_category() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_string(&events, &mut results, "work", false, true);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "event1");
    }

    #[test]
    fn test_filter_by_string_exclude_category() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_string(&events, &mut results, "study", true, true);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].description, "event1");
        assert_eq!(results[1].description, "event3");
    }

    #[test]
    fn test_filter_by_string_description() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_string(&events, &mut results, "event2", false, false);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "event2");
    }

    //
    // filter_by_date() tests
    //
    #[test]
    fn test_filter_by_date_before() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_date(&events, &mut results, "2022-04-15", DateComparison::Before).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "event1");
    }

    #[test]
    fn test_filter_by_date_after() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_date(&events, &mut results, "2022-04-15", DateComparison::After).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "event3");
    }

    #[test]
    fn test_filter_by_date_exact() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_date(&events, &mut results, "2022-04-15", DateComparison::Exact).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "event2");
    }

    #[test]
    fn test_filter_by_date_today() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_date(&events, &mut results, "", DateComparison::Today).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "event3");
    }

    #[test]
    fn test_filter_by_date_all() {
        let events = create_test_events();
        let mut results = Vec::new();
        filter_by_date(&events, &mut results, "2022-04-15", DateComparison::All).unwrap();

        assert_eq!(results.len(), 3);
    }
}

