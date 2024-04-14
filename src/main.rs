mod utils;
use utils::all_utils::{read_csv, print_events, filter_by_date, filter_by_string, append_to_csv, open_file_for_append, delete_events, parse_string, validate_date_format, DateComparison, StringFormat, Event};
use std::path::Path;
use clap::{App, Arg, SubCommand};

// mostly used arg names for consistance and maintaining
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
                                    .long(TODAY_ARG)
                                    .takes_value(false)
                                    .required(false)
                                    .help("Choose events on todays date.")
                            )
                            .arg(
                                Arg::new(BEFORE_DATE_ARG)
                                .long(BEFORE_DATE_ARG)
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events before given date\nGive date in format: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new(AFTER_DATE_ARG)
                                .long(AFTER_DATE_ARG)
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events after given date\nGive date in format: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new(DATE_ARG)
                                .long(DATE_ARG)
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events on given date\nGive date in format: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new(CATEGORY_ARG)
                                .long(CATEGORY_ARG)
                                .takes_value(true)
                                .value_name("CAT[,CAT...]")
                                .required(false)
                                .help("Filter one or multiple categories separated by commas")
                            )
                            .arg(
                                Arg::new("exclude")
                                .long("exclude")
                                .takes_value(false)
                                .requires(CATEGORY_ARG)
                                .help("Exclude the category filter.")
                            )
                            .arg(
                                Arg::new(DESCRIPTION_ARG)
                                .long(DESCRIPTION_ARG)
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
                                .long(DATE_ARG)
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Add event with given date\nGive date in format: YYYY-MM-DD\nNo date: use todays date")
                            )
                            .arg(
                                Arg::new(DESCRIPTION_ARG)
                                .long(DESCRIPTION_ARG)
                                .takes_value(true)
                                .value_name("DESCRIPTION")
                                .required(true)
                                .help("Add event description")
                            )
                            .arg(
                                Arg::new(CATEGORY_ARG)
                                .long(CATEGORY_ARG)
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
                                .long("dry-run")
                                .takes_value(false)
                                .required(false)
                                .help("List filtered to be deleted events without deleting them.")
                            )
                            .arg(
                                Arg::new(DESCRIPTION_ARG)
                                .long(DESCRIPTION_ARG)
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by start of the description.")
                            )
                            .arg(
                                Arg::new(DATE_ARG)
                                .long(DATE_ARG)
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(AFTER_DATE_ARG)
                                .long(AFTER_DATE_ARG)
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(BEFORE_DATE_ARG)
                                .long(BEFORE_DATE_ARG)
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(TODAY_ARG)
                                .long(TODAY_ARG)
                                .takes_value(false)
                                .required(false)
                                .help("Filter to delete by date.")
                            )
                            .arg(
                                Arg::new(CATEGORY_ARG)
                                .long(CATEGORY_ARG)
                                .takes_value(true)
                                .required(false)
                                .help("Filter to delete by start of the primary or secondary category.")
                            )
                            .arg(
                                Arg::new("all")
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

    // create new vector for filtered results
    let mut result_events = Vec::new();

    // match subvommand matches to list, add and delete
    match matches.subcommand() {
        Some(("list", list_matches)) => {
            // without args present, print all events
            if !list_matches.args_present() {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, "", DateComparison::All) {
                    eprintln!("Error parsing date: {}", err);
                    std::process::exit(1);
                }
            }

            // add todays date matches on results
            if list_matches.is_present(TODAY_ARG) {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, "", DateComparison::Today) {
                    eprintln!("Error parsing date: {}", err);
                    std::process::exit(1);
                }
            }

            // add before given date matches on results
            if let Some(date) = list_matches.value_of(BEFORE_DATE_ARG) {
                if validate_date_format(date) {
                    if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Before) {
                        eprintln!("Error parsing date: {}", err);
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                    std::process::exit(1);
                }
            }

            // add after given date matches on results
            if let Some(date) = list_matches.value_of(AFTER_DATE_ARG) {
                if validate_date_format(date) {
                    if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::After) {
                        eprintln!("Error parsing date: {}", err);
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                    std::process::exit(1);
                }
            }

            // add given date matches on results
            if let Some(date) = list_matches.value_of(DATE_ARG) {
                if validate_date_format(date) {
                    if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Exact) {
                        eprintln!("Error parsing date: {}", err);
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                    std::process::exit(1);
                }

            }

            // add given category/categories matches to results, depending if excluded or not
            if let Some(categories) = list_matches.value_of(CATEGORY_ARG) {
                let exclude = list_matches.is_present("exclude");
                filter_by_string(&orig_events, &mut result_events, categories , exclude, true);
            }

            // add given category/categories matches to results
            if let Some(description) = list_matches.value_of(DESCRIPTION_ARG) {
                filter_by_string(&orig_events, &mut result_events, description , false, false);
            }

            // print all results
            print_events(&mut result_events);
        }

        // add given event to the used file
        Some(("add", add_matches)) => {

            // only if description arg is given do all
            if let Some(description_str) = add_matches.value_of(DESCRIPTION_ARG) {
                // open file for appending, exit program if fail
                let mut file = match open_file_for_append(&path_string) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("Error opening file: {}", err);
                        std::process::exit(1);
                    }
                };

                // use given date, if the date is in correct format ...
                let event_naive = if let Some(event_date_str) = add_matches.value_of(DATE_ARG) {
                    // validate and test the date. if either of them fail, exit the program
                    if validate_date_format(event_date_str) {
                        match Event::test_date(event_date_str) {
                            Ok(event_date) => event_date,
                            Err(err) => {
                                eprintln!("Error parsing date: {}", err);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprint!("Date need to be YYYY-mm-dd");
                        std::process::exit(1);
                    }
                // ... else use todays date
                } else {
                    chrono::Local::now().naive_local().date()
                };

                // create primary and secondary categories from possibly given category arg
                let (primary_category_str, secondary_category_str) = match add_matches.value_of(CATEGORY_ARG) {
                    // use parse_string() to get 2 strings depending on given category input
                    Some(category) => {
                        let lower_category = category.to_lowercase();

                        match parse_string(&lower_category, ',') {
                            Ok((primary,secondary)) => (primary, secondary),
                            Err(err) => {
                                eprintln!("{}", err);
                                std::process::exit(1);
                            }
                        }
                    }
                    // if no input, use empty category strings
                    None => (String::new(), String::new()),
                };

                // create event from the information
                let new_event = Event::new(
                    event_naive,
                    description_str.to_string(),
                    primary_category_str,
                    secondary_category_str
                );

                // append event to the file if no errors appear
                if let Err(err) = append_to_csv(&mut file, new_event.format_to_string(StringFormat::Csv)) {
                    eprintln!("Error appending to CSV file: {}", err);
                    std::process::exit(1);
                }
            // if no description arg, stop running
            } else {
                eprintln!("Error: Can't add event without description argument.");
                std::process::exit(1);
            }
        }
        // delete filtered dates if not dry-run
        Some(("delete", delete_matches)) => {
            // check for dry-run
            let dry_run = delete_matches.is_present("dry-run");
            // no args given stop running
            if !delete_matches.args_present() {
                eprintln!("Error: Add filters. Available filters: all, description, category, date, after-date, before-date, today, dry-run. More info from --help");
                std::process::exit(1);
            }

            // filter all to delete
            if delete_matches.is_present("all") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, "", DateComparison::All) {
                    eprintln!("Error parsing date: {}", err);
                    std::process::exit(1);
                }
            // if rest of the accepted args are present
            } else if delete_matches.is_present(TODAY_ARG) || delete_matches.is_present(DESCRIPTION_ARG) || delete_matches.is_present(CATEGORY_ARG) || delete_matches.is_present(DATE_ARG) || delete_matches.is_present(AFTER_DATE_ARG) || delete_matches.is_present(BEFORE_DATE_ARG)|| delete_matches.is_present(DATE_ARG) {
                // filter to delete with description
                if let Some(description_str) = delete_matches.value_of(DESCRIPTION_ARG) {
                    filter_by_string(&orig_events, &mut result_events, description_str, false, false);
                }
                // filter to delete with category
                if let Some(category_str) = delete_matches.value_of(CATEGORY_ARG) {
                    filter_by_string(&orig_events, &mut result_events, category_str, false, true)
                }
                // filter to delete with today
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, "", DateComparison::Today) {
                    eprintln!("Error parsing date: {}", err);
                    std::process::exit(1);
                }
                // filter to delete with date while validating the given input
                if let Some(date) = delete_matches.value_of(DATE_ARG) {
                    if validate_date_format(date) {
                        if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Exact) {
                            eprintln!("Error parsing date: {}", err);
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                        std::process::exit(1);
                    }
                }
                // filter to delete with after-date while validating the given input
                if let Some(date) = delete_matches.value_of(AFTER_DATE_ARG) {
                    if validate_date_format(date) {
                        if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::After) {
                            eprintln!("Error parsing date: {}", err);
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                        std::process::exit(1);
                    }
                }
                // filter to delete with before-date while validating the given input
                if let Some(date) = delete_matches.value_of(BEFORE_DATE_ARG) {
                    if validate_date_format(date) {
                        if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Before) {
                            eprintln!("Error parsing date: {}", err);
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!("Error parsing date. Use format YYYY-mm-dd.");
                        std::process::exit(1);
                    }
                }
            }
            // rewrite the file without filtered events or just print them on when dry-run
            match dry_run {
                true => {
                    println!("Following events are filtered for deleting:");
                    print_events(&mut result_events)
                },
                false => {
                    // Perform actual deletion of events
                    delete_events(&path_string, &orig_events, &result_events)
                        .unwrap_or_else(|err| {
                            // Handle error by printing a custom message and panic
                            eprintln!("Error deleting events: {}", err);
                            std::process::exit(1);
                        });
                }
            }
        }
        // if subcommand is given but coded in, give error
        Some((command, _)) => {
            eprintln!("Error: Not accepted subcommand: {}. Use: list, add or delete", command);
            std::process::exit(1);
        }
        // if no subcommand is given, give error
        None => {
           eprintln!("Error: No provided subcommand.  Use: list, add or delete");
           std::process::exit(1);
        }
    }
}


