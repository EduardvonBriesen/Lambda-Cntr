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
    lambda-cntr attach [OPTIONS] <pod_name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --namespace <namespace>    Namespace of container [default: default]

ARGS:
    <pod_name>    Pod Name
```

To make full use λ-Cntr build your own [Dockerfile](https://github.com/EduardvonBriesen/Lambda-Cntr/blob/main/Dockerfile) to including your development tools and adjust [cntr.yaml](https://github.com/EduardvonBriesen/Lambda-Cntr/blob/main/cntr.yaml) to refer to your image.

## Caveats 

- Currently only with Containerd as the Kuberntes container runtime.
- The containerd-socket passed in [cntr.yaml](https://github.com/EduardvonBriesen/Lambda-Cntr/blob/main/cntr.yaml) is specific to k3s and has to be adjusted when used with other Kubernetes distributions.
- Exiting the shell only exits the pod you attached to, but leaves you in the shell of the cntr-pod. Exit a second time to properly shutdown the application.