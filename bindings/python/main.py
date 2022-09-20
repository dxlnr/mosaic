from python_sdk.client import spawn_client, PyClient

def main():
    c1 = spawn_client("http://127.0.0.1:8081", PyClient)
    # try:
    #     c1.join()
    # except KeyboardInterrupt:
    #     c1.stop()

if __name__ == "__main__":
    main()
