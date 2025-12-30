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
        file::{
            day_path, download_input_file, get_parse_config, get_root_path, get_running_args,
            get_year_from_path,
        },
        get_day,
    },
};

pub async fn run(matches: &ArgMatches) -> Result<(), AocError> {
    let day = get_day(matches)?;
    let path = get_root_path()?;
    let year = get_year_from_path(&path)?;

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

    let ext = if args.common.files.len() == 1 {
        let v: String = args.common.files.keys().next().unwrap().to_owned();
        Some(v)
    } else if let Some(runner) = &args.runner
        && REGISTER.by_extension(runner).is_some()
    {
        Some(runner.to_owned())
    } else {
        None
    };

    let Some(ext) = ext else {
        let vals = args.common.files.keys().collect::<Vec<_>>();
        return Err(AocError::UnsupportedLanguage(format!(
            "Too many to pick from: {:?}",
            vals
        )));
    };
    let Some(compiler) = REGISTER.by_extension(&ext) else {
        return Err(AocError::UnsupportedLanguage(ext));
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
