extern crate lambda_cntr;

#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

fn attach(args: &ArgMatches) {
    let container_id = args.value_of("pod_name").unwrap().to_string();
    let namespace = args.value_of("namespace").unwrap().to_string();
    lambda_cntr::kube_controller::deploy_and_attach(container_id, namespace);
}

fn main() {
    let attach_command = SubCommand::with_name("attach")
        .about("Attach Cntr-Pod to Container in Kubeneretes")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("pod_name")
                .help("Pod Name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("namespace")
                .help("Namespace of container")
                .short("n")
                .long("namespace")
                .takes_value(true)
                .default_value("default"),
        );

    let matches = App::new("\u{03bb}-Cntr")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(attach_command)
        .get_matches();

    match matches.subcommand() {
        ("attach", Some(attach_matches)) => attach(attach_matches),
        ("", None) => unreachable!(),
        _ => unreachable!(),
    };
}
