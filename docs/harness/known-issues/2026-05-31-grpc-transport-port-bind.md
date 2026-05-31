# 2026-05-31-grpc-transport-port-bind

**Sensor**: grpc-transport
**Date**: 2026-05-31
**Reason**: CI/tests invoking `tests/grpc_transport.rs` fail in this environment with `Operation not permitted` when binding sockets (`bind port 0`), indicating an execution-environment restriction unrelated to code correctness.

**Impact**: This is a reproducible local/sandbox limitation that affects the grpc transport integration test suite (`tests/grpc_transport.rs`), specifically scenarios `scenario_a_initialize_returns_server_info` through `scenario_g_unknown_method_returns_error_response`.

**Mitigation**: Run `sensors.sh` with `--exclude-sensor grpc-transport --known-issue docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md --reason "sandbox socket bind restriction"`.

**Recorded in**: `docs/harness/progress/2026-05-30-harness-bootstrap.md` and `docs/harness/progress.md`
