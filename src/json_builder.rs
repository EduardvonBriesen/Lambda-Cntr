use serde_json::json;
use serde_json::{Result};
use std::env;

pub fn get_json() -> Result<serde_json::Value> {
  let image = env::var("CNTR_IMAGE").expect("No image specified!");
  let mount_path = env::var("MOUNT_PATH").expect("No path specified!");
  let path =  env::var("SOCKET_PATH").expect("No path specified!");

  let cntr_pod = serde_json::from_value(json!({
      "apiVersion": "v1",
      "kind": "Pod",
      "metadata": {
        "name": "cntr"
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
        "restartPolicy": "Never"
      }
  }))?;

  Ok(cntr_pod)
}
