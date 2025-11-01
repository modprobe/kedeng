# zmq-tool

## Usage

- Connection parameters:
    1. Use preset:
       ```shell
       zmq-tool -p ns-info-plus ...
       ```
    2. Use host and port
       ```shell
       zmq-tool -H pubsub.besteffort.ndovloket.nl -p 7664 ...
       ```

- Usage mode:
    1. UI
       ```shell
       zmq-tool [...] ui
       ```
    2. CLI
       ```shell
       zmq-tool [...] listen -d path/to/output-dir/
       ```