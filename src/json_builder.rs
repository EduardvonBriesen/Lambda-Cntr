use serde_json::json;
use std::env;
use log::{error, info};

pub fn get_json(image: String, socket: String, node: String, container_engine: String) -> anyhow::Result<serde_json::Value, ()> {

  // If no image was defined, env is used, then default
  let mut container_image = String::new();
  if image.len() < 1 {
    let image_env = env::var("CNTR_IMAGE");
    if image_env.is_ok() {
      container_image = image_env.unwrap();
      info!("Using image: {}", image);
    } else {
      container_image =  String::from("onestone070/lambda-cntr:latest");
      info!("No Image specified, using default: {}", container_image);
    }
  } else {
    container_image = image;
  }

  // If no path was defined, env is used or error thrown
  let mut container_socket = String::new();
  if socket.len() < 1 {
    let socket_env = env::var("SOCKET_PATH");
    if socket_env.is_ok() {
      container_socket = socket_env.unwrap();
      info!("Using socket: {}", container_socket);
    } else {
      error!("Please pass the socket path or set the env variable 'SOCKET_PATH'");
      return Err(());
    }
  } else {
    container_socket = socket;
  }

  let pod_name = format!("lambda-cntr-{}", node);

  let mut mount_path;
  match container_engine.as_str() {
    "docker" => {
      mount_path = String::from("/run/docker.sock");
    }
    "containerd" => {
      mount_path = String::from("/run/containerd/containerd.sock");
    }
    _ => {
      mount_path = String::from("/run/containerd/containerd.sock");
    }
  }

  let cntr_pod = serde_json::from_value(json!({
      "apiVersion": "v1",
      "kind": "Pod",
      "metadata": {
        "name": pod_name,
      },
      "spec": {
        "hostPID": true,
        "containers": [
          {
            "name": "lambda-cntr",
            "image": container_image,
            "imagePullPolicy": "Always",
            "command": [
              "sleep",
              "360000"
            ],
            "securityContext": {
              "privileged": true,
              "runAsUser": 0
            },
            "volumeMounts": [
              {
                "name": "container-sock",
                "mountPath": mount_path,
              }
            ],
            "env": [
              {
                "name": "CONTAINERD_NAMESPACE",
                "value": "k8s.io"
              }
            ]
          }
        ],
        "volumes": [
          {
            "name": "container-sock",
            "hostPath": {
              "path": container_socket,
              "type": "Socket"
            }
          }
        ],
        "restartPolicy": "Never",
        "nodeSelector": {
            "kubernetes.io/hostname": node,
          }
      }
  }));

  match cntr_pod {
    Ok(cntr_pod) => return Ok(cntr_pod),
    Err(_) => Err(())
  }

}
