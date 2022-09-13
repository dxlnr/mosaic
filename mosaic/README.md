<h1 align="center">
  <b>Mosaic: Aggregation Server</b><br>
</h1>

Server Layout is described here.


## Building

### Sourcing protoc
*prost-build* depends on the Protocol Buffers compiler, **protoc**, to parse .proto files into a representation that can be transformed into Rust. If set, prost-build uses the PROTOC for locating protoc. E.g. in a typical Linux installation:

```bash
PROTOC=/usr/bin/protoc
# Installing for Linux
sudo apt install -y protobuf-compiler
# Check the protoc version
protoc --version
```
If no PROTOC environment variable is set then prost-build will search the current path for protoc or protoc.exe. If prost-build can not find protoc via these methods the compile_protos method will fail.

