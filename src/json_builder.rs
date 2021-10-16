use serde_json::json;
use serde_json::Result;
use std::env;

pub fn get_json() -> Result<serde_json::Value> {
  let image = env::var("CNTR_IMAGE").expect("No image specified!");
  let path = env::var("SOCKET_PATH").expect("No path specified!");
  let node = env::var("NODE").expect("No node specified!");
  let container_engine = env::var("CONTAINER_ENGINE").expect("No engine specified!");

  let pod_name = format!("lambda-cntr-{}", node);

  let mut mount_path = String::new();
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
            "image": image,
            "imagePullPolicy": "Always",
            "command": [
              "sleep",
              "3600"
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
              "path": path,
              "type": "Socket"
            }
          }
        ],
        "restartPolicy": "Never",
        "nodeSelector": {
            "kubernetes.io/hostname": node,
          }
      }
  }))?;

  Ok(cntr_pod)
}
