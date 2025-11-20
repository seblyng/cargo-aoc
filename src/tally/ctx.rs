use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    error::AocError,
    tally::{
        fns::get_discovered_days,
        types::{DayError, ErrorTypes},
        util::{get_aoc_configs, get_aoc_infos},
    },
    task_config::Config,
    util::AocInfo,
};

pub struct PipelineCtx {
    pub year: usize,
    pub root: PathBuf,
    pub configs: HashMap<usize, Config>,
    pub infos: HashMap<usize, AocInfo>,
    pub errors: Vec<DayError>,
}

impl PipelineCtx {
    pub async fn new(year: usize, root: PathBuf, days: &[usize]) -> Result<Self, AocError> {
        let infos = get_aoc_infos(year, days).await?;
        let discovered = get_discovered_days(&root, days)?;
        let configs = get_aoc_configs(&root, &discovered);

        let errors = {
            let mut errors = Vec::new();
            let set = discovered.iter().map(|d| d.day).collect::<HashSet<_>>();

            for day in days {
                if !set.contains(day) {
                    errors.push(DayError {
                        day: *day,
                        info: infos[day].clone(),
                        error: ErrorTypes::MissingImplementation,
                    });
                }
            }

            errors
        };

        Ok(Self {
            year,
            root,
            configs,
            infos,
            errors,
        })
    }
    pub fn push_error(&mut self, day: usize, err: ErrorTypes) {
        let info = self.infos.get(&day).cloned().expect("Could not get info");
        self.errors.push(DayError {
            day,
            info,
            error: err,
        });
    }
}
