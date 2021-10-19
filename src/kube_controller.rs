use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use log::{error, info};

use crate::json_builder;
use kube::{
    api::{Api, AttachParams, DeleteParams, ListParams, PostParams, ResourceExt, WatchEvent},
    Client,
};
use std::env;
use tokio::io::AsyncWriteExt;

#[warn(dead_code)]
fn main() {}

#[tokio::main]
#[warn(unused_must_use)]
pub async fn deploy_and_attach() -> anyhow::Result<()> {
    let namespace = env::var("NAMESPACE").expect("No namespace specified!");
    let container_name = env::var("POD_NAME").expect("No pod specified!");

    env_logger::init();
    
    match Client::try_default().await {
        Ok(client) => {
            let pods: Api<Pod> = Api::namespaced(client.clone(), &namespace);
            match pods.get(&container_name).await {
                Ok(pod) => match get_container_id(pod.clone()).await {
                    Ok(id) => {
                        let node = get_node(pod.clone()).await?;
                        deploy(&pods, node.clone()).await?;
                        attach(&pods, node.clone(), id.to_string()).await?;
                        delete(&pods, node.clone()).await?;
                    }
                    Err(()) => {}
                },
                Err(_) => {
                    error!(
                        "No pod \"{}\" found in namespace \"{}\"!",
                        container_name,
                        namespace.clone()
                    );
                }
            }
        }
        Err(_) => error!("Could not connect to client!"),
    }

    Ok(())
}

#[tokio::main]
pub async fn deploy_and_execute() -> anyhow::Result<()> {
    let namespace = env::var("NAMESPACE").expect("No namespace specified!");
    let container_name = env::var("POD_NAME").expect("No pod specified!");
    let cmd = env::var("CMD").expect("No command specified!");

    env_logger::init();

    match Client::try_default().await {
        Ok(client) => {
            let pods: Api<Pod> = Api::namespaced(client, &namespace);
            match pods.get(&container_name).await {
                Ok(pod) => match get_container_id(pod.clone()).await {
                    Ok(id) => {
                        let node = get_node(pod.clone()).await?;
                        deploy(&pods, node.clone()).await?;
                        execute(&pods, node.clone(), id.to_string(), cmd).await?;
                    }
                    Err(()) => {}
                },
                Err(_) => {
                    error!(
                        "No pod \"{}\" found in namespace \"{}\"!",
                        container_name,
                        namespace.clone()
                    );
                }
            }
        }
        Err(_) => error!("Could not connect to client!"),
    }

    Ok(())
}

async fn deploy(pods: &Api<Pod>, node: String) -> anyhow::Result<()> {
    let cntr_pod = json_builder::get_json().expect("Unable to parse json");
    let cntr_pod = serde_json::from_value(cntr_pod).expect("Unable to parse json");

    let pod_name = format!("lambda-cntr-{}", node);

    match pods.get(&pod_name).await {
        Ok(_p) => info!("Lambda-Cntr-Pod already exist on {}, attaching ...", node),
        Err(_p) => {
            info!("Lambda-Cntr-Pod doesn't exist on {}, creating ...", node);
            pods.create(&PostParams::default(), &cntr_pod).await?;

            // Wait until the pod is running, otherwise we get 500 error.
            let lp = ListParams::default()
                .fields(&format!("metadata.name={}", pod_name))
                .timeout(60);
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

async fn attach(pods: &Api<Pod>, node: String, id: String) -> anyhow::Result<()> {
    let ap = AttachParams::interactive_tty();
    let pod_name = format!("lambda-cntr-{}", node);
    let mut attached = pods.exec(&pod_name, vec!["/bin/bash"], &ap).await?;

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

async fn execute(pods: &Api<Pod>, node: String, id: String, cmd: String) -> anyhow::Result<()> {
    let ap = AttachParams::interactive_tty();
    let pod_name = format!("lambda-cntr-{}", node);
    let mut attached = pods.exec(&pod_name, vec!["/bin/bash"], &ap).await?;

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

async fn delete(pods: &Api<Pod>, node: String) -> anyhow::Result<()> {
    // Delete it
    info!("Deleting Lambda-Cntr-Pod from {}", node);
    let pod_name = format!("lambda-cntr-{}", node);
    pods.delete(&pod_name, &DeleteParams::default())
        .await?
        .map_left(|pdel| {
            assert_eq!(pdel.name(), pod_name);
        });

    Ok(())
}

// Returns container_id of pod and sets container engine env.
pub async fn get_container_id(pod: Pod) -> anyhow::Result<String, ()> {
    let container_name = env::var("CONTAINER_NAME").expect("No container name specified!");
    let mut container_id = String::new();

    if let Some(p_status) = pod.clone().status {
        if let Some(c_status) = p_status.container_statuses {
            if c_status.len() == 1 {
                for c in c_status {
                    if let Some(c_id) = c.container_id {
                        let id = c_id.clone();
                        let v: Vec<&str> = id.split("://").collect();
                        container_id = v[1].to_string();
                        env::set_var("CONTAINER_ENGINE", v[0].to_string());
                    }
                }
            } else {
                if container_name.eq("") {
                    error!(
                        "The Pod {} contains more than one container, please specify a container.",
                        pod.name()
                    );
                    return Err(());
                }
                for c in c_status {
                    if c.name == container_name {
                        if let Some(c_id) = c.container_id {
                            let id = c_id.clone();
                            let v: Vec<&str> = id.split("://").collect();
                            container_id = v[1].to_string();
                            env::set_var("CONTAINER_ENGINE", v[0].to_string());
                            break;
                        }
                    }
                }
                if container_id.eq("") {
                    error!(
                        "The Pod {} did not contain a container named {}.",
                        pod.name(),
                        container_name
                    );
                    return Err(());
                }
            }
        }
    }
    Ok(container_id)
}

pub async fn get_node(pod: Pod) -> anyhow::Result<String> {
    let mut node = String::new();

    if let Some(p_spec) = pod.clone().spec {
        if let Some(n_name) = p_spec.node_name {
            node = n_name;
            env::set_var("NODE", node.to_string());
        }
    }
    Ok(node)
}
