use clap::ArgMatches;

use crate::{
    error::AocError,
    tally::{
        ctx::PipelineCtx,
        fns::{
            convert_to_print_format, get_compiled_days, get_discovered_days, get_run_result,
            get_verified_days,
        },
        print_fns::print_table,
        util::{get_number_of_runs, get_possible_days, get_year_from_path},
    },
    util::file::get_root_path,
};

mod ctx;
mod fns;
mod print_fns;
mod types;
mod util;

pub async fn tally(matches: &ArgMatches) -> Result<(), AocError> {
    let number_of_runs = get_number_of_runs(matches)?;

    let root = get_root_path()?;
    let year = get_year_from_path(&root)?;
    let days = get_possible_days(year)?;

    let mut ctx = PipelineCtx::new(year, root.clone(), &days).await?;

    let discovered = get_discovered_days(&root, &days)?;
    let compiled = get_compiled_days(&mut ctx, discovered).await?;
    let verified = get_verified_days(&mut ctx, compiled).await?;
    let res = get_run_result(&mut ctx, verified, number_of_runs).await;

    let converted = convert_to_print_format(ctx, res);

    print_table(converted, year);

    Ok(())
}
