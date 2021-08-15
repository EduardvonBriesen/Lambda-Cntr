extern crate lambda_cntr;

#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

fn deploy(args: &ArgMatches) {
    let container_id = args.value_of("id").unwrap().to_string();
    lambda_cntr::pod_shell::deploy_and_attach(container_id);
}

fn main() {
    let attach_command = SubCommand::with_name("attach")
        .about("Attach Cntr-Pod to Container in Kubeneretes")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("id")
                .help("container id")
                .required(true)
                .index(1),
        );

    let matches = App::new("\u{03bb}-Cntr")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(attach_command)
        .get_matches();

    match matches.subcommand() {
        ("attach", Some(attach_matches)) => deploy(attach_matches),
        ("", None) => unreachable!(),
        _ => unreachable!(),
    };
}
