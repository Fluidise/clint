# Commit Message Validation

This glossary defines the language used to assess fluidise-clint's commit-message validation behavior and performance.

## Language

**CLI validation latency**:
The elapsed time from launching fluidise-clint to process completion after emitting its validation result. It includes automatic configuration discovery, message reading, validation, and JSON output when using the default output format.
_Avoid_: Startup latency when referring to the complete validation invocation.

**Hook validation latency**:
The elapsed time from starting the repository-provided commit-msg hook script to its exit, including locating the linter, validation, and reporting. It excludes Git's work before invoking the script and is distinct from CLI validation latency.
_Avoid_: CLI validation latency when referring to the complete hook wrapper invocation.
