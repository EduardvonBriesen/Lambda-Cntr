from kubernetes import client, config
import timeit

import helper as aux

CNTR_POD_NAME = 'cntr'
TEST_POD_NAME = 'busybox'
NAMESPACE = 'default'
SAMPLE_SIZE = 20
FILE = 'start-up.txt'


def benchmark_lambda_start_up_cold(api, repeat: int) -> list[float]:
    times = []
    aux.deploy_test_pod(api)

    # Delete lambda-cntr pod in case on exists
    aux.delete_pod(api, CNTR_POD_NAME)

    # Deploy and attach lambda-cntr repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        starttime = timeit.default_timer()
        aux.lambda_exec('true')
        times.append(timeit.default_timer() - starttime)
        aux.delete_pod(api, CNTR_POD_NAME)

    aux.delete_pod(api, TEST_POD_NAME)
    return times


def benchmark_lambda_start_up_warm(api, repeat: int) -> list[float]:
    times = []
    aux.deploy_test_pod(api)

    # Create lambda-cntr pod
    aux.lambda_exec('true')

    # Deploy and attach lambda-cntr repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        starttime = timeit.default_timer()
        aux.lambda_exec('true')
        times.append(timeit.default_timer() - starttime)

    aux.delete_pod(api, TEST_POD_NAME)
    return times


def benchmark_ephemeral_start_up_cold(api, repeat: int) -> list[float]:
    times = []

    # Deploy and attach ephemeral containers repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        aux.deploy_test_pod(api)
        starttime = timeit.default_timer()
        aux.ephemeral_attach()
        times.append(timeit.default_timer() - starttime)
        aux.delete_pod(api, TEST_POD_NAME)

    return times


def benchmark_ephemeral_start_up_warm(api, repeat: int) -> list[float]:
    times = []
    aux.deploy_test_pod(api)
    aux.ephemeral_attach()

    # Deploy and attach ephemeral containers repeatedly
    for x in range(repeat):
        print('[', x+1, '|', repeat, ']')
        aux.deploy_test_pod(api)
        starttime = timeit.default_timer()
        aux.ephemeral_attach()
        times.append(timeit.default_timer() - starttime)

    aux.delete_pod(api, TEST_POD_NAME)
    return times


def main():
    config.load_kube_config()
    api = client.CoreV1Api()
    aux.create_test_namespace(api)

    lambda_cold = benchmark_lambda_start_up_cold(api, SAMPLE_SIZE)
    lambda_warm = benchmark_lambda_start_up_warm(api, SAMPLE_SIZE)
    ephem_cold = benchmark_ephemeral_start_up_cold(api, SAMPLE_SIZE)
    ephem_warm = benchmark_ephemeral_start_up_warm(api, SAMPLE_SIZE)

    with open(FILE, 'a') as f:
        f.write('LAMBDA_COLD_STARTUP: %s\n' % lambda_cold)
        f.write('LAMBDA_COLD_STARTUP_AVG: %s\n' %
                (sum(lambda_cold)/len(lambda_cold)))
        f.write('LAMBDA_WARM_STARTUP: %s\n' % lambda_warm)
        f.write('LAMBDA_WARM_STARTUP_AVG: %s\n' %
                (sum(lambda_warm)/len(lambda_warm)))
        f.write('EPHEM_COLD_STARTUP: %s\n' % ephem_cold)
        f.write('EPHEM_COLD_STARTUP_AVG: %s\n' %
                (sum(ephem_cold)/len(ephem_cold)))
        f.write('EPHEM_WARM_STARTUP: %s\n' % ephem_warm)
        f.write('EPHEM_WARM_STARTUP_AVG: %s\n' %
                (sum(ephem_warm)/len(ephem_warm)))


if __name__ == '__main__':
    main()
