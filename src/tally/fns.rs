use std::{collections::HashMap, path::Path};

use indicatif::MultiProgress;

use crate::{
    error::AocError,
    tally::{
        ctx::PipelineCtx,
        print_fns::BuildRes,
        types::{CompiledDay, DayError, DiscoveredDay, ErrorTypes, RunDayResult, VerifiedDay},
        util::{compile_day, get_progressbar, prepare_args, run_day},
    },
};

pub fn get_discovered_days(root: &Path, days: &[usize]) -> Result<Vec<DiscoveredDay>, AocError> {
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
        let day_text = day_fmt(*day);
        if let Some((_name, folder)) = map.iter().find(|(name, _path)| name.contains(&day_text)) {
            matched_folders.push(DiscoveredDay {
                day: *day,
                folder: folder.clone(),
            })
        }
    }

    Ok(matched_folders)
}

pub async fn get_compiled_days(
    ctx: &mut PipelineCtx,
    discovered: Vec<DiscoveredDay>,
) -> Result<Vec<CompiledDay>, AocError> {
    let progress = get_progressbar(discovered.len() as u64);
    progress.set_message("compiling");

    let prepared = futures::future::join_all(discovered.into_iter().map(|d| {
        let progress = progress.clone();
        let ctx = &*ctx;
        async move {
            let args = prepare_args(ctx, &d.folder, d.day).await;
            progress.inc(1);
            args.map(|args| (d.day, args)).ok_or(d.day)
        }
    }))
    .await;

    let mut results = Vec::new();
    std::thread::scope(|scope| {
        let mut handles = Vec::new();

        for res in prepared.into_iter() {
            match res {
                Ok((day, args)) => {
                    handles.push(scope.spawn(move || compile_day(day, args)));
                }
                Err(day) => {
                    ctx.push_error(day, ErrorTypes::InputDownload);
                }
            }
        }

        for h in handles {
            match h.join().unwrap() {
                Ok(compiled) => {
                    results.push(compiled);
                }
                Err((day, err)) => {
                    ctx.push_error(day, err);
                }
            }
        }
    });

    Ok(results)
}

pub async fn get_verified_days(
    ctx: &mut PipelineCtx,
    compiled: Vec<CompiledDay>,
) -> Result<Vec<VerifiedDay>, AocError> {
    let progress = get_progressbar(compiled.len() as u64);
    progress.set_message("verifying");

    let mut out = Vec::new();

    for c in compiled {
        progress.inc(1);

        match ctx.infos.get(&c.day) {
            Some(info) if !info.is_unimplemented() => {
                out.push(VerifiedDay {
                    day: c.day,
                    expr: c.expr,
                    info: info.clone(),
                });
            }
            Some(_info) => ctx.push_error(c.day, ErrorTypes::MissingImplementation),
            None => ctx.push_error(c.day, ErrorTypes::MissingDay),
        }
    }

    Ok(out)
}

pub async fn get_run_result(
    ctx: &mut PipelineCtx,
    days: Vec<VerifiedDay>,
    num_runs: usize,
) -> Vec<RunDayResult> {
    let multi = MultiProgress::new();

    let mut results = Vec::new();
    std::thread::scope(|scope| {
        let mut handles = Vec::new();

        for day in days {
            let pb = multi.add(get_progressbar(num_runs as u64));
            let config = ctx.configs[&day.day].clone();
            handles.push(scope.spawn(move || {
                run_day(num_runs, day.expr, &config, &pb)
                    .map(|run_res| RunDayResult {
                        day: day.day,
                        info: day.info.clone(),
                        run: run_res,
                    })
                    .map_err(|e| (day.day, e))
            }));
        }

        for h in handles {
            match h.join().unwrap() {
                Ok(res) => {
                    results.push(res);
                }

                Err((day, err)) => {
                    ctx.push_error(day, err);
                }
            }
        }
    });

    results
}

pub fn convert_to_print_format(
    ctx: PipelineCtx,
    days: Vec<RunDayResult>,
) -> Vec<Result<BuildRes, DayError>> {
    let mut vec = days
        .into_iter()
        .map(|res| Ok::<BuildRes, DayError>(res.into()))
        .chain(ctx.errors.into_iter().map(Err::<BuildRes, DayError>))
        .collect::<Vec<Result<_, DayError>>>();

    vec.sort_unstable_by_key(|r| match r {
        Ok(res) => res.day,
        Err(e) => e.day,
    });
    vec
}
