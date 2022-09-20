from http import server
import threading
import time
from typing import Optional

from python_sdk import mosaic_sdk
print(mosaic_sdk.__all__)


class MosaicClient(threading.Thread):
    def __init__(self, server_address: str, client):
        print("\n")
        self._mosaic_client = mosaic_sdk.Client(server_address)

        # Client API
        #
        # https://github.com/python/cpython/blob/3.9/Lib/multiprocessing/process.py#L80
        # stores the Client class with its args and kwargs.
        self._client = client
        # Instantiate global model to None.
        self._global_model = None
        # threading interals
        self._exit_event = threading.Event()

        # Primitive lock objects. Once a thread has acquired a lock, 
        # subsequent attempts to acquire it block, until it is released; 
        # any thread may release it.
        self._step_lock = threading.Lock()
        super().__init__(daemon=True)

        print(f"\nMosaicClient init done.")


    def run(self):
        self._client = self._client()

        try:
            self._run()
        except Exception as err:
            self._exit_event.set()

    def _run(self):
        print(f"\t : Protocol is beeing performed : ")
        while not self._exit_event.is_set():
            self._step()

    def _step(self):
        with self._step_lock:
            print(f"\t(n) Performing single step: ")
            self._mosaic_client.step()

    def _train(self):
        print("Start training.")

        local_update = self._client.train_round(self._global_model)
        # local_update_ = self._client.serialize_training(local_update)

        try:
            self._mosaic_client.set_local_update(local_update)
        except:
            print("Failure in train.")

# Endpoint.
class PyClient():
    def __init__(self) -> None:
        # self.model = model
        super().__init__()

    def train_round(self, training_input: Optional[list]):
        print("\t\tPyClient: Training ...")
        time.sleep(3.0)
        print("\t\tPyClient: Training done.")
        # return self.model


def spawn_client(
    server_address: str,
    client: PyClient,
):
    """."""
    mosaic_client = MosaicClient(
        server_address, client
    )
    # spawns the internal mosaic client in a separate thread.
    # `start` calls the `run` method of `MosaicClient`
    # https://docs.python.org/3.8/library/threading.html#threading.Thread.start
    # https://docs.python.org/3.8/library/threading.html#threading.Thread.run
    mosaic_client.start()
    return mosaic_client
