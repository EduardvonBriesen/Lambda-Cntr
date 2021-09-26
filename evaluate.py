
from kubernetes import client, config
from kubernetes.client.rest import ApiException
from kubernetes.client.models.v1_namespace import V1Namespace
import subprocess
import os
import time
import timeit
import csv

CNTR_POD_NAME = 'cntr'
DEBUG_IMAGE = 'onestone070/lambda-cntr:latest'
TEST_POD_NAME = 'busybox'
TEST_POD_IMAGE = 'busybox'
NAMESPACE = 'default'


def create_test_namespace(api, namespace: str):
    try:
        namespaces = api.list_namespace(_request_timeout=3)
    except ApiException as e:
        if e.status != 404:
            print('Unknown error: %s' % e)
            exit(1)
    if not any(ns.metadata.name == namespace for ns in namespaces.items):
        print(
            f'From {os.path.basename(__file__)}:Creating namespace: %s' % namespace)
        api.create_namespace(V1Namespace(metadata=dict(name=namespace)))
    else:
        print(
            f'From {os.path.basename(__file__)}:Using existing namespace: %s' % namespace)


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


def lambda_attach(namespace: str, pod_name: str):
    cmd = ['./target/debug/lambda-cntr', 'attach', pod_name, '-s',
           '/run/k3s/containerd/containerd.sock', '-n', namespace, '-i', DEBUG_IMAGE]
    print(f'Attaching lambda-cntr with: ' + ' '.join(cmd))
    subprocess.run(cmd)


def ephemeral_attach(namespace: str, pod_name: str):
    cmd = ['kubectl', 'debug', '-n', namespace,
           '-it', pod_name, f'--image={DEBUG_IMAGE}']
    print(f'Attaching ephemeral container with: ' + ' '.join(cmd))
    subprocess.run(cmd)


def benchmark_lambda_start_up(api, repeat: int) -> list[float]:
    times = []
    deploy_test_pod(api, NAMESPACE, TEST_POD_NAME)

    # Delete lambda-cntr pod in case on exists
    delete_pod(api, NAMESPACE, CNTR_POD_NAME)

    # Deploy and attach lambda-cntr repeatedly
    for x in range(repeat):
        starttime = timeit.default_timer()
        lambda_attach(NAMESPACE, TEST_POD_NAME)
        times.append(timeit.default_timer() - starttime)
        delete_pod(api, NAMESPACE, CNTR_POD_NAME)

    delete_pod(api, NAMESPACE, TEST_POD_NAME)
    print(times)
    return times


def benchmark_ephemeral_start_up(api, repeat: int) -> list[float]:
    times = []

    # Deploy and attach ephemeral containers repeatedly
    for x in range(repeat):
        deploy_test_pod(api, NAMESPACE, TEST_POD_NAME)
        starttime = timeit.default_timer()
        ephemeral_attach(NAMESPACE, TEST_POD_NAME)
        times.append(timeit.default_timer() - starttime)
        delete_pod(api, NAMESPACE, TEST_POD_NAME)

    print(times)
    return times


def main():
    config.load_kube_config()
    api = client.CoreV1Api()

    create_test_namespace(api, NAMESPACE)

    with open('results.txt', 'w') as f:
        f.write("%s\n" % benchmark_lambda_start_up(api, 5))
        f.write("%s\n" % benchmark_ephemeral_start_up(api, 5))


if __name__ == '__main__':
    main()
