use std::path::Path;

use regex::Regex;
use serde::Deserialize;
use serde_regex;

#[derive(Debug, Deserialize)]
pub struct TaskConfig {
    #[serde(with = "serde_regex")]
    pub answer: Regex,

    #[serde(default, with = "serde_regex")]
    pub time: Option<Regex>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub task_one: TaskConfig,
    pub task_two: TaskConfig,
}

impl Config {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let toml_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&toml_str).map_err(std::io::Error::other)?;
        Ok(config)
    }

    pub fn get_times(&self, output: &str) -> (Option<usize>, Option<usize>) {
        let strip = strip_ansi_escapes::strip(output);
        let text = std::str::from_utf8(&strip).unwrap();

        let parse = |line: &str, regex: &Regex| {
            regex
                .captures(line)
                .iter()
                .next()
                .and_then(|cap| cap[1].parse::<usize>().ok())
        };

        let mut ans1: Option<usize> = None;
        let mut ans2: Option<usize> = None;

        for line in text.split('\n').filter(|line| !line.is_empty()) {
            if let Some(time) = &self.task_one.time
                && let Some(ans) = parse(line, &time)
                && ans1.is_none()
            {
                ans1 = Some(ans);
            } else if let Some(time) = &self.task_two.time
                && let Some(ans) = parse(line, time)
                && ans2.is_none()
            {
                ans2 = Some(ans);
            }
        }
        (ans1, ans2)
    }

    pub fn get_answers(&self, output: &str) -> (Option<String>, Option<String>) {
        let strip = strip_ansi_escapes::strip(output);
        let text = std::str::from_utf8(&strip).unwrap();

        let parse = |line: &str, regex: &Regex| {
            regex
                .captures(line)
                .iter()
                .next()
                .map(|cap| cap[1].to_string())
        };

        let mut ans1: Option<String> = None;
        let mut ans2: Option<String> = None;

        for line in text.split('\n').filter(|line| !line.is_empty()) {
            if let Some(ans) = parse(line, &self.task_one.answer)
                && ans1.is_none()
            {
                ans1 = Some(ans);
            } else if let Some(ans) = parse(line, &self.task_two.answer)
                && ans2.is_none()
            {
                ans2 = Some(ans);
            }
        }
        (ans1, ans2)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            task_one: TaskConfig {
                answer: Regex::new(r"^(.*)$").unwrap(),
                time: None,
            },
            task_two: TaskConfig {
                answer: Regex::new(r"^(.*)$").unwrap(),
                time: None,
            },
        }
    }
}
