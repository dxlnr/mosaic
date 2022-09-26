from abc import ABC, abstractmethod
from http import server
import threading
import time
from typing import Optional

from .model import CNN
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

        # some configs for later
        self.conf_standalone = True

        print(f"\nMosaicClient init done.")

    @abstractmethod
    def serialize_local_model(self) -> list:
        r"""
        Serializes the local model into a `list` data type. The data type of the
        elements must match the data type attached as metadata.
        :returns: The local model (self.model) as a `list`.
        """
        raise NotImplementedError()

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

        local_update = self._client.train_single_update(self._global_model)
        # local_update_ = self._client.serialize_training(local_update)

        try:
            self._mosaic_client.set_local_model(local_update)
        except:
            print("Failure in train.")

    def _run_standalone(self):
        print(f"\t : Protocol is beeing performed : ")
        while not self._exit_event.is_set():
            self._step()


    def stop(self) -> None:
        """."""
        self._exit_event.set()
        # with self._tick_lock:
        #     state = self.__mosaic_client.save()
        self._client.on_stop()

# Endpoint.
class PyClient():
    def __init__(self) -> None:
        self.model = CNN()
        # tensors, dtypes, shapes = self.serialize_local_model(self.model)
        tensors = self.serialize_torch_model(self.model)
        print(self.model)
        print("\n")
        print(tensors[5])
        super().__init__()

    def train_single_update(self, training_input: Optional[list]):
        print("\t\tPyClient: Training ...")
        time.sleep(3.0)
        print("\t\tPyClient: Training done.")
        # return self.model

    def serialize_torch_model(self, model):
        tensors = [[list(val.cpu().numpy()), val.cpu().dtype, val.cpu().numpy().shape] for _, val in model.state_dict().items()]
        # dtypes = [val.cpu().dtype for _, val in model.state_dict().items()]
        # shapes = [val.cpu().numpy().shape for _, val in model.state_dict().items()]
        # return tensors, dtypes, shapes
        return tensors

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
