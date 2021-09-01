# λ-Cntr

λ-Cntr is a replacement for `kubectl exec` that brings all your developers tools with you.
It is an extension of [Cntr](https://github.com/Mic92/cntr), enabeling its use in a Kubernetes cluster.

## Usage

λ-Cntr provides a subcommand `attach` that allows you to attach to a Pod.

```console
$ lambda-cntr attach --help
lambda-cntr-attach 0.1.0
Eduard von Briesen <e.v.briesen@gmail.com>
Attach Cntr-Pod to Container in Kubeneretes

USAGE:
    lambda-cntr attach [FLAGS] [OPTIONS] <pod_name> --socket-path <socket-path>

FLAGS:
    -d, --docker     Set if Docker is used as container engine [default: containerd]
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --image <image>                Set your container image [default: onestone070/cntr]
    -l, --log-level <log-level>        Set the logging level (e.g. "info,kube=debug") [default: info]
    -n, --namespace <namespace>        Namespace of container [default: default]
    -s, --socket-path <socket-path>    Path to the socket of the container engine on your node (e.g.
                                       "/run/k3s/containerd/containerd.sock")

ARGS:
    <pod_name>    Pod Name
```

To make full use λ-Cntr build your own [Dockerfile](https://github.com/EduardvonBriesen/Lambda-Cntr/blob/main/Dockerfile) to including your development tools and adjust [cntr.yaml](https://github.com/EduardvonBriesen/Lambda-Cntr/blob/main/cntr.yaml) to refer to your image.

## Caveats 

- Exiting the shell only exits the pod you attached to, but leaves you in the shell of the cntr-pod. Exit a second time to properly shutdown the application.