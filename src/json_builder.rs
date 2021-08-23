use serde_json::json;
use serde_json::{Result, Value};

pub fn get_json() -> Result<serde_json::Value> {
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
              "image": "onestone070/cntr",
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
                  "name": "containerd-sock",
                  "mountPath": "/run/containerd/containerd.sock"
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
              "name": "containerd-sock",
              "hostPath": {
                "path": "/run/k3s/containerd/containerd.sock",
                "type": "Socket"
              }
            }
          ],
          "restartPolicy": "Never"
        }
    }))?;
    
    Ok(cntr_pod)
}
