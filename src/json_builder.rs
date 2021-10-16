use serde_json::json;
use serde_json::Result;
use std::env;

pub fn get_json() -> Result<serde_json::Value> {
  let image = env::var("CNTR_IMAGE").expect("No image specified!");
  let path = env::var("SOCKET_PATH").expect("No path specified!");
  let container_engine = env::var("CONTAINER_ENGINE").expect("No engine specified!");

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
        "name": "lambda-cntr"
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
              "360000"
            ],
            "securityContext": {
              "privileged": true,
              "runAsUser": 0
            },
            "volumeMounts": [
              {
                "name": "container-sock",
                "mountPath": mount_path.to_string(),
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
        "restartPolicy": "Never"
      }
  }))?;

  Ok(cntr_pod)
}
