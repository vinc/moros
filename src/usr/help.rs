use crate::api::console::Style;
use crate::api::process::ExitCode;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    match args.len() {
        1 => help_summary(),
        2 => help_command(args[1]),
        _ => {
            help();
            Err(ExitCode::UsageError)
        }
    }
}

fn help_command(cmd: &str) -> Result<(), ExitCode> {
    match cmd {
        "-h" | "--help" => {
            help();
            Ok(())
        }
        "date" => help_date(),
        "edit" => help_edit(),
        _ => help_unknown(cmd),
    }
}

fn help_unknown(cmd: &str) -> Result<(), ExitCode> {
    error!("Help not found for command '{}'", cmd);
    Err(ExitCode::Failure)
}

fn print_usage(alias: &str, command: &str, usage: &str) {
    let csi_col1 = Style::color("LightGreen");
    let csi_col2 = Style::color("LightCyan");
    let csi_reset = Style::reset();
    println!(
        "  {}{}{}{:21}{}{}",
        csi_col1, alias, csi_col2, command, csi_reset, usage
    );
}

fn help_summary() -> Result<(), ExitCode> {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();

    println!("{}Usage:{}", csi_color, csi_reset);
    print_usage("", "<dir>", " Change directory");
    print_usage("", "<cmd>", " Execute command");
    println!();

    println!("{}Commands:{}", csi_color, csi_reset);
    print_usage(
        "c",
        "opy <file> <file>",
        "Copy file from source to destination",
    );
    print_usage("d", "elete <file>", "Delete file or empty directory");
    print_usage("e", "dit <file>", "Edit existing or new file");
    print_usage("f", "ind <str> <path>", "Find pattern in path");
    print_usage("h", "elp <cmd>", "Display help about a command");
    print_usage("l", "ist <dir>", "List entries in directory");
    print_usage(
        "m",
        "ove <file> <file>",
        "Move file from source to destination",
    );
    print_usage("p", "rint <str>", "Print string to screen");
    print_usage("q", "uit", "Quit the console");
    print_usage("r", "ead <file>", "Read file to screen");
    print_usage("w", "rite <file>", "Write file or directory");
    println!();

    println!("{}Credits:{}", csi_color, csi_reset);
    println!("  Made with <3 in 2019-2023 by Vincent Ollivier <v@vinc.cc>");
    Ok(())
}

fn help_edit() -> Result<(), ExitCode> {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "MOROS text editor is a very simple editor inspired by Pico.");
    println!();
    println!("{}Commands:{}", csi_color, csi_reset);
    let commands = [
        ("^Q", "Quit editor"),
        ("^W", "Write to file"),
        ("^X", "Write to file and quit"),
        ("^T", "Go to top of file"),
        ("^B", "Go to bottom of file"),
        ("^A", "Go to beginning of line"),
        ("^E", "Go to end of line"),
        ("^D", "Cut line"),
        ("^Y", "Copy line"),
        ("^P", "Paste line"),
    ];
    for (command, usage) in &commands {
        let csi_color = Style::color("LightCyan");
        let csi_reset = Style::reset();
        println!("  {}{}{}    {}", csi_color, command, csi_reset, usage);
    }
    Ok(())
}

fn help_date() -> Result<(), ExitCode> {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("The date command's formatting behavior is based on strftime.");
    println!();
    println!("{}Specifiers:{}", csi_color, csi_reset);
    let specifiers = [
        (
            "%a",
            "Abbreviated weekday name",
            "Thu",
        ),
        (
            "%A",
            "Full weekday name",
            "Thursday",
        ),
        (
            "%b",
            "Abbreviated month name",
            "Aug",
        ),
        (
            "%B",
            "Full month name",
            "August",
        ),
        (
            "%c",
            "Date and time, equivalent to %a %b %-d %-H:%M:%S %-Y",
            "Thu Aug 23 14:55:02 2001",
        ),
        (
            "%C",
            "Year divided by 100 and truncated to integer (00-99)",
            "20",
        ),
        (
            "%d",
            "Day of the month, zero-padded (01-31)",
            "23",
        ),
        (
            "%D",
            "Short MM/DD/YY date, equivalent to %-m/%d/%y",
            "8/23/01",
        ),
        (
            "%F",
            "Short YYYY-MM-DD date, equivalent to %-Y-%m-%d",
            "2001-08-23",
        ),
        (
            "%g",
            "Week-based year, last two digits (00-99)",
            "01",
        ),
        (
            "%G",
            "Week-based year",
            "2001",
        ),
        (
            "%H",
            "Hour in 24h format (00-23)",
            "14",
        ),
        (
            "%I",
            "Hour in 12h format (01-12)",
            "02",
        ),
        (
            "%j",
            "Day of the year (001-366)",
            "235",
        ),
        (
            "%m",
            "Month as a decimal number (01-12)",
            "08",
        ),
        (
            "%M",
            "Minute (00-59)",
            "55",
        ),
        (
            "%N",
            "Subsecond nanoseconds. Always 9 digits",
            "012345678",
        ),
        (
            "%p", 
            "am or pm designation", "pm"),
        (
            "%P", 
            "AM or PM designation", "PM"),
        (
            "%r",
            "12-hour clock time, equivalent to %-I:%M:%S %p",
            "2:55:02 pm",
        ),
        (
            "%R",
            "24-hour HH:MM time, equivalent to %-H:%M",
            "14:55"
        ),
        (
            "%S",
            "Second (00-59)",
            "02"
        ),
        (
            "%T",
            "24-hour clock time with seconds, equivalent to %-H:%M:%S",
            "14:55:02",
        ),
        (
            "%u",
            "ISO 8601 weekday as number with Monday as 1 (1-7)",
            "4",
        ),
        (
            "%U",
            "Week number with Sunday as first day of the week (00-53)",
            "33",
        ),
        (
            "%V",
            "ISO 8601 week number (01-53)",
            "34",
        ),
        (
            "%w",
            "Weekday as a decimal number with Sunday as 0 (0-6)",
            "4",
        ),
        (
            "%W",
            "Week number with Monday as first day of the week (00-53)",
            "34",
        ),
        (
            "%y",
            "Year, last two digits (00-99)",
            "01",
        ),
        (
            "%Y",
            "Full year, including + if ≥10,000",
            "2001",
        ),
        (
            "%z",
            "ISO 8601 offset from UTC in timezone (+HHMM)",
            "+0100",
        ),
        (
            "%%",
            "Literal %",
            "%"
        ),
    ];
    for (specifier, usage, _exemple) in &specifiers {
        let csi_color = Style::color("LightCyan");
        let csi_reset = Style::reset();
        println!("  {}{}{}    {}", csi_color, specifier, csi_reset, usage);
    }
    Ok(())
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} help {}[<command>]{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
