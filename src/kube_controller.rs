use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use log::{error, info};
use std::collections::HashMap;

use crate::json_builder;
use kube::{
    api::{Api, AttachParams, DeleteParams, ListParams, PostParams, ResourceExt, WatchEvent},
    Client,
};
use tokio::io::AsyncWriteExt;
use std::env;

fn main() {}

#[tokio::main]
pub async fn deploy_and_attach() -> anyhow::Result<()> {
    let namespace = env::var("NAMESPACE").expect("No namespace specified!");
    let container_id = env::var("CONTAINER_ID").expect("No container specified!");

    env_logger::init();
    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let map = get_container_id(pods.clone()).await?;

    if !map.contains_key(&container_id) {
        error!(
            "No pod \"{}\" found in namespace \"{}\"!",
            container_id,
            namespace.clone()
        );
    } else {
        if let Some(id) = map.get(&container_id) {
            deploy(&pods).await?;
            attach(&pods, id.to_string()).await?;
            delete(&pods).await?;
        }
    }

    Ok(())
}

async fn deploy(pods: &Api<Pod>) -> anyhow::Result<()> {
    let cntr_pod = json_builder::get_json().expect("Unable to parse json");
    let cntr_pod = serde_json::from_value(cntr_pod).expect("Unable to parse json");

    let p = pods.get("cntr").await;
    match p {
        Ok(_p) => info!("Cntr-Pod already exist, attaching ..."),
        Err(_p) => {
            // Stop on error including a pod already exists or is still being deleted.
            info!("Cntr-Pod doesn't exist, creating ...");
            pods.create(&PostParams::default(), &cntr_pod).await?;
            // Wait until the pod is running, otherwise we get 500 error.
            let lp = ListParams::default()
                .fields("metadata.name=cntr")
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
    let mut attached = pods.exec("cntr", vec!["/bin/bash"], &ap).await?;

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

    info!("Attached to Cntr-Pod");

    // When done, type `exit\n` to end it, so the pod is deleted.
    attached.await;

    Ok(())
}

async fn delete(pods: &Api<Pod>) -> anyhow::Result<()> {
    // Delete it
    info!("Deleting Cntr-Pod");
    pods.delete("cntr", &DeleteParams::default())
        .await?
        .map_left(|pdel| {
            assert_eq!(pdel.name(), "cntr");
        });

    Ok(())
}

pub async fn get_container_id(pods: Api<Pod>) -> anyhow::Result<HashMap<String, String>> {
    let mut map = HashMap::new();

    for p in pods.list(&Default::default()).await? {
        if let Some(p_status) = p.clone().status {
            if let Some(c_status) = p_status.container_statuses {
                for c in c_status {
                    if let Some(container_id) = c.container_id {
                        let id = container_id.clone();
                        if let Some(id) = id.strip_prefix("containerd://") {
                            map.insert(p.name(), id.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(map)
}
