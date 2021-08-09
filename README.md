# λ-Cntr

λ-Cntr is an extension of [Cntr](https://github.com/Mic92/cntr), enabeling its use in a serverless environment.

# Usage

Deploy Cntr to a Kubernetes cluster and get a shell running in the Pod.

```console
kubectl apply -f cntr.yaml
kubectl exec --stdin --tty cntr -- /bin/bash
```
From here Cntr can attach to any container running on the node.
