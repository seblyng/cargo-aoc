use table_generator::{Column, Table};

use crate::{
    tally::{
        types::{DayError, RunDayResult},
        util::format_duration,
    },
    util::get_time_symbol,
};

#[derive(Debug, Default)]
pub struct TableInfo {
    pub title: String,
    pub ans1: Option<String>,
    pub ans2: Option<String>,

    pub correct1: bool,
    pub correct2: bool,
}

#[derive(Debug, Default)]
pub struct Time(pub Option<usize>, pub Option<usize>);

#[derive(Debug)]
pub struct BuildRes {
    pub day: usize,
    pub info: TableInfo,
    pub time: Time,
}

impl From<RunDayResult> for BuildRes {
    fn from(res: RunDayResult) -> Self {
        let is_correct = |ans: &Option<String>, real: &Option<String>| {
            ans.is_some() && real.is_some() && ans == real
        };

        let table_info = TableInfo {
            title: res.info.title,
            ans1: res.run.p1.value.clone(),
            ans2: res.run.p2.value.clone(),
            correct1: is_correct(&res.run.p1.value, &res.info.part1_answer),
            correct2: is_correct(&res.run.p2.value, &res.info.part2_answer),
        };

        BuildRes {
            day: res.day,
            info: table_info,
            time: Time(res.run.p1.time, res.run.p2.time),
        }
    }
}

fn create_rows(day: &BuildRes) -> (Vec<String>, Vec<String>) {
    let ans1 = day.info.ans1.clone().unwrap_or("NA".to_string());
    let time1 = day.time.0.map(format_duration).unwrap_or("NA".to_string());
    let part1_symbol = if day.info.correct1 {
        "✅".into()
    } else {
        "❌".into()
    };

    let ans2 = day.info.ans2.clone().unwrap_or("NA".to_string());
    let time2 = day.time.1.map(format_duration).unwrap_or("NA".to_string());
    let part2_symbol = if day.info.correct2 {
        "✅".into()
    } else {
        "❌".into()
    };
    (
        vec![ans1, time1, part1_symbol],
        vec![ans2, time2, part2_symbol],
    )
}

pub fn print_table(days: Vec<Result<BuildRes, DayError>>, year: usize) {
    let mut table = Table::new(format!("Advent of Code {year}"));

    let mut day_col = Column::new("Day");
    let mut part1_col = Column::new("Part 1");
    let mut part2_col = Column::new("Part 2");

    for (i, day) in days.into_iter().enumerate() {
        match day {
            Ok(day) => {
                day_col.add_row(vec![day.day.to_string(), day.info.title.clone()]);
                let (p1, p2) = create_rows(&day);
                part1_col.add_row(p1);
                part2_col.add_row(p2);
            }
            Err(e) => {
                day_col.add_row(vec![e.day.to_string(), e.info.title.clone()]);
                let text = e.error.to_string().replace('\n', " ");
                table.add_span(i, 1..=2, text);

                // Need to add a dummy column. the span will override it.
                part1_col.add_row(vec!["", "", ""]);
                part2_col.add_row(vec!["", "", ""]);
            }
        }
    }

    table.add_column(day_col);
    table.add_column(part1_col);
    table.add_column(part2_col);
    println!("{}", table);
}

// TODO: Fix this. Maybe it does not need to be a part of the first iteration
#[allow(dead_code)]
pub fn print_info(
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
