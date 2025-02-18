use clap::ArgMatches;

use crate::{error::AocError, util::file::get_root_path};

pub async fn token(matches: &ArgMatches) -> Result<(), AocError> {
    if let Some(token) = matches.get_one::<String>("set") {
        let mut path = get_root_path()?;
        path.push(".env");

        tokio::fs::write(path, format!("AOC_TOKEN={token}"))
            .await
            .expect("Couldn't write to file");
    } else {
        println!(
            "{}",
            dotenv::var("AOC_TOKEN").unwrap_or_else(|_| "Could not find token".to_string())
        );
    }
    Ok(())
}
