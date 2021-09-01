use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::{Result, Value};

pub fn get_json() -> Result<serde_json::Value> {
  let image = "onestone070/cntr";
  let mount_path = "/run/containerd/containerd.sock";
  let path = "/run/k3s/containerd/containerd.sock";

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
            "name": "cntr",
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
        "restartPolicy": "Never"
      }
  }))?;

  Ok(cntr_pod)
}
