use std::{collections::HashMap, io::Read, path::Path};

use crate::{
    error::AocError,
    language::{Common, REGISTER, RunningArgs},
    tally::{
        ctx::PipelineCtx,
        types::{Answer, CompiledDay, DiscoveredDay, ErrorTypes, RunRes},
    },
    task_config::Config,
    util::{
        AocInfo,
        file::{download_input_file, find_file, get_parse_config},
        get_day_title_and_answers, get_time_symbol,
    },
};
use chrono::prelude::*;
use clap::ArgMatches;
use duct::Expression;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};

pub fn get_progressbar(len: u64) -> ProgressBar {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {msg}... {bar:40.cyan/blue} {pos:>7}/{len:7}",
    )
    .unwrap()
    .progress_chars("##-");

    ProgressBar::new(len).with_style(sty)
}

pub fn get_possible_days(year: usize) -> Result<Vec<usize>, AocError> {
    let now = chrono::Utc::now();
    const LAST_DAY_2025: usize = 12;

    if (2015..=2024).contains(&year) {
        return Ok((1..=25).collect());
    }

    if year as i32 == now.year() {
        if now.month() == 12 {
            let day = now.day() as usize;
            Ok((1..=day.min(LAST_DAY_2025)).collect())
        } else {
            Err(AocError::InvalidMonth)
        }
    } else {
        Ok((1..=LAST_DAY_2025).collect())
    }
}

pub fn get_aoc_configs(year: &Path, days: &[DiscoveredDay]) -> HashMap<usize, Config> {
    days.iter()
        .map(|d| {
            let config = get_parse_config(year, &d.folder);
            (d.day, config)
        })
        .collect::<HashMap<_, _>>()
}

pub async fn get_aoc_infos(
    year: usize,
    days: &[usize],
) -> Result<HashMap<usize, AocInfo>, AocError> {
    let progress = get_progressbar(days.len() as u64);
    progress.set_message("fetching day info");

    let tasks = days
        .iter()
        .map(|d| {
            let progress = progress.clone();
            async move {
                let res = get_day_title_and_answers(*d as u32, year as u32)
                    .await
                    .map(|aoc_info| (*d, aoc_info));
                progress.inc(1);
                res
            }
        })
        .collect::<Vec<_>>();

    join_all(tasks)
        .await
        .into_iter()
        .collect::<Result<HashMap<_, _>, _>>()
}

pub fn get_number_of_runs(matches: &ArgMatches) -> Result<usize, AocError> {
    Ok(matches
        .get_one::<String>("runs")
        .ok_or(AocError::ArgMatches)?
        .parse()?)
}

pub async fn prepare_args(ctx: &PipelineCtx, day_path: &Path, day: usize) -> Option<RunningArgs> {
    let main = find_file(day_path, "main", Some(&REGISTER.compiler_exts()))?;
    let input_path = day_path.join("input");

    if !input_path.exists()
        && (download_input_file(day as u32, ctx.year as i32, day_path).await).is_err()
    {
        return None;
    }

    Some(RunningArgs {
        release: true,
        arguments: vec![],
        common: Common {
            day_folder: day_path.to_path_buf(),
            input_file: input_path,
            day: day as i32,
            file: main,
            root_folder: ctx.root.to_path_buf(),
        },
    })
}

pub fn compile_day(
    day: usize,
    args: RunningArgs,
    progress: &ProgressBar,
) -> Result<CompiledDay, (usize, ErrorTypes)> {
    let Some(ext) = args
        .common
        .file
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_string())
    else {
        progress.inc(1);
        return Err((day, ErrorTypes::MissingExtension));
    };

    let Some(compiler) = REGISTER.compiler_by_extension(&ext) else {
        progress.inc(1);
        return Err((day, ErrorTypes::Unsupported(ext)));
    };

    let res = match compiler.compile(args) {
        Ok(expr) => Ok(CompiledDay { day, expr }),
        Err(err) => Err((day, ErrorTypes::Compiler(err.to_string()))),
    };
    progress.inc(1);
    res
}

pub fn run_day(
    num_runs: usize,
    expr: Expression,
    config: &Config,
    progress: &ProgressBar,
) -> Result<RunRes, ErrorTypes> {
    let mut vec = Vec::new();
    for _ in 0..num_runs {
        let expr = expr.clone();

        let (mut r, w) = std::io::pipe().unwrap();
        let (mut stdoutr, stdoutw) = std::io::pipe().unwrap();
        let out = expr
            .unchecked()
            .stderr_file(w)
            .stdout_file(stdoutw)
            .run()
            .map_err(ErrorTypes::DuctError)?;

        if !out.status.success() {
            progress.finish_and_clear();
            let mut vec = Vec::new();
            r.read_to_end(&mut vec).expect("reading to vec");
            let text = std::str::from_utf8(&vec)
                .expect("Getting stderr")
                .to_owned();

            return Err(ErrorTypes::Runtime(text));
        }

        let mut stdout = Vec::new();
        stdoutr.read_to_end(&mut stdout).expect("reading to vec");
        let stdout = std::str::from_utf8(&stdout).expect("error converting stdout to text");

        let (Some(p1), p2) = config.get_answers(stdout) else {
            progress.finish_and_clear();
            return Err(ErrorTypes::GetAnswers);
        };

        let (t1, t2) = config.get_times(stdout);

        vec.push(((p1, t1), (p2, t2)));

        progress.inc(1);
    }
    let p1_time = vec
        .iter()
        .map(|((_p1, t1), _p2)| t1)
        .copied()
        .collect::<Option<Vec<usize>>>()
        .map(|vals| vals.into_iter().sum::<usize>() / num_runs);

    let p2_time = vec
        .iter()
        .map(|(_p1, (_p2, t2))| t2)
        .copied()
        .collect::<Option<Vec<usize>>>()
        .map(|vals| vals.into_iter().sum::<usize>() / num_runs);
    let res = RunRes {
        p1: Answer {
            value: Some(vec[0].0.0.clone()),
            time: p1_time,
        },
        p2: Answer {
            value: vec[0].1.0.clone(),
            time: p2_time,
        },
    };

    Ok(res)
}

pub fn format_duration(duration: usize) -> String {
    let unit = get_time_symbol();
    format!("{}{}", duration, unit)
}
