from abc import ABC, abstractmethod
from http import server
import threading
import time
from typing import Optional, List
from justbackoff import Backoff
import itertools
import numpy as np

from .model import CNN
from python_sdk import modalic_sdk

print(modalic_sdk.__all__)

import logging
modalic_sdk.init_logging()
LOG = logging.getLogger("client")


class ModalicClient(threading.Thread):
    def __init__(self, server_address: str, client, state: Optional[List[int]] = None, scalar: float = 1.0):
        print("\n")
        self._modalic_client = modalic_sdk.Client(server_address, scalar, state)

        # Client API
        #
        # https://github.com/python/cpython/blob/3.9/Lib/multiprocessing/process.py#L80
        # stores the Client class with its args and kwargs.
        self._client = client
        # Instantiate global model to None.
        self._global_model = None
        #
        self._error_on_fetch_global_model = False
        # threading internals
        self._exit_event = threading.Event()
        self._poll_period = Backoff(min_ms=100, max_ms=10000, factor=1.2, jitter=False)

        # Primitive lock objects. Once a thread has acquired a lock,
        # subsequent attempts to acquire it block, until it is released;
        # any thread may release it.
        self._step_lock = threading.Lock()
        super().__init__(daemon=True)

        # some configs for later
        self.conf_standalone = True

        print(f"\nModalicClient init done.")

    @abstractmethod
    def serialize_local_model(self) -> list:
        r"""
        Serializes the local model into a `list` data type. The data type of the
        elements must match the data type attached as metadata.
        :returns: The local model (self.model) as a `list`.
        """
        raise NotImplementedError()

    @abstractmethod
    def on_new_global_model(self, model):
        r"""."""
        raise NotImplementedError()

    def _fetch_global_model(self):
        # LOG.debug("fetch global model")
        try:
            global_model = self._modalic_client.global_model()
        except (
            modalic_sdk.GlobalModelUnavailable,
            modalic_sdk.GlobalModelDataTypeMisMatch,
        ) as err:
            print("failed to get global model: %s", err)
            self._error_on_fetch_global_model = True
        else:
            if global_model is not None:
                self._global_model = self._client.deserialize_training_input(
                    global_model
                )
            else:
                self._global_model = None
            self._error_on_fetch_global_model = False

    def _set_local_model(self, local_model: list):
        """
        Sets a local model. This method can be called at any time. Internally the
        participant first caches the local model. As soon as the participant is selected as an
        update participant, the currently cached local model is used. This means that the cache
        is empty after this operation.

        If a local model is already in the cache and `set_local_model` is called with a new local
        model, the current cached local model will be replaced by the new one.
        If the participant is an update participant and there is no local model in the cache,
        the participant waits until a local model is set or until a new round has been started.

        Args:
            local_model: The local model in the form of a list. The data type of the
                elements must match the data type defined in the coordinator configuration.

        Raises:
            LocalModelLengthMisMatch: If the length of the local model does not match the
                length defined in the coordinator configuration.
            LocalModelDataTypeMisMatch: If the data type of the local model does not match
                the data type defined in the coordinator configuration.
        """
        try:
            print("test test test")
            self._modalic_client.set_model(local_model)
        except (
            modalic_sdk.UninitializedParticipant,
        ) as err:
            print("failed to set local model: %s", err)
            self._exit_event.set()

    def run(self):
        self._client = self._client()

        try:
            self._run()
        except Exception as err:
            print("run error ? : ", err)
            self._exit_event.set()

    def _run(self):
        print(f"\t : Protocol is beeing performed : ")
        while not self._exit_event.is_set():
            self._step()

    def _step(self):
        with self._step_lock:
            print(f"\t(n) Performing single step: ")
            self._modalic_client.tick()

            # self._fetch_global_model()

            if (
                self._modalic_client.new_global_model()
                or self._error_on_fetch_global_model
            ):
                print("fetching.")
                self._fetch_global_model()

                if not self._error_on_fetch_global_model:
                    self._client.on_new_global_model(self._global_model)

            if (
                self._modalic_client.should_set_model()
                and self._client.participate_in_update_task()
                and not self._error_on_fetch_global_model
            ):
                print("train.")
                self._train()

            # self._train()
            made_progress = self._modalic_client.made_progress()
            print("________________________________________made progress?: ", made_progress)

        if made_progress:
            self._poll_period.reset()
            self._exit_event.wait(timeout=self._poll_period.duration())
        else:
            self._exit_event.wait(timeout=self._poll_period.duration())

    def _train(self):
        print("Start training.")

        local_update = self._client.train_single_update(self._global_model)
        # local_update_ = self._client.serialize_training(local_update)
        print(local_update)
        try:
            print("helloooooo")
            self._set_local_model(local_update)
        except (
            modalic_sdk.LocalModelLengthMisMatch,
            modalic_sdk.LocalModelDataTypeMisMatch,
        ) as err:
            print("helloooooo noooooooooo")
            print("failed to set local model: %s", err)

    def _run_standalone(self):
        print(f"\t : Protocol is beeing performed : ")
        while not self._exit_event.is_set():
            self._step()

    def stop(self) -> None:
        """."""
        self._exit_event.set()
        with self._step_lock:
            print("client stopped.")
            # state = self.__modalic_client.save()
        # self._client.on_stop()


# Endpoint.
class PyClient:
    def __init__(self) -> None:
        self.model = CNN()

        # tensors = self._torch_model_to_modalic_tensors(self.model)
        self.tensors = [0.1, 0.2, 0.345, 0.3]

        super().__init__()

    def train_single_update(self, training_input: Optional[list]):
        print("\t\tPyClient: Training ...")
        time.sleep(2.0)
        print("\t\tPyClient: Training done.")
        return self.tensors

    def on_new_global_model(self, model):
        r"""."""
        print("client calls: on_new_global_model")

    def participate_in_update_task(self) -> bool:
        r"""."""
        return True

    # def _torch_model_to_modalic_tensors(self, model):
    #     r"""."""
    #     layers = [val.cpu().numpy() for _, val in model.state_dict().items()]
    #     tensors = [
    #         modalic_sdk.modalicTensor(list(layer.flatten()), 0, list(layer.shape))
    #         for layer in layers
    #     ]
    #     return tensors


class modalicTensor:
    def __init__(self) -> None:
        pass


def spawn_client(
    server_address: str,
    client: PyClient,
    state: Optional[List[int]] = None,
    scalar: float = 1.0,
):
    """."""
    modalic_client = ModalicClient(server_address, client)
    # spawns the internal modalic client in a separate thread.
    # `start` calls the `run` method of `modalicClient`
    # https://docs.python.org/3.8/library/threading.html#threading.Thread.start
    # https://docs.python.org/3.8/library/threading.html#threading.Thread.run
    modalic_client.start()
    return modalic_client
