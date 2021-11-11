# λ-Cntr

λ-Cntr is a replacement for `kubectl exec` that brings all your developers tools with you.
It is an extension of [Cntr](https://github.com/Mic92/cntr), enabeling its use in a Kubernetes cluster.

## Usage

λ-Cntr provides a subcommand `attach` that allows you to attach to a Pod.

```console
$ lambda-cntr attach --help
lambda-cntr-attach 0.1.0
Eduard von Briesen <e.v.briesen@gmail.com>
Attach Cntr-Pod to Container in Kubernetes

USAGE:
    lambda-cntr attach [OPTIONS] <pod_name> [container_name]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --image <image>                Set your container image [env: CNTR_IMAGE=]  [default: onestone070/lambda-
                                       cntr:latest]
    -n, --namespace <namespace>        Specify th namespace of the target Pod [default: default]
    -s, --socket-path <socket-path>    Path to the socket of the container engine on your node [env: SOCKET_PATH=]
                                       [default: /run/containerd/containerd.sock]

ARGS:
    <pod_name>          Pod Name
    <container_name>    Specify the container in the target Pod
```

To make full use λ-Cntr build your own [Dockerfile](https://github.com/EduardvonBriesen/Lambda-Cntr/blob/main/Dockerfile) to including your development tools.

## Caveats 

- Exiting the shell only exits the pod you attached to, but leaves you in the shell of the cntr-pod. Exit a second time to properly shutdown the application.