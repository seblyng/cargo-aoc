use std::path::{Path, PathBuf};

use clap::ArgMatches;

use crate::error::AocError;

async fn setup_template_project(year: i32) -> Result<PathBuf, AocError> {
    if Path::new(&format!("{year}")).exists() {
        return Err(AocError::SetupExists);
    }

    let year = format!("{}", year);
    let path = Path::new(&year);
    tokio::fs::create_dir(&path).await?;

    let template_dir = format!("{}/template", env!("CARGO_MANIFEST_DIR"));

    for day in 1..=25 {
        let day = format!("day_{:0>2}", day);
        tokio::process::Command::new("cargo")
            .args(["new", &day])
            .current_dir(&year)
            .output()
            .await?;

        tokio::fs::copy(
            format!("{template_dir}/template.rs"),
            format!("{year}/{day}/src/main.rs"),
        )
        .await?;
    }
    Ok(path.to_path_buf())
}

async fn get_session_token(year_path: &Path) -> Result<(), AocError> {
    if dotenv::var("AOC_TOKEN").is_err() {
        println!("Paste session token here for automatic download of input files");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if !input.is_empty() {
            let env_file = year_path.join(".env");
            tokio::fs::write(env_file, format!("AOC_TOKEN={input}"))
                .await
                .expect("Couldn't write to file");
        }
    }
    Ok(())
}

fn get_year(matches: &ArgMatches) -> Result<i32, AocError> {
    let year = matches
        .get_one::<String>("year")
        .ok_or(AocError::ArgMatches)?;
    if year.chars().count() == 2 {
        Ok(format!("20{}", year).parse()?)
    } else {
        Ok(year.parse()?)
    }
}

pub async fn setup(args: &ArgMatches) -> Result<(), AocError> {
    let year = get_year(args)?;

    let path = setup_template_project(year).await?;
    get_session_token(&path).await?;
    Ok(())
}
