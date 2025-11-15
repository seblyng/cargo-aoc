use std::io::{BufRead, BufReader};

use chrono::prelude::*;
use clap::ArgMatches;
use duct::cmd;

#[cfg(feature = "submit")]
use crate::util::submit::{self, get_submit_task};
use crate::{
    assert::assert_answer,
    error::AocError,
    util::{
        file::{day_path, download_input_file, get_parse_config, get_root_path},
        get_day,
    },
};

fn get_input_file(matches: &ArgMatches) -> &str {
    if matches.get_flag("test") {
        "test"
    } else {
        "input"
    }
}

pub async fn run(matches: &ArgMatches) -> Result<(), AocError> {
    let day = get_day(matches)?;
    let path = get_root_path()?;
    let year = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i32>()
        .unwrap();

    let dir = day_path(&path, day).await?;

    if !dir.join("input").exists() {
        let current_year = Utc::now().year();
        let current_month = Utc::now().month();

        if year < 2015 || year > current_year {
            return Err(AocError::InvalidYear);
        }
        if year == current_year && current_month < 12 {
            return Err(AocError::InvalidMonth);
        }

        download_input_file(day, year, &dir).await?;
    }

    let input = get_input_file(matches);

    let trailing_args = matches
        .get_many::<&str>("args")
        .unwrap_or_default()
        .copied()
        .collect::<Vec<_>>();

    let args = std::iter::once("run")
        .chain(trailing_args)
        .chain(["--color", "always"])
        .chain(std::iter::once(input));

    let cmd = cmd("cargo", args);
    let reader = cmd.dir(&dir).stderr_to_stdout().reader()?;

    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let mut out = String::new();
    while let Some(Ok(line)) = lines.next() {
        println!("{}", line);
        out.push_str(&line);
        out.push('\n');
    }

    if matches.get_flag("assert") {
        let parse_file = get_parse_config(&path, &dir);
        assert_answer(&out, day, year, parse_file).await?;
    }

    // Only try to submit if the submit flag is passed
    #[cfg(feature = "submit")]
    if let Some(task) = get_submit_task(matches).transpose()? {
        let parse_file = get_parse_config(&path, &dir);
        let output = submit::submit(&out, task, day, year, parse_file).await?;
        println!("Task {}: {}", task, output);
    }
    Ok(())
}
