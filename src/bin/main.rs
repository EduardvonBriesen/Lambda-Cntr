extern crate lambda_cntr;

#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};



fn deploy(args: &ArgMatches) {
    lambda_cntr::pod_shell::main().expect("Failed to deploy cntr-pod");
}

fn main() {
    let deploy_command = SubCommand::with_name("deploy")
        .about("Deploy Cntr-Pod to Kubeneretes")
        .version(crate_version!())
        .author(crate_authors!("\n"));

    let matches = App::new("\u{03bb}-Cntr")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(deploy_command)
        .get_matches();

    match matches.subcommand() {
        ("deploy", Some(deploy_matches)) => deploy(deploy_matches),
        ("", None) => unreachable!(),
        _ => unreachable!(),
    };
}
