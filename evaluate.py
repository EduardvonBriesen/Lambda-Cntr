from kubernetes import client, config
from kubernetes.client.rest import ApiException
from kubernetes.client.models.v1_namespace import V1Namespace
import subprocess
import os
import time
import timeit

CNTR_POD_NAME = 'cntr'
DEBUG_IMAGE = 'onestone070/lambda-cntr:latest'
EPHEMERAL_DEBUG_IMAGE = 'busybox'
TEST_POD_NAME = 'busybox'
TEST_POD_IMAGE = 'busybox'
NAMESPACE = 'default'
SOCKET = '/run/k3s/containerd/containerd.sock'


def create_test_namespace(api, namespace: str):
    try:
        namespaces = api.list_namespace(_request_timeout=3)
    except ApiException as e:
        if e.status != 404:
            print('Unknown error: %s' % e)
            exit(1)
    if not any(ns.metadata.name == namespace for ns in namespaces.items):
        print(f'Creating namespace: %s' % namespace)
        api.create_namespace(V1Namespace(metadata=dict(name=namespace)))
    else:
        print(f'Using existing namespace: %s' % namespace)


def deploy_test_pod(api, namespace: str, pod_name: str):
    api_response = None
    try:
        api_response = api.read_namespaced_pod(
            namespace=namespace, name=pod_name)
    except ApiException as e:
        if e.status != 404:
            print('Unknown error: %s' % e)
            exit(1)

    if api_response:
        print(
            f'Pod {pod_name} does already exist.')
    else:
        print(
            f'Pod {pod_name} does not exist. Creating it...')
        # Create pod manifest
        pod_manifest = {
            'apiVersion': 'v1',
            'kind': 'Pod',
            'metadata': {
                'name': pod_name,
                'namespace': namespace
            },
            'spec': {
                'containers': [{
                    'image': TEST_POD_IMAGE,
                    'name': f'container',
                    'command': ['sleep', '3600']
                }],
                'restartPolicy': 'Never'
            }
        }

        print(f'POD MANIFEST:\n{pod_manifest}')

        api_response = api.create_namespaced_pod(
            namespace=namespace, body=pod_manifest)

        while True:
            api_response = api.read_namespaced_pod(
                namespace=namespace, name=pod_name)
            if api_response.status.phase != 'Pending':
                break
            time.sleep(0.01)

        print(
            f'Pod {pod_name} in {namespace} created.')


def delete_pod(api, namespace: str, pod_name: str):
    api_response = None
    try:
        api_response = api.read_namespaced_pod(
            namespace=namespace, name=pod_name)
    except ApiException as e:
        if e.status != 404:
            print('Unknown error: %s' % e)
            exit(1)

    if api_response:
        print(
            f'Pod {pod_name} does exist. Deleting it...')
        api_response = api.delete_namespaced_pod(
            name=pod_name, namespace=namespace)

        while True:
            api_response = None
            try:
                api_response = api.read_namespaced_pod(
                    namespace=namespace, name=pod_name)
            except ApiException as e:
                if e.status != 404:
                    print('Unknown error: %s' % e)
                    exit(1)
            if api_response == None:
                break
            time.sleep(0.01)
    else:
        print(
            f'Pod {pod_name} does not exist.')


# def lambda_attach(namespace: str, pod_name: str):
#     cmd = ['./target/debug/lambda-cntr', 'attach', pod_name,
#            '-s', SOCKET, '-n', namespace, '-i', DEBUG_IMAGE]
#     print(f'Attaching lambda-cntr with: ' + ' '.join(cmd))
#     subprocess.run(cmd)


def lambda_exec(namespace: str, pod_name: str, cmd: str):
    cmd = ['./target/debug/lambda-cntr', 'execute', pod_name, cmd,
           '-s', SOCKET, '-n', namespace, '-i', DEBUG_IMAGE]
    print(f'Attaching lambda-cntr with: ' + ' '.join(cmd))
    subprocess.run(cmd)


def ephemeral_attach(api, namespace: str, pod_name: str):
    cmd = ['kubectl', 'debug', '-n', namespace, pod_name, f'--image=ubuntu']
    print(f'Attaching ephemeral container with: ' + ' '.join(cmd))
    subprocess.run(cmd)

    while True:
        api_response = api.read_namespaced_pod(
            namespace=namespace, name=pod_name)
        if api_response.status.ephemeral_container_statuses != None:
            break
        time.sleep(0.0001)


def benchmark_lambda_start_up_cold(api, repeat: int) -> list[float]:
    times = []
    deploy_test_pod(api, NAMESPACE, TEST_POD_NAME)

    # Delete lambda-cntr pod in case on exists
    delete_pod(api, NAMESPACE, CNTR_POD_NAME)

    # Deploy and attach lambda-cntr repeatedly
    for x in range(repeat):
        starttime = timeit.default_timer()
        lambda_exec(NAMESPACE, TEST_POD_NAME, 'true')
        times.append(timeit.default_timer() - starttime)
        delete_pod(api, NAMESPACE, CNTR_POD_NAME)

    delete_pod(api, NAMESPACE, TEST_POD_NAME)
    print(times)
    return times


def benchmark_lambda_start_up_warm(api, repeat: int) -> list[float]:
    times = []
    deploy_test_pod(api, NAMESPACE, TEST_POD_NAME)

    # Create lambda-cntr pod
    lambda_exec(NAMESPACE, TEST_POD_NAME, 'true')

    # Deploy and attach lambda-cntr repeatedly
    for x in range(repeat):
        starttime = timeit.default_timer()
        lambda_exec(NAMESPACE, TEST_POD_NAME, 'true')
        times.append(timeit.default_timer() - starttime)

    delete_pod(api, NAMESPACE, TEST_POD_NAME)
    print(times)
    return times


def benchmark_ephemeral_start_up(api, repeat: int) -> list[float]:
    times = []
    deploy_test_pod(api, NAMESPACE, TEST_POD_NAME)

    # Deploy and attach ephemeral containers repeatedly
    for x in range(repeat):
        starttime = timeit.default_timer()
        ephemeral_attach(api, NAMESPACE, TEST_POD_NAME)
        times.append(timeit.default_timer() - starttime)
        delete_pod(api, NAMESPACE, TEST_POD_NAME)

    return times


def pod_memory(pod_name: str) -> list[tuple[str, int]]:
    result = []
    api = client.CustomObjectsApi()
    resource = api.list_namespaced_custom_object(
        group='metrics.k8s.io', version='v1beta1', namespace=NAMESPACE, plural='pods')
    for pod in resource['items']:
        if pod['metadata']['name'] == pod_name:
            print(pod)
            for container in pod['containers']:
                memory = int(container['usage']['memory'].removesuffix('Ki'))
                result.append([container['name'], memory])
    return result


def benchmark_lambda_memory(api,  file: str):
    deploy_test_pod(api, NAMESPACE, TEST_POD_NAME)
    # Delete lambda-cntr pod in case on exists
    delete_pod(api, NAMESPACE, CNTR_POD_NAME)

    with open(file, 'a') as f:
        # Measure memory of test-pod
        mem = pod_memory(NAMESPACE, TEST_POD_NAME)
        f.write('POD_IDLE: %s\n' % mem)

        # Attach lambda-cntr
        lambda_exec(NAMESPACE, TEST_POD_NAME, 'sleep 30')
        time.sleep(5)

        # Measure memory of test-/debug-pod
        mem = pod_memory(NAMESPACE, TEST_POD_NAME)
        f.write('POD_ATTACHED: %s\n' % mem)
        mem = pod_memory(NAMESPACE, CNTR_POD_NAME)
        f.write('LAMBDA_ATTACHED: %s\n' % mem)

        print(pod_memory(NAMESPACE, TEST_POD_IMAGE))


def benchmark_ephemeral_memory(api,  file: str):
    deploy_test_pod(api, NAMESPACE, TEST_POD_NAME)

    with open(file, 'a') as f:
        # Measure memory of test-pod
        mem = pod_memory(NAMESPACE, TEST_POD_NAME)
        f.write('POD_IDLE: %s\n' % mem)

        # Attach ephemeral container
        ephemeral_attach(api, NAMESPACE, TEST_POD_NAME)
        # Measure memory of test-/debug-pod
        mem = pod_memory(NAMESPACE, TEST_POD_NAME)
        f.write('POD_ATTACHED: %s\n' % mem)

        print(pod_memory(NAMESPACE, TEST_POD_IMAGE))


def main():
    config.load_kube_config()
    api = client.CoreV1Api()
    create_test_namespace(api, NAMESPACE)

    cold =  benchmark_lambda_start_up_cold(api, 20)
    warm = benchmark_lambda_start_up_warm(api, 20)
    ephem = benchmark_ephemeral_start_up(api, 20)

    with open('results.txt', 'a') as f:
        f.write('COLD_STARTUP: %s\n' % cold)
        f.write('COLD_STARTUP_AVG: %s\n' % (sum(cold)/len(cold)))
        f.write('WARM_STARTUP: %s\n' % warm)
        f.write('WARM_STARTUP_AVG: %s\n' % (sum(warm)/len(warm)))
        f.write('EPHEM_STARTUP: %s\n' % ephem)
        f.write('EPHEM_STARTUP_AVG: %s\n' % (sum(ephem)/len(ephem)))
    delete_pod(api, NAMESPACE, TEST_POD_NAME)


if __name__ == '__main__':
    main()
