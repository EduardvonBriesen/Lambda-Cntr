from kubernetes import client, config
from kubernetes.client.rest import ApiException
from kubernetes.client.models.v1_namespace import V1Namespace
import subprocess
import os
import time
import timeit

CNTR_POD_NAME = 'cntr'
DEBUG_IMAGE = 'onestone070/lambda-cntr:latest'
EPHEMERAL_DEBUG_IMAGE = 'onestone070/lambda-cntr:ephem'
TEST_POD_NAME = 'busybox'
TEST_POD_IMAGE = 'busybox'
NAMESPACE = 'default'
SOCKET = '/run/k3s/containerd/containerd.sock'


def create_test_namespace(api):
    try:
        namespaces = api.list_namespace(_request_timeout=3)
    except ApiException as e:
        if e.status != 404:
            print('Unknown error: %s' % e)
            exit(1)
    if not any(ns.metadata.name == NAMESPACE for ns in namespaces.items):
        print(f'Creating namespace: %s' % NAMESPACE)
        api.create_namespace(V1Namespace(metadata=dict(name=NAMESPACE)))
    else:
        print(f'Using existing namespace: %s' % NAMESPACE)


def deploy_test_pod(api):
    api_response = None
    try:
        api_response = api.read_namespaced_pod(
            namespace=NAMESPACE, name=TEST_POD_NAME)
    except ApiException as e:
        if e.status != 404:
            print('Unknown error: %s' % e)
            exit(1)

    if api_response:
        print(
            f'Pod {TEST_POD_NAME} does already exist.')
    else:
        print(
            f'Pod {TEST_POD_NAME} does not exist. Creating it...')
        # Create pod manifest
        pod_manifest = {
            'apiVersion': 'v1',
            'kind': 'Pod',
            'metadata': {
                'name': TEST_POD_NAME,
                'namespace': NAMESPACE
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
            namespace=NAMESPACE, body=pod_manifest)

        while True:
            api_response = api.read_namespaced_pod(
                namespace=NAMESPACE, name=TEST_POD_NAME)
            if api_response.status.phase != 'Pending':
                break
            time.sleep(0.01)

        print(
            f'Pod {TEST_POD_NAME} in {NAMESPACE} created.')


def delete_pod(api, pod_name: str):
    api_response = None
    try:
        api_response = api.read_namespaced_pod(
            namespace=NAMESPACE, name=pod_name)
    except ApiException as e:
        if e.status != 404:
            print('Unknown error: %s' % e)
            exit(1)

    if api_response:
        print(
            f'Pod {pod_name} does exist. Deleting it...')
        api_response = api.delete_namespaced_pod(
            name=pod_name, namespace=NAMESPACE)

        while True:
            api_response = None
            try:
                api_response = api.read_namespaced_pod(
                    namespace=NAMESPACE, name=pod_name)
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


def lambda_attach():
    cmd = ['./target/debug/lambda-cntr', 'attach', TEST_POD_NAME,
           '-s', SOCKET, '-n', NAMESPACE, '-i', DEBUG_IMAGE]
    print(f'Attaching lambda-cntr with: ' + ' '.join(cmd))
    subprocess.run(cmd)


def lambda_exec(cmd: str):
    cmd = ['./target/debug/lambda-cntr', 'execute', TEST_POD_NAME, cmd,
           '-s', SOCKET, '-n', NAMESPACE, '-i', DEBUG_IMAGE]
    print(f'Attaching lambda-cntr with: ' + ' '.join(cmd))
    subprocess.run(cmd)


def ephemeral_attach(api):
    cmd = ['kubectl', 'debug', '-n', NAMESPACE,
           TEST_POD_NAME, f'--image={EPHEMERAL_DEBUG_IMAGE}']
    print(f'Attaching ephemeral container with: ' + ' '.join(cmd))
    subprocess.run(cmd)

    while True:
        api_response = api.read_namespaced_pod(
            namespace=NAMESPACE, name=TEST_POD_NAME)
        if api_response.status.ephemeral_container_statuses != None:
            break
        time.sleep(0.0001)


def benchmark_lambda_start_up_cold(api, repeat: int) -> list[float]:
    times = []
    deploy_test_pod(api)

    # Delete lambda-cntr pod in case on exists
    delete_pod(api, CNTR_POD_NAME)

    # Deploy and attach lambda-cntr repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        starttime = timeit.default_timer()
        lambda_exec('true')
        times.append(timeit.default_timer() - starttime)
        delete_pod(api, CNTR_POD_NAME)

    delete_pod(api, TEST_POD_NAME)
    return times


def benchmark_lambda_start_up_warm(api, repeat: int) -> list[float]:
    times = []
    deploy_test_pod(api)

    # Create lambda-cntr pod
    lambda_exec('true')

    # Deploy and attach lambda-cntr repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        starttime = timeit.default_timer()
        lambda_exec('true')
        times.append(timeit.default_timer() - starttime)

    delete_pod(api, TEST_POD_NAME)
    return times


def benchmark_ephemeral_start_up_cold(api, repeat: int) -> list[float]:
    times = []

    # Deploy and attach ephemeral containers repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        deploy_test_pod(api)
        starttime = timeit.default_timer()
        ephemeral_attach(api)
        times.append(timeit.default_timer() - starttime)
        delete_pod(api, TEST_POD_NAME)

    return times

def benchmark_ephemeral_start_up_warm(api, repeat: int) -> list[float]:
    times = []
    ephemeral_attach(api)

    # Deploy and attach ephemeral containers repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        deploy_test_pod(api)
        starttime = timeit.default_timer()
        ephemeral_attach(api)
        times.append(timeit.default_timer() - starttime)

    delete_pod(api, TEST_POD_NAME)
    return times

def pod_memory(pod_name: str) -> list[tuple[str, int]]:
    result = []
    api = client.CustomObjectsApi()
    resource = api.list_namespaced_custom_object(
        group='metrics.k8s.io', version='v1beta1', namespace=NAMESPACE, plural='pods')
    print(resource)
    for pod in resource['items']:
        if pod['metadata']['name'] == pod_name:
            for container in pod['containers']:
                memory = int(container['usage']['memory'].removesuffix('Ki'))
                result.append([container['name'], memory])
    return result


def benchmark_lambda_memory(api,  file: str):
    deploy_test_pod(api)
    # Delete lambda-cntr pod in case on exists
    delete_pod(api, CNTR_POD_NAME)

    with open(file, 'a') as f:
        # Measure memory of test-pod
        mem = pod_memory(TEST_POD_NAME)
        f.write('POD_IDLE: %s\n' % mem)

        # Attach lambda-cntr
        lambda_exec('true')

        # Measure memory of lambda-pod
        mem = pod_memory(CNTR_POD_NAME)
        f.write('LAMBDA_ATTACHED: %s\n' % mem)

    delete_pod(api, TEST_POD_IMAGE)
    delete_pod(api, CNTR_POD_NAME)


def benchmark_ephemeral_memory(api,  file: str):
    deploy_test_pod(api)

    with open(file, 'a') as f:
        # Measure memory of test-pod
        mem = pod_memory(TEST_POD_NAME)
        f.write('POD_IDLE: %s\n' % mem)

        # Attach ephemeral container
        ephemeral_attach(api)
        # Measure memory of test-/debug-pod
        mem = pod_memory(TEST_POD_NAME)
        f.write('POD_ATTACHED: %s\n' % mem)

        print(pod_memory(TEST_POD_IMAGE))
    # delete_pod(api, TEST_POD_IMAGE)


def main():
    config.load_kube_config()
    api = client.CoreV1Api()
    create_test_namespace(api)

    lambda_cold = benchmark_lambda_start_up_cold(api, 20)
    lambda_warm = benchmark_lambda_start_up_warm(api, 20)
    ephem_cold = benchmark_ephemeral_start_up_cold(api, 20)
    ephem_warm = benchmark_ephemeral_start_up_warm(api, 20)

    with open('results.txt', 'a') as f:
        f.write('LAMBDA_COLD_STARTUP: %s\n' % lambda_cold)
        f.write('LAMBDA_COLD_STARTUP_AVG: %s\n' % (sum(lambda_cold)/len(lambda_cold)))
        f.write('LAMBDA_WARM_STARTUP: %s\n' % lambda_warm)
        f.write('LAMBDA_WARM_STARTUP_AVG: %s\n' % (sum(lambda_warm)/len(lambda_warm)))
        f.write('EPHEM_COLD_STARTUP: %s\n' % ephem_cold)
        f.write('EPHEM_COLD_STARTUP_AVG: %s\n' % (sum(ephem_cold)/len(ephem_cold)))
        f.write('EPHEM_WARM_STARTUP: %s\n' % ephem_warm)
        f.write('EPHEM_WARM_STARTUP_AVG: %s\n' % (sum(ephem_warm)/len(ephem_warm)))

if __name__ == '__main__':
    main()
