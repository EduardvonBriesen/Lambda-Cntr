extern crate lambda_cntr;

#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::env;

fn attach(args: &ArgMatches) {
    let pod_name = args.value_of("pod_name").unwrap().to_string();
    let mut container_name = "".to_string();
    if args.is_present("container_name") {
        container_name = args.value_of("container_name").unwrap().to_string();
    }
    let namespace = args.value_of("namespace").unwrap().to_string();
    let image = args.value_of("image").unwrap().to_string();
    let socket = args.value_of("socket-path").unwrap().to_string();

    env::set_var("RUST_LOG", "info");

    match lambda_cntr::kube_controller::deploy_and_attach(
        pod_name,
        container_name,
        namespace,
        image,
        socket,
    ) {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }
}

fn execute(args: &ArgMatches) {
    let pod_name = args.value_of("pod_name").unwrap().to_string();
    let mut container_name = "".to_string();
    if args.is_present("container_name") {
        container_name = args.value_of("container_name").unwrap().to_string();
    }
    let namespace = args.value_of("namespace").unwrap().to_string();
    let cmd = args.value_of("command").unwrap().to_string();
    let image = args.value_of("image").unwrap().to_string();
    let socket = args.value_of("socket-path").unwrap().to_string();

    env::set_var("RUST_LOG", "info");

    match lambda_cntr::kube_controller::deploy_and_execute(
        pod_name,
        container_name,
        namespace,
        cmd,
        image,
        socket,
    ) {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }
}

fn main() {
    let attach_command = SubCommand::with_name("attach")
        .about("Attach \u{03bb}-Cntr-Pod to Container in Kubernetes")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("pod_name")
                .help("Pod Name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("container_name")
                .help("Specify the container in the target Pod")
                .index(2),
        )
        .arg(
            Arg::with_name("namespace")
                .help("Specify th namespace of the target Pod")
                .short("n")
                .long("namespace")
                .takes_value(true)
                .default_value("default"),
        )
        .arg(
            Arg::with_name("image")
                .help("Set your container image")
                .short("i")
                .long("image")
                .env("CNTR_IMAGE")
                .takes_value(true)
                .default_value("onestone070/lambda-cntr:latest"),
        )
        .arg(
            Arg::with_name("socket-path")
                .help("Path to the socket of the container engine on your node")
                .short("s")
                .long("socket-path")
                .env("SOCKET_PATH")
                .takes_value(true)
                .default_value("/run/containerd/containerd.sock"),
        );

    let execute_command = SubCommand::with_name("execute")
        .about("Execute command in Container")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("pod_name")
                .help("Pod Name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("command")
                .help("Command")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("container_name")
                .help("Specify the container in the target Pod")
                .index(3)
                .default_value(""),
        )
        .arg(
            Arg::with_name("namespace")
                .help("Namespace of container")
                .short("n")
                .long("namespace")
                .takes_value(true)
                .default_value("default"),
        )
        .arg(
            Arg::with_name("image")
                .help("Set your container image")
                .short("i")
                .long("image")
                .env("CNTR_IMAGE")
                .takes_value(true)
                .default_value("onestone070/lambda-cntr:latest"),
        )
        .arg(
            Arg::with_name("socket-path")
                .help("Path to the socket of the container engine on your node")
                .short("s")
                .long("socket-path")
                .env("SOCKET_PATH")
                .takes_value(true)
                .default_value("/run/containerd/containerd.sock"),
        );

    let matches = App::new("\u{03bb}-Cntr")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(attach_command)
        .subcommand(execute_command)
        .get_matches();

    match matches.subcommand() {
        ("attach", Some(attach_matches)) => attach(attach_matches),
        ("execute", Some(execute_command)) => execute(execute_command),
        ("", None) => unreachable!(),
        _ => unreachable!(),
    };
}
