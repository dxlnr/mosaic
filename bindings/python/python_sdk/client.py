from http import server
import threading
import time
from typing import Optional

from python_sdk import mosaic_sdk
print(mosaic_sdk.__all__)


class MosaicClient(threading.Thread):
    def __init__(self, server_address: str, client):
        self._mosaic_client = mosaic_sdk.Client(server_address)

        # https://github.com/python/cpython/blob/3.9/Lib/multiprocessing/process.py#L80
        # stores the Client class with its args and kwargs
        self._client = client
        self._global_model = None

        # threading interals
        self._exit_event = threading.Event()
        self._tick_lock = threading.Lock()
        super().__init__(daemon=True)

        print(f"MosaicClient init done.")


    def run(self):
        self._client = self._client()

        try:
            self._run()
        except Exception as err:
            self._exit_event.set()

    def _run(self):
        while not self._exit_event.is_set():
            print("jooo")
            self._go()

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
    def __init__(self, model: list) -> None:
        self.model = model
        super().__init__()

    def train_round(self, training_input: Optional[list]) -> list:
        print("training")
        time.sleep(3.0)
        print("training done")
        return self.model


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
