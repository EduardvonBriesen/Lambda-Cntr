extern crate lambda_cntr;

#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::env;

fn attach(args: &ArgMatches) {
    env::set_var("POD_NAME", args.value_of("pod_name").unwrap().to_string());
    env::set_var(
        "CONTAINER_NAME",
        args.value_of("container_name").unwrap().to_string(),
    );
    env::set_var("NAMESPACE", args.value_of("namespace").unwrap().to_string());
    env::set_var("RUST_LOG", args.value_of("log-level").unwrap().to_string());
    env::set_var("CNTR_IMAGE", args.value_of("image").unwrap().to_string());
    env::set_var(
        "SOCKET_PATH",
        args.value_of("socket-path").unwrap().to_string(),
    );
    lambda_cntr::kube_controller::deploy_and_attach();
}

fn execute(args: &ArgMatches) {
    env::set_var("POD_NAME", args.value_of("pod_name").unwrap().to_string());
    env::set_var(
        "CONTAINER_NAME",
        args.value_of("container_name").unwrap().to_string(),
    );
    env::set_var("CMD", args.value_of("command").unwrap().to_string());
    env::set_var("NAMESPACE", args.value_of("namespace").unwrap().to_string());
    env::set_var("RUST_LOG", args.value_of("log-level").unwrap().to_string());
    env::set_var("CNTR_IMAGE", args.value_of("image").unwrap().to_string());
    env::set_var(
        "SOCKET_PATH",
        args.value_of("socket-path").unwrap().to_string(),
    );
    if args.is_present("docker") {
        env::set_var("MOUNT_PATH", "/run/docker/docker.sock");
    } else {
        env::set_var("MOUNT_PATH", "/run/containerd/containerd.sock");
    }
    lambda_cntr::kube_controller::deploy_and_execute();
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
            Arg::with_name("container_name")
                .help("Specify the container in the target Pod")
                .index(2)
                .default_value(""),
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
            Arg::with_name("log-level")
                .help("Set the logging level (e.g. \"info,kube=debug\")")
                .short("l")
                .long("log-level")
                .takes_value(true)
                .default_value("info"),
        )
        .arg(
            Arg::with_name("image")
                .help("Set your container image")
                .short("i")
                .long("image")
                .takes_value(true)
                .default_value("onestone070/lambda-cntr"),
        )
        .arg(
            Arg::with_name("socket-path")
                .help("Path to the socket of the container engine on your node (e.g. \"/run/k3s/containerd/containerd.sock\")")
                .short("s")
                .long("socket-path")
                .takes_value(true)
                .required(true),
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
            Arg::with_name("log-level")
                .help("Set the logging level (e.g. \"info,kube=debug\")")
                .short("l")
                .long("log-level")
                .takes_value(true)
                .default_value("info"),
        )
        .arg(
            Arg::with_name("image")
                .help("Set your container image")
                .short("i")
                .long("image")
                .takes_value(true)
                .default_value("onestone070/lambda-cntr"),
        )
        .arg(
            Arg::with_name("socket-path")
                .help("Path to the socket of the container engine on your node (e.g. \"/run/k3s/containerd/containerd.sock\")")
                .short("s")
                .long("socket-path")
                .takes_value(true)
                .required(true),
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
