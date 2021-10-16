use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use log::{error, info};
use std::collections::HashMap;

use crate::json_builder;
use kube::{
    api::{
        Api, AttachParams, AttachedProcess, DeleteParams, ListParams, PostParams, ResourceExt,
        WatchEvent,
    },
    Client,
};
use std::env;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

fn main() {}

#[tokio::main]
pub async fn deploy_and_attach() -> anyhow::Result<()> {
    let namespace = env::var("NAMESPACE").expect("No namespace specified!");
    let container_name = env::var("CONTAINER_ID").expect("No container specified!");

    env_logger::init();
    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let pod: Pod = pods.get(&container_name).await?;

    if pod.name().is_empty() {
        error!(
            "No pod \"{}\" found in namespace \"{}\"!",
            container_name,
            namespace.clone()
        );
    } else {
        let id = get_container_id(pod).await?;
        deploy(&pods).await?;
        attach(&pods, id.to_string()).await?;
        delete(&pods).await?;
    }

    Ok(())
}

#[tokio::main]
pub async fn deploy_and_execute() -> anyhow::Result<()> {
    let namespace = env::var("NAMESPACE").expect("No namespace specified!");
    let container_name = env::var("CONTAINER_ID").expect("No container specified!");
    let cmd = env::var("CMD").expect("No command specified!");

    info!("Command {}", cmd);

    env_logger::init();
    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let pod: Pod = pods.get(&container_name).await?;

    if pod.name().is_empty() {
        error!(
            "No pod \"{}\" found in namespace \"{}\"!",
            container_name,
            namespace.clone()
        );
    } else {
        let id = get_container_id(pod).await?;
        deploy(&pods).await?;
        execute(&pods, id.to_string(), cmd).await?;
    }

    Ok(())
}

async fn deploy(pods: &Api<Pod>) -> anyhow::Result<()> {
    let cntr_pod = json_builder::get_json().expect("Unable to parse json");
    let cntr_pod = serde_json::from_value(cntr_pod).expect("Unable to parse json");

    let p = pods.get("lambda-cntr").await;
    match p {
        Ok(_p) => info!("Lambda-Cntr-Pod already exist, attaching ..."),
        Err(_p) => {
            // Stop on error including a pod already exists or is still being deleted.
            info!("Lambda-Cntr-Pod doesn't exist, creating ...");
            pods.create(&PostParams::default(), &cntr_pod).await?;
            // Wait until the pod is running, otherwise we get 500 error.
            let lp = ListParams::default()
                .fields("metadata.name=lambda-cntr")
                .timeout(20);
            let mut stream = pods.watch(&lp, "0").await?.boxed();
            while let Some(status) = stream.try_next().await? {
                match status {
                    WatchEvent::Added(o) => {
                        info!("Added {}", o.name());
                    }
                    WatchEvent::Modified(o) => {
                        let s = o.status.as_ref().expect("status exists on pod");
                        if s.phase.clone().unwrap_or_default() == "Running" {
                            info!("Ready to attach to {}", o.name());
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    };

    Ok(())
}

async fn attach(pods: &Api<Pod>, id: String) -> anyhow::Result<()> {
    let ap = AttachParams::interactive_tty();
    let mut attached = pods.exec("lambda-cntr", vec!["/bin/bash"], &ap).await?;

    // The received streams from `AttachedProcess`
    let mut stdin_writer = attached.stdin().unwrap();
    let mut stdout_reader = attached.stdout().unwrap();

    let s = format!("cntr attach {}\n", &id);
    stdin_writer.write(s.as_bytes()).await?;

    // > For interactive uses, it is recommended to spawn a thread dedicated to user input and use blocking IO directly in that thread.
    // > https://docs.rs/tokio/0.2.24/tokio/io/fn.stdin.html
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    // pipe current stdin to the stdin writer from ws
    tokio::spawn(async move {
        tokio::io::copy(&mut stdin, &mut stdin_writer)
            .await
            .unwrap();
    });
    // pipe stdout from ws to current stdout
    tokio::spawn(async move {
        tokio::io::copy(&mut stdout_reader, &mut stdout)
            .await
            .unwrap();
    });

    info!("Attached to Lambda-Cntr-Pod");
    // When done, type `exit\n` to end it, so the pod is deleted.
    attached.await;

    Ok(())
}

async fn execute(pods: &Api<Pod>, id: String, cmd: String) -> anyhow::Result<()> {
    let ap = AttachParams::interactive_tty();
    let mut attached = pods.exec("lambda-cntr", vec!["/bin/bash"], &ap).await?;

    // The received streams from `AttachedProcess`
    let mut stdin_writer = attached.stdin().unwrap();
    let mut stdout_reader = attached.stdout().unwrap();

    let s = format!("cntr attach {} -- {} && exit\n", &id, &cmd);
    stdin_writer.write(s.as_bytes()).await?;

    let mut stdout = tokio::io::stdout();
    // pipe current stdin to the stdin writer from ws
    // pipe stdout from ws to current stdout
    tokio::spawn(async move {
        tokio::io::copy(&mut stdout_reader, &mut stdout)
            .await
            .unwrap();
    });
    attached.await;

    Ok(())
}

async fn delete(pods: &Api<Pod>) -> anyhow::Result<()> {
    // Delete it
    info!("Deleting Lambda-Cntr-Pod");
    pods.delete("lambda-cntr", &DeleteParams::default())
        .await?
        .map_left(|pdel| {
            assert_eq!(pdel.name(), "lambda-cntr");
        });

    Ok(())
}

// Returns container_id of pod and sets container engine env.

pub async fn get_container_id(pod: Pod) -> anyhow::Result<String> {
    let mut container_id = String::new();

    if let Some(p_status) = pod.clone().status {
        if let Some(c_status) = p_status.container_statuses {
            for c in c_status {
                if let Some(c_id) = c.container_id {
                    let id = c_id.clone();
                    let v: Vec<&str> = id.split("://").collect();
                    container_id = v[1].to_string();
                    env::set_var("CONTAINER_ENGINE", v[0].to_string());
                }
            }
        }
    }

    Ok(container_id)
}
