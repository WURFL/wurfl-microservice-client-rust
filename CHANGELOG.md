### 0.2.1
- Fixed device lookups failing to deserialize when the server omits the `error` field on success (WM server >= 2.18.0). `error`, `mtime` and `ltime` now default when absent
- `get_api_version()` now returns the crate version from Cargo.toml
- Added server-independent tests with a mock server

### 0.2.0
- Updated rust edition used and project dependencies

### 0.1.1
- Added changelog. Small documentation changes

### 0.1.0
- Initial version: GetInfo, LookupUseragent, LookupRequest, LookupDeviceId and other functions to set cache and requested capabilitities