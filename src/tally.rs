use clap::ArgMatches;

use crate::{error::AocError, util::get_time_symbol};

use crate::util::tally_util::*;

fn print_info(
    days: Vec<(usize, (usize, Option<usize>))>,
    not_done: Vec<usize>,
    number_of_runs: usize,
) {
    let unit = get_time_symbol();
    let red_text = |s: usize| format!("\x1b[0;33;31m{}\x1b[0m", s);
    let gold_text = |s: &str| format!("\x1b[0;33;10m{}\x1b[0m:", s);
    let silver_text = |s: &str| format!("\x1b[0;34;34m{}\x1b[0m:", s);

    if !not_done.is_empty() {
        let mut not_done = not_done;
        not_done.sort_unstable();
        let mut s = String::new();
        let mut first = true;
        for day in not_done {
            if !first {
                s.push_str(", ");
            }
            s.push_str(&red_text(day));
            first = false;
        }
        println!("Days not completed: {}", s);
    }
    println!("STATS:");
    println!("Number of runs: {}:\n", number_of_runs);

    let print_info = |text: String, vec: Vec<(usize, usize)>| {
        println!("{}", text);

        let mut data: Vec<_> = vec.iter().map(|(_, time)| *time).collect();
        data.sort_unstable();

        let median = data[data.len() / 2];

        let total = vec.iter().map(|(_, time)| time).sum::<usize>();
        let avg = total / vec.len();

        let (highest_day, highest_time) = vec.iter().max_by_key(|k| k.1).unwrap();

        println!("\t Total time:  \t{}{unit}", total);
        println!("\t Average time:\t{}{unit}", avg);
        println!("\t Median time: \t{}{unit}", median);
        println!(
            "\t Highest time:\t{}{unit}, day: {}",
            highest_time, highest_day
        );
        println!();
    };

    let silver = days
        .iter()
        .map(|(day, (p1, _))| (*day, *p1))
        .collect::<Vec<_>>();
    let gold = days
        .iter()
        .filter_map(|(day, (_, p2))| p2.map(|p2| (*day, p2)))
        .collect::<Vec<_>>();

    let total = gold
        .iter()
        .chain(silver.iter())
        .map(|(_, time)| time)
        .sum::<usize>();

    print_info(silver_text("Silver"), silver);
    print_info(gold_text("Gold"), gold);
    let unit = get_time_symbol();
    println!("\nTOTAL TIME: {}{unit}", total);
}

fn format_duration(duration: usize) -> String {
    let unit = get_time_symbol();
    format!("{}{}", duration, unit)
}

pub fn print_table(days: Vec<Result<BuildRes, Error>>, year: usize) {
    let max_name_len = days
        .iter()
        .map(|res| match res {
            Ok(br) => br.info.title.len(),
            Err(err) => err.title.len(),
        })
        .max()
        .unwrap_or(5);
    let max_part1_len = days
        .iter()
        .flatten()
        .map(|br| br.info.ans1.as_ref().unwrap_or(&"NA".to_string()).len())
        .max()
        .unwrap_or(5);
    let max_part2_len = days
        .iter()
        .flatten()
        .map(|br| br.info.ans2.as_ref().unwrap_or(&"NA".to_string()).len())
        .max()
        .unwrap_or(5);

    let max_part1_time_len = days
        .iter()
        .flatten()
        .map(|br| {
            br.time
                .0
                .map(format_duration)
                .unwrap_or("NA".to_string())
                .len()
        })
        .max()
        .unwrap_or(5);
    let max_part2_time_len = days
        .iter()
        .flatten()
        .map(|br| {
            br.time
                .1
                .map(format_duration)
                .unwrap_or("NA".to_string())
                .len()
        })
        .max()
        .unwrap_or(5);

    let day_header_len = max_name_len + 5;
    let part1_header_len = max_part1_len + 8 + max_part1_time_len;
    let part2_header_len = max_part2_len + 8 + max_part2_time_len;

    let max_total_len = day_header_len + part1_header_len + part2_header_len + 5;
    let title_length = max_total_len - 2;

    println!("╔{}╗", "═".repeat(max_total_len + 3));
    println!(
        "║ {:^title_length$}  ║",
        format!("🦀 Advent of Code {year} 🦀")
    );
    println!(
        "╠{}╦{}╦{}╣",
        "═".repeat(day_header_len + 2),
        "═".repeat(part1_header_len + 2),
        "═".repeat(part2_header_len + 2),
    );
    println!(
        "║ {:day_header_len$} ║ {:part1_header_len$} ║ {:part2_header_len$} ║",
        "Day", "Part 1", "Part 2"
    );
    println!(
        "╠{}╦{}╬{}╦{}╦{}╬{}╦{}╦{}╣",
        "═".repeat(4),
        "═".repeat(max_name_len + 2),
        "═".repeat(max_part1_len + 2),
        "═".repeat(max_part1_time_len + 2),
        "═".repeat(4),
        "═".repeat(max_part2_len + 2),
        "═".repeat(max_part2_time_len + 2),
        "═".repeat(4),
    );

    for day in days {
        match day {
            Ok(day) => {
                let part1_symbol = if day.info.correct1 { "✅" } else { "❌" };
                let part2_symbol = if day.info.correct2 { "✅" } else { "❌" };

                println!(
                    "║ {:>2} ║ {:max_name_len$} ║ {:max_part1_len$} ║ {:max_part1_time_len$} ║ {} ║ \
                     {:max_part2_len$} ║ {:max_part2_time_len$} ║ {} ║ ",
                    day.day,
                    day.info.title,
                    day.info.ans1.unwrap_or("NA".to_string()),
                    day.time.0.map(format_duration).unwrap_or("NA".to_string()),
                    part1_symbol,
                    day.info.ans2.unwrap_or("NA".to_string()),
                    day.time.1.map(format_duration).unwrap_or("NA".to_string()),
                    part2_symbol,
                );
            }
            Err(e) => {
                let available_space = max_total_len - day_header_len - 2;
                let mut s = e.r#type.to_string().replace('\n', " ");
                s.truncate(available_space);
                println!(
                    "║ {:>2} ║ {:max_name_len$} ║ {:available_space$} ║",
                    e.day, e.title, s
                );
            }
        }
    }
    println!(
        "╚{}╩{}╩{}╩{}╩{}╩{}╩{}╩{}╝",
        "═".repeat(4),
        "═".repeat(max_name_len + 2),
        "═".repeat(max_part1_len + 2),
        "═".repeat(max_part1_time_len + 2),
        "═".repeat(4),
        "═".repeat(max_part2_len + 2),
        "═".repeat(max_part2_time_len + 2),
        "═".repeat(4),
    );
}
