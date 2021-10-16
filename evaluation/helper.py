from kubernetes import client
from kubernetes.client.rest import ApiException
from kubernetes.client.models.v1_namespace import V1Namespace
import time
import subprocess

NAMESPACE = 'default'
TEST_POD_NAME = 'busybox'
TEST_POD_IMAGE = 'busybox'
SOCKET = '/run/k3s/containerd/containerd.sock'
DEBUG_IMAGE = 'onestone070/lambda-cntr:latest'
EPHEMERAL_DEBUG_IMAGE = 'onestone070/lambda-cntr:ephem'

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
    cmd = ['./target/release/lambda-cntr', 'attach', TEST_POD_NAME,
           '-s', SOCKET, '-n', NAMESPACE, '-i', DEBUG_IMAGE]
    print(f'Attaching lambda-cntr with: ' + ' '.join(cmd))
    subprocess.run(cmd)

def lambda_exec(cmd: str):
    cmd = ['./target/debug/lambda-cntr', 'execute', TEST_POD_NAME, cmd,
           '-s', SOCKET, '-n', NAMESPACE, '-i', DEBUG_IMAGE]
    print(f'Attaching lambda-cntr with: ' + ' '.join(cmd))
    subprocess.run(cmd)

def ephemeral_attach():
    cmd = ['sudo', 'kubectl', 'debug', '-n', NAMESPACE,
           TEST_POD_NAME, f'--image={EPHEMERAL_DEBUG_IMAGE}']
    print(f'Attaching ephemeral container with: ' + ' '.join(cmd))
    subprocess.run(cmd)

def ephemeral_attach_background(api):
    cmd = ['sudo', 'kubectl', 'debug', '-it', '-n', NAMESPACE,
           TEST_POD_NAME, f'--image={EPHEMERAL_DEBUG_IMAGE}']
    print(f'Attaching ephemeral container with: ' + ' '.join(cmd))
    subprocess.Popen(cmd)

    while True:
        api_response = api.read_namespaced_pod(
            namespace=NAMESPACE, name=TEST_POD_NAME)
        if api_response.status.ephemeral_container_statuses != None:
            break
        time.sleep(0.0001)

def pod_memory(pod_name: str) -> list[tuple[str, int]]:
    result = []
    api = client.CustomObjectsApi()
    resource = api.list_namespaced_custom_object(
        group='metrics.k8s.io', version='v1beta1', namespace=NAMESPACE, plural='pods')
    for pod in resource['items']:
        if pod['metadata']['name'] == pod_name:
            for container in pod['containers']:
                memory = int(container['usage']['memory'].removesuffix('Ki'))
                result.append([container['name'], memory])
    return result