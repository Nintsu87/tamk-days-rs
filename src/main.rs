mod utils;
use utils::csv_utils::{read_csv, print_events, add_all, filter_today, filter_by_date, filter_by_category, DateComparison};
use std::path::Path;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("NinasAlmanak")
                    .version("1.0")
                    .author("--")
                    .about("project")
                    .subcommand(
                        SubCommand::with_name("list")
                            .about("Print all events if no filters are specified\n\tcargo run -- list")
                            .arg(
                                Arg::new("today")
                                    .short('t')
                                    .long("today")
                                    .takes_value(false)
                                    .required(false)
                                    .help("Choose events on todays date.")
                            )
                            .arg(
                                Arg::new("before-date")
                                .short('b')
                                .long("before-date")
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events before given date\nGive date in form: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new("after-date")
                                .short('a')
                                .long("after-date")
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events after given date\nGive date in form: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new("date")
                                .short('d')
                                .long("date")
                                .takes_value(true)
                                .value_name("YYYY-MM-DD")
                                .required(false)
                                .help("Choose events on given date\nGive date in form: YYYY-MM-DD")
                            )
                            .arg(
                                Arg::new("categories")
                                .short('c')
                                .long("categories")
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
                                .requires("categories")
                                .help("Exclude the category pick.")
                            )
                    )
                    .get_matches();

    // create operating system free path to the events.csv and create a String for file handling
    let current_dir = match std::env::current_dir() {
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
    if let Some(list_matches) = matches.subcommand_matches("list") {
        if !list_matches.args_present() {
            add_all(&orig_events, &mut result_events);
        }
        if list_matches.is_present("today") {
            filter_today(&orig_events, &mut result_events);
        }
        if list_matches.is_present("before-date"){
            if let Some(date) = list_matches.value_of("before-date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Before) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else {
                println!("Something unexpected happened!");
            }
        }
        if list_matches.is_present("after-date"){
            if let Some(date) = list_matches.value_of("after-date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::After) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else {
                println!("Something unexpected happened!");
            }
        }

        if list_matches.is_present("date"){
            if let Some(date) = list_matches.value_of("date") {
                if let Err(err) = filter_by_date(&orig_events, &mut result_events, date, DateComparison::Exact) {
                    eprintln!("Error parsing date: {}", err);
                    return;
                }
            } else {
                println!("Something unexpected happened!");
            }
        }

        if list_matches.is_present("categories"){
            if let Some(categories) = list_matches.value_of("categories") {
                let exclude = list_matches.is_present("exclude");
                filter_by_category(&orig_events, &mut result_events, categories , exclude);
            } else {
                println!("Something unexpected happened!");
            }
        }
        print_events(&result_events);
    } else {
        // Jos annettua alikomentoa ei ole määritelty
        eprintln!("Error: Unknown or missing subcommand. Available subcommands: list");
        // Tulosta virheviesti, jos tuntematon alikomento
    }

}


