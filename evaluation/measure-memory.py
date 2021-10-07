from kubernetes import client, config
import helper as aux

CNTR_POD_NAME = 'cntr'
TEST_POD_NAME = 'busybox'
NAMESPACE = 'default'
FILE = 'memory.txt'


def benchmark_lambda_memory(api,  file: str):
    aux.deploy_test_pod(api)
    # Delete lambda-cntr pod in case on exists
    aux.delete_pod(api, CNTR_POD_NAME)
    # Attach lambda-cntr
    aux.lambda_exec('true')

    with open(file, 'a') as f:
        # Measure memory of test-pod
        mem = aux.pod_memory(TEST_POD_NAME)
        f.write('POD: %s\n' % mem)

        # Measure memory of lambda-pod
        mem = aux.pod_memory(CNTR_POD_NAME)
        f.write('LAMBDA: %s\n' % mem)

    aux.delete_pod(api, TEST_POD_NAME)
    aux.delete_pod(api, CNTR_POD_NAME)


def benchmark_ephemeral_memory(api,  file: str):
    aux.deploy_test_pod(api)

    with open(file, 'a') as f:
        # Measure memory of test-pod
        mem = aux.pod_memory(TEST_POD_NAME)
        f.write('POD_IDLE: %s\n' % mem)

        # Attach ephemeral container
        aux.ephemeral_attach(api)
        # Measure memory of test-/debug-pod
        mem = aux.pod_memory(TEST_POD_NAME)
        f.write('POD_ATTACHED: %s\n' % mem)

        print(aux.pod_memory(TEST_POD_NAME))

    aux.delete_pod(api, TEST_POD_NAME)


def main():
    config.load_kube_config()
    api = client.CoreV1Api()
    aux.create_test_namespace(api)

    benchmark_lambda_memory(api, FILE)
    benchmark_ephemeral_memory(api, FILE)


if __name__ == '__main__':
    main()
