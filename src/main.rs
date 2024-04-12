mod utils;
use utils::csv_utils::{read_csv, print_events, filter_by_date, filter_by_string, append_to_csv, open_file, delete_events, parse_string, validate_date_format, DateComparison, StringFormat};

use std::path::Path;
use clap::{App, Arg, SubCommand};

use crate::utils::csv_utils::Event;


const DESCRIPTION_ARG: &str = "description";
const CATEGORY_ARG: &str = "category";
const TODAY_ARG: &str = "today";
const BEFORE_DATE_ARG: &str = "before-date";
const AFTER_DATE_ARG: &str = "after-date";
const DATE_ARG: &str = "date";

fn main() {
    let matches = App::new("NinasAlmanak")
                    .version("1.0")
                    .author("--")
                    .about("project")
                    .subcommand(
                        SubCommand::with_name("list")
                            .about("Print all events if no filters are specified\n\tcargo run -- list")
                            .arg(
                                Arg::new(TODAY_ARG)
                                    .short('t')
                                    .long("today")
                                    .takes_value(false)
                                    .required(false)
                                    .help("Choose events on todays date.")
                            )
                            .arg(
                                Arg::new(BEFORE_DATE_ARG)
                                .short('b')
                                .long("before-date")
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events before given date\nGive date in format: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new(AFTER_DATE_ARG)
                                .short('a')
                                .long("after-date")
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events after given date\nGive date in format: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new(DATE_ARG)
                                .short('d')
                                .long("date")
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events on given date\nGive date in format: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new(CATEGORY_ARG)
                                .short('c')
                                .long("category")
                                .takes_value(true)
                                .value_name("CAT[,CAT...]")
                                .required(false)
                                .help("Pick one or multiple categories separated by commas")
                            )
                            .arg(
                                Arg::new("exclude")
                                .short('e')
                                .long("exclude")
                                .takes_value(false)
                                .requires(CATEGORY_ARG)
                                .help("Exclude the category pick.")
                            )
                            .arg(
                                Arg::new(DESCRIPTION_ARG)
                                .short('x')
                                .long("description")
                                .takes_value(true)
                                .value_name("DESCRIPTION")
                                .required(false)
                                .help("Choose events with start of description value")
                            )

                    )
                    .subcommand(
                        SubCommand::with_name("add")
                            .about("Add event in used csv file.\n\tcargo run -- add\n\nNo given date: use todays date.")
                            .arg(
                                Arg::new(DATE_ARG)
                                .short('d')
                                .long("date")
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Add event with given date\nGive date in format: YYYY-MM-DD\nNo date: use todays date")
                            )
                            .arg(
                                Arg::new(DESCRIPTION_ARG)
                                .short('x')
                                .long("description")
                                .takes_value(true)
                                .value_name("DESCRIPTION")
                                .required(true)
                                .help("Add event description")
                            )
                            .arg(
                                Arg::new(CATEGORY_ARG)
                                .short('c')
                                .long("category")
                                .takes_value(true)
                                .value_name("PRIMARY[,SECONDARY]")
                                .required(false)
                                .help("Add event category/categories.\nGive category in format: \n\t<primary_category[,secondary_category]>")
                            )
                    )
                    .subcommand(
                        SubCommand::with_name("delete")
                            .about("Delete event in used csv file with filters.")
                            .arg(
                                Arg::new("dry-run")
                                .short('r')
                                .long("dry-run")
                                .takes_value(false)
                                .required(false)
                                .help("List filtered to be deleted events without deleting them.")
                            )
                            .arg(
                                Arg::new(DESCRIPTION_ARG)
                                .short('x')
                                .long("description")
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by start of the description.")
                            )
                            .arg(
                                Arg::new(DATE_ARG)
                                .short('d')
                                .long("date")
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(AFTER_DATE_ARG)
                                .short('a')
                                .long("after-date")
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(BEFORE_DATE_ARG)
                                .short('b')
                                .long("before-date")
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(TODAY_ARG)
                                .short('t')
                                .long("today")
                                .takes_value(false)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(CATEGORY_ARG)
                                .short('c')
                                .long("category")
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by start of the primary or secondary category.")
                            )
                            .arg(
                                Arg::new("all")
                                .short('k')
                                .long("all")
                                .takes_value(false)
                                .required(false)
                                .help("Filter to delete every event.")
                            )
                    )
                    .get_matches();

    // create operating system free path to the events.csv and create a String for file handling
    let current_dir =  match std::env::current_dir(){
        Ok(path) => path,
        Err(err) => {
            eprintln!("Failed to get current directory: {}", err);
            return;
        }
    };
    let relative_path = Path::new("src").join("utils").join("events.csv");
    let full_path = current_dir.join(relative_path);
    let path_string = full_path.to_string_lossy().into_owned();

    // create event vector
    let orig_events = match read_csv(&path_string) {
        Ok(csv_events) => csv_events,
        Err(err) => {
            // If there's an error reading CSV, print the error message
            eprintln!("Error reading CSV file: {}", err);
            return;
        }
    };

    let mut result_events = Vec::new();
    match matches.subcommand() {
        Some(("list", list_matches)) => {
            if !list_matches.args_present() {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, "", DateComparison::All) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            }

            if list_matches.is_present(TODAY_ARG) {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, "", DateComparison::Today) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            }


            if let Some(date) = list_matches.value_of(BEFORE_DATE_ARG) {
                if validate_date_format(date) {
                    if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Before) {
                        eprintln!("Error parsing date: {}", err);
                        return;
                    }
                } else {
                    eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                    return;
                }
            }

            if let Some(date) = list_matches.value_of(AFTER_DATE_ARG) {
                if validate_date_format(date) {
                    if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::After) {
                        eprintln!("Error parsing date: {}", err);
                        return;
                    }
                } else {
                    eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                    return;
                }
            }

            if let Some(date) = list_matches.value_of(DATE_ARG) {
                if validate_date_format(date) {
                    if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Exact) {
                        eprintln!("Error parsing date: {}", err);
                        return;
                    }
                } else {
                    eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                    return;
                }

            }

            if let Some(categories) = list_matches.value_of(CATEGORY_ARG) {
                let exclude = list_matches.is_present("exclude");
                filter_by_string(&orig_events, &mut result_events, categories , exclude, false);
            }

            if let Some(description) = list_matches.value_of(DESCRIPTION_ARG) {
                filter_by_string(&orig_events, &mut result_events, description , false, true);
            }

            print_events(&result_events);

        }
        Some(("add", add_matches)) => {
            if let Some(description_str) = add_matches.value_of("description") {
                let mut file = match open_file(&path_string) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("Error opening file: {}", err);
                        return;  // Exit the program early if opening the file fails
                    }
                };

                // Attempt to parse the date string into a NaiveDate


                let event_naive = if let Some(event_date_str) = add_matches.value_of("date") {
                    if validate_date_format(event_date_str) {
                    // Attempt to parse the date string into a NaiveDate
                        match Event::test_date(event_date_str) {
                            Ok(event_date) => event_date,
                            Err(err) => {
                                // Handle parsing error (e.g., invalid date format)
                                eprintln!("Error parsing date: {}", err);
                                // Default to today's date if parsing fails
                                return;
                            }
                        }
                    } else {
                        eprint!("Date need to be YYYY-mm-dd");
                        return;
                    }
                } else {
                    // No date string provided, default to today's date
                    chrono::Local::now().naive_local().date()
                };
                /*
                let (primary_category_str, secondary_category_str) = match add_matches.value_of("category") {
                    Some(category) => {
                        let lower_category = category.to_lowercase();
                        let categories: Vec<&str> = lower_category.split(',').map(|s| s.trim()).collect();

                        match categories.len() {
                            1 => (categories[0].to_string(), String::new()),
                            2 => (categories[0].to_string(), categories[1].to_string()),
                            _ => {
                                eprintln!("Error: Use 1-2 categories separated with comma.");
                                return; // Exit the function if category count is invalid
                            }
                        }
                    }
                    None => (String::new(), String::new()),
                };*/

                let (primary_category_str, secondary_category_str) = match add_matches.value_of("category") {
                    Some(category) => {
                        let lower_category = category.to_lowercase();

                        match parse_string(&lower_category, ',') {
                            Ok((primary,secondary)) => (primary, secondary),
                            Err(err) => {
                                eprintln!("{}", err);
                                return;
                            }
                        }
                    }
                    None => (String::new(), String::new()),
                };

                let new_event = Event::new(
                    event_naive,
                    description_str.to_string(),
                    primary_category_str,
                    secondary_category_str
                );

                if let Err(err) = append_to_csv(&mut file, new_event.format_to_string(StringFormat::Csv)) {
                    eprintln!("Error appending to CSV file: {}", err);
                    // Handle error as needed (e.g., log, cleanup, etc.)
                }
            } else {
                eprintln!("Error: Can't add event without description argument.")
            }
        }
        Some(("delete", delete_matches)) => {
            let dry_run = delete_matches.is_present("dry-run");
            if !delete_matches.args_present() {
                eprintln!("Error: Add filters. Available filters: all, description, category, dry-run. More info from --help")
            }
            if delete_matches.is_present("all") {
                let empty_string = String::new();
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, &empty_string, DateComparison::All) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else if delete_matches.is_present("today") {
                let empty_string = String::new();
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, &empty_string, DateComparison::Today) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else if delete_matches.is_present("description") || delete_matches.is_present("category") || delete_matches.is_present("date") || delete_matches.is_present("after-date") || delete_matches.is_present("before-date")|| delete_matches.is_present("date") {
                if let Some(description_str) = delete_matches.value_of("description") {
                    filter_by_string(&orig_events, &mut result_events, description_str, false, false);
                }
                if let Some(category_str) = delete_matches.value_of("category") {
                    filter_by_string(&orig_events, &mut result_events, category_str, false, true)
                }

                if let Some(date) = delete_matches.value_of("date") {
                    if validate_date_format(date) {
                        if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Exact) {
                            eprintln!("Error parsing date: {}", err);
                            return;
                        }
                    } else {
                        eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                        return;
                    }
                }
                if let Some(date) = delete_matches.value_of("after-date") {
                    if validate_date_format(date) {
                        if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::After) {
                            eprintln!("Error parsing date: {}", err);
                            return;
                        }
                    } else {
                        eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                        return;
                    }
                }
                if let Some(date) = delete_matches.value_of("before-date") {
                    if validate_date_format(date) {
                        if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Before) {
                            eprintln!("Error parsing date: {}", err);
                            return;
                        }
                    } else {
                        eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                        return;
                    }
                }
            }
            match dry_run {
                true => {
                    println!("Following events are filtered for deleting:");
                    print_events(&result_events)
                },
                false => {
                    // Perform actual deletion of events
                    delete_events(&path_string, &orig_events, &result_events)
                        .unwrap_or_else(|err| {
                            // Handle error by printing a custom message and panic
                            eprintln!("Error deleting events: {}", err);
                            std::process::exit(1); // Exit the program with a non-zero status
                        });
                }
            }
        }
        Some((command, _)) => {
            eprintln!("Error: Not accepted subcommand: {}. Use: list, add or delete", command);
        }
        None => {
            // Handle other subcommands or return an error if needed
           eprintln!("Error: No provided subcommand.  Use: list, add or delete");
        }
    }/* my version of version control
    if let Some(list_matches) = matches.subcommand_matches("list") {
        if !list_matches.args_present() {
            let empty_string = String::new();
            if let Err(err) = filter_by_date(&orig_events, &mut result_events, &empty_string, DateComparison::All) {
                eprintln!("Error parsing date: {}", err);
                return;
            }
        }

        if list_matches.is_present("today") {
            let empty_string = String::new();
            if let Err(err) = filter_by_date(&orig_events, &mut result_events, &empty_string, DateComparison::Today) {
                eprintln!("Error parsing date: {}", err);
                return;
            }
        }

        if list_matches.is_present("before-date"){
            if let Some(date) = list_matches.value_of("before-date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Before) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else {
                println!("Something unexpected happened in list-before-date!");
            }
        }
        if list_matches.is_present("after-date"){
            if let Some(date) = list_matches.value_of("after-date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::After) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else {
                println!("Something unexpected happened in list-after-date!");
            }
        }

        if list_matches.is_present("date"){
            if let Some(date) = list_matches.value_of("date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Exact) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else {
                println!("Something unexpected happenedin list-date!");
            }
        }

        if list_matches.is_present("categories"){
            if let Some(categories) = list_matches.value_of("categories") {
                let exclude = list_matches.is_present("exclude");
                filter_by_string(&orig_events, &mut result_events, categories , exclude, false);
            } else {
                println!("Something unexpected happened in list-categories!");
            }
        }

        if list_matches.is_present("description"){
            if let Some(description) = list_matches.value_of("description") {
                filter_by_string(&orig_events, &mut result_events, description , false, true);
            } else {
                println!("Something unexpected happened in list-description!");
            }
        }
        print_events(&result_events);
    } else if let Some(add_matches) = matches.subcommand_matches("add") {
        if let Some(description_str) = add_matches.value_of("description") {
            let mut file = match open_file(&path_string) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Error opening file: {}", err);
                    return;  // Exit the program early if opening the file fails
                }
            };

            // Attempt to parse the date string into a NaiveDate
            let event_naive = if let Some(event_date_str) = add_matches.value_of("date") {
                // Attempt to parse the date string into a NaiveDate
                match Event::test_date(event_date_str) {
                    Ok(event_date) => event_date,
                    Err(err) => {
                        // Handle parsing error (e.g., invalid date format)
                        eprintln!("Error parsing date: {}", err);
                        // Default to today's date if parsing fails
                        chrono::Local::now().naive_local().date()
                    }
                }
            } else {
                // No date string provided, default to today's date
                chrono::Local::now().naive_local().date()
            };

            println!("{}", event_naive.format("%Y-%m-%d").to_string());
            let (primary_category_str, secondary_category_str) = match add_matches.value_of("category") {
                Some(category) => {
                    let lower_category = category.to_lowercase();
                    let categories: Vec<&str> = lower_category.split(',').map(|s| s.trim()).collect();

                    match categories.len() {
                        1 => (categories[0].to_string(), String::new()),
                        2 => (categories[0].to_string(), categories[1].to_string()),
                        _ => {
                            eprintln!("Error: Use 1-2 categories separated with comma.");
                            return; // Exit the function if category count is invalid
                        }
                    }
                }
                None => (String::new(), String::new()),
            };

            let new_event = Event::new(
                event_naive,
                description_str.to_string(),
                primary_category_str,
                secondary_category_str
            );
            println!("{}",new_event.format_to_string(StringFormat::Print));

            if let Err(err) = append_to_csv(&mut file, new_event.format_to_string(StringFormat::Csv)) {
                eprintln!("Error appending to CSV file: {}", err);
                // Handle error as needed (e.g., log, cleanup, etc.)
            }
        } else {
            eprintln!("Error: Can't add event without description argument.")
        }

    } else if let Some(delete_matches) = matches.subcommand_matches("delete") {
        let dry_run = delete_matches.is_present("dry-run");
        if !delete_matches.args_present() {
            eprintln!("Error: Add filters. Available filters: all, description, category, dry-run. More info from --help")
        }
        if delete_matches.is_present("all") {
            let empty_string = String::new();
            if let Err(err) = filter_by_date(&orig_events, &mut result_events, &empty_string, DateComparison::All) {
                eprintln!("Error parsing date: {}", err);
                return;
            }
        } else if delete_matches.is_present("today") {
            let empty_string = String::new();
            if let Err(err) = filter_by_date(&orig_events, &mut result_events, &empty_string, DateComparison::Today) {
                eprintln!("Error parsing date: {}", err);
                return;
            }
        } else if delete_matches.is_present("description") || delete_matches.is_present("category") || delete_matches.is_present("date") || delete_matches.is_present("after-date") || delete_matches.is_present("before-date")|| delete_matches.is_present("date") {
            if let Some(description_str) = delete_matches.value_of("description") {
                filter_by_string(&orig_events, &mut result_events, description_str, false, false);
            }
            if let Some(category_str) = delete_matches.value_of("category") {
                filter_by_string(&orig_events, &mut result_events, category_str, false, true)
            }

            if let Some(date) = delete_matches.value_of("date") {
                println!("{}",date);
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Exact) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            }
            if let Some(date) = delete_matches.value_of("after-date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::After) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            }
            if let Some(date) = delete_matches.value_of("before-date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Before) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            }

        }

        match dry_run {
            true => {
                println!("Following events are filtered for deleting:");
                print_events(&result_events)
            },
            false => {
                // Perform actual deletion of events
                delete_events(&path_string, &orig_events, &result_events)
                    .unwrap_or_else(|err| {
                        // Handle error by printing a custom message and panic
                        eprintln!("Error deleting events: {}", err);
                        std::process::exit(1); // Exit the program with a non-zero status
                    });
            }
        }
    } else {
        // Jos annettua alikomentoa ei ole määritelty
        eprintln!("Error: Unknown or missing subcommand. Available subcommands: list, add, delete");
        // Tulosta virheviesti, jos tuntematon alikomento
    }*/

}


