# λ-Cntr

λ-Cntr is an extension of [Cntr](https://github.com/Mic92/cntr), enabeling its use in a serverless environment.

## Usage

λ-Cntr provides a subcommand `attach` that deploys a Cntr-Pod and attaches it to a Pod.

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
