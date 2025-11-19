use std::io::{BufRead, BufReader};

use chrono::prelude::*;
use clap::ArgMatches;

#[cfg(feature = "submit")]
use crate::util::submit::{self, get_submit_task};
use crate::{
    assert::assert_answer,
    error::AocError,
    language::REGISTER,
    util::{
        file::{day_path, download_input_file, get_parse_config, get_root_path, get_running_args},
        get_day,
    },
};

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

    let args = get_running_args(matches).await?;
    let ext = args.common.file.extension().unwrap().to_str().unwrap();
    let Some(compiler) = REGISTER.by_extension(ext) else {
        return Err(AocError::UnsupportedLanguage(ext.to_owned()));
    };

    let reader = compiler.execute(args)?.stderr_to_stdout().reader()?;

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
