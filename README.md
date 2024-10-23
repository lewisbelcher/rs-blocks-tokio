# WIP: rs-blocks-tokio

This is a work-in-progress repo for an async version of
[`rs-blocks`](https://github.com/lewisbelcher/rs-blocks).

The multi-threaded implementation is replaced by a single-threaded async
implementation which makes more use of macros to keep the code base much more
concise.

## TODOs

- Write tests
- Introduce logging/tracing

### Implementation Questions

- Should `into_stream` be be `try_into_stream`? Some blocks (primarily battery)
  have logic that could return an error while setting up the stream.
