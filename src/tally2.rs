use std::{
    collections::HashMap,
    io::Read,
    num,
    path::{Path, PathBuf},
    sync::mpsc::{self},
};

use chrono::prelude::*;
use clap::ArgMatches;
use duct::Expression;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};

type Sender = mpsc::Sender<(usize, ErrorTypes)>;

use crate::{
    error::AocError,
    language::{Common, Compile, REGISTER, RunningArgs},
    util::{
        AocInfo,
        file::{download_input_file, find_file, get_root_path},
        get_day_title_and_answers,
        tally_util::{self, get_number_of_runs, parse_get_times, parse_get_times2},
    },
};

#[derive(Debug)]
enum ErrorTypes {
    MissingDay,
    InputDownload,
    Compiler(String),
    Runtime(String),
    MissingImplementation,
    GetTime,
    GetAnswers,
}

#[derive(Debug)]
struct Answer {
    value: String,
    time: usize,
}
#[derive(Debug)]
struct RunRes {
    p1: Answer,
    p2: Answer,
}

async fn run_day(
    num_runs: usize,
    year: usize,
    day: usize,
    expr: Expression,
    s: &Sender,
) -> Result<Option<RunRes>, AocError> {
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
            .expect("duct panic");

        if !out.status.success() {
            let mut vec = Vec::new();
            r.read_to_end(&mut vec).expect("reading to vec");
            let text = std::str::from_utf8(&vec)
                .expect("Getting stderr")
                .to_owned();
            s.send((day, ErrorTypes::Runtime(text))).unwrap();
            return Ok(None);
        }

        let mut stdout = Vec::new();
        stdoutr.read_to_end(&mut stdout).expect("reading to vec");

        let Ok((t1, t2)) = parse_get_times2(&stdout) else {
            s.send((day, ErrorTypes::GetTime)).unwrap();
            return Ok(None);
        };
        let t2 = t2.unwrap_or_default();
        let ((Some(p1), p2)) = tally_util::parse_get_answers2(&stdout) else {
            s.send((day, ErrorTypes::GetAnswers)).unwrap();
            return Ok(None);
        };
        let p2 = p2.unwrap_or_default();

        vec.push(((p1, t1), (p2, t2)));
    }

    let res = RunRes {
        p1: Answer {
            value: vec[0].0.0.clone(),
            time: vec.iter().map(|(p1, p2)| p1.1).sum::<usize>() / num_runs,
        },
        p2: Answer {
            value: vec[0].1.0.clone(),
            time: vec.iter().map(|(p1, p2)| p2.1).sum::<usize>() / num_runs,
        },
    };

    Ok(Some(res))
}

pub async fn tally(matches: &ArgMatches) -> Result<(), AocError> {
    let number_of_runs = get_number_of_runs(matches)?;

    let res: Vec<(usize, ErrorTypes)> = Vec::new();
    let (s, r) = mpsc::channel::<(usize, ErrorTypes)>();

    let root_folder = get_root_path()?;
    let year = root_folder
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()?;

    let days = get_possible_days(year)?;
    let folders = get_day_folders(&root_folder, days, &s)?;
    let folders = folders.collect::<Vec<_>>();

    let days = folders.iter().map(|(day, _path)| *day).collect::<Vec<_>>();
    let map = get_aoc_infos(year, &days).await?;

    let compiled = compile_days(&root_folder, year, folders.into_iter(), &s).await?;
    let mut verified = verify_days(
        compiled.map(|(day, expr)| (day, expr, map[&day].clone())),
        &s,
    )
    .await?;

    verified.next();
    let (day, expr, info) = verified.next().unwrap();
    // dbg!(&day, &expr, &info);
    let r = run_day(number_of_runs, year, day, expr, &s).await?;
    // dbg!(r);

    // while let Ok(res) = r.try_recv() {
    //     match res {
    //         (_day, err) if !matches!(err, ErrorTypes::MissingDay) => {
    //             dbg!(err);
    //         }
    //         _ => {}
    //     };
    // }

    Ok(())
}

async fn verify_days(
    days: impl Iterator<Item = (usize, duct::Expression, AocInfo)>,
    s: &Sender,
) -> Result<impl Iterator<Item = (usize, duct::Expression, AocInfo)>, AocError> {
    let progress = get_progressbar(days.size_hint().0 as u64);
    progress.set_message("verifying");

    let progess = &progress;
    Ok(days
        .into_iter()
        .map(move |(day, expr, info)| {
            let res = if info.is_unimplemented() {
                s.send((day, ErrorTypes::MissingImplementation)).unwrap();
                None
            } else {
                Some((day, expr, info))
            };
            progress.inc(1);
            res
        })
        .flatten())
}

async fn get_aoc_infos(year: usize, days: &[usize]) -> Result<HashMap<usize, AocInfo>, AocError> {
    let progress = get_progressbar(days.len() as u64);
    progress.set_message("fetching day info");

    let tasks = days
        .into_iter()
        .map(|day| {
            let progress = progress.clone();
            async move {
                let res = get_day_title_and_answers(*day as u32, year as u32)
                    .await
                    .map(|aoc_info| (*day, aoc_info));
                progress.inc(1);
                res
            }
        })
        .collect::<Vec<_>>();

    Ok(join_all(tasks)
        .await
        .into_iter()
        .collect::<Result<HashMap<_, _>, _>>()?)
}

async fn prepare_args(
    root: &Path,
    year: usize,
    day: usize,
    folder: &Path,
    s: &Sender,
) -> Option<RunningArgs> {
    let main = find_file(folder, "main")?;
    let input_path = folder.join("input");

    if !input_path.exists() {
        if let Err(_) = download_input_file(day as u32, year as i32, folder).await {
            let _ = s.send((day, ErrorTypes::InputDownload));
            return None;
        }
    }

    Some(RunningArgs {
        release: true,
        arguments: vec![],
        common: Common {
            day_folder: folder.to_path_buf(),
            input_file: input_path,
            day: day as i32,
            file: main,
            root_folder: root.to_path_buf(),
        },
    })
}

fn compile_day(day: usize, args: RunningArgs, s: &Sender) -> Option<duct::Expression> {
    let ext = args.common.file.extension()?.to_str()?.to_string();
    let compiler = REGISTER.compiler_by_extension(&ext)?;

    match compiler.compile(args) {
        Ok(expr) => Some(expr),
        Err(e) => {
            let _ = s.send((day, ErrorTypes::Compiler(e.to_string())));
            None
        }
    }
}

async fn compile_days(
    root: &Path,
    year: usize,
    days: impl Iterator<Item = (usize, PathBuf)>,
    s: &Sender,
) -> Result<impl Iterator<Item = (usize, duct::Expression)>, AocError> {
    let progress = get_progressbar(days.size_hint().0 as _);
    progress.set_message("compiling");

    let tasks = days.into_iter().map(|(day, folder)| {
        let s = s.clone();
        let root = root.to_path_buf();
        let progress = progress.clone();

        tokio::spawn(async move {
            let res = if let Some(args) = prepare_args(&root, year, day, &folder, &s).await {
                compile_day(day, args, &s).map(|r| (day, r))
            } else {
                None
            };
            progress.inc(1);
            res
        })
    });

    Ok(futures::future::join_all(tasks)
        .await
        .into_iter()
        .filter_map(|res| res.ok().flatten()))
}

pub fn get_day_folders(
    root: &Path,
    days: impl Iterator<Item = usize>,
    s: &Sender,
) -> Result<impl Iterator<Item = (usize, PathBuf)>, AocError> {
    let day_fmt = |day: usize| format!("{:02}", day);

    let mut map = HashMap::new();
    for entry in std::fs::read_dir(root)?
        .flatten()
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
    {
        let name = entry.file_name().into_string().unwrap();
        let path = entry.path();
        map.insert(name, path);
    }

    let mut matched_folders = Vec::new();

    for day in days {
        let day_text = day_fmt(day);
        if let Some((_name, folder)) = map.iter().find(|(name, _path)| name.contains(&day_text)) {
            matched_folders.push((day, folder.clone()));
        } else {
            s.send((day, ErrorTypes::MissingDay)).unwrap();
        }
    }

    Ok(matched_folders.into_iter())
}
pub fn get_possible_days(year: usize) -> Result<impl Iterator<Item = usize>, AocError> {
    let now = chrono::Utc::now();
    const LAST_DAY_2025: usize = 13;

    if (2015..=2024).contains(&year) {
        return Ok(1..=25);
    }

    if year as i32 == now.year() {
        if now.month() < 12 {
            Err(AocError::InvalidMonth)
        } else {
            Ok(1..=now.day() as usize)
        }
    } else {
        Ok(1..=LAST_DAY_2025)
    }
}

fn get_progressbar(len: u64) -> ProgressBar {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {msg}... {bar:40.cyan/blue} {pos:>7}/{len:7}",
    )
    .unwrap()
    .progress_chars("##-");

    ProgressBar::new(len).with_style(sty)
}
