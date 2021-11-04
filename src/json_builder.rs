use serde_json::json;

pub fn get_json(
  image: String,
  socket: String,
  node: String,
  container_engine: String,
) -> anyhow::Result<serde_json::Value, ()> {
  let pod_name = format!("lambda-cntr-{}", node);

  let mount_path;
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
              "path": socket,
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
    Err(_) => Err(()),
  }
}
