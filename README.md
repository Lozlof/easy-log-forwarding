# easy-log-forwarding
#### A demonstration of how the `better-logger` `relay` feature could be used in production
### For more details see:
#### https://crates.io/crates/better-logger
#### https://github.com/Gistyr/better-logger
### When to use the `relay` feature:
`better-logger` already includes built-in network logging for both native and wasm targets. By default, it sends log messages directly to a specified endpoint without needing any intermediary service.       
However, when running in the browser, `CORS (Cross-Origin Resource Sharing)` restrictions can prevent requests from being sent to external domains.        
**Example:**    
- This senario works without a relay: 
  - Website: https://test.com/
  - Logging endpoint: https://logs.test.com/
- This senario requires a relay:
  - Website: https://test.com/
  - Logging endpoint: https://hooks.slack.com/
- Fix this issue by creating the relay:
  - https://test.com/ -> https://logs.test.com -> https://hooks.slack.com/
## config.toml
#### Put in same directory as the executable
```toml
terminal_logs = true
terminal_log_lvl = "error"
wasm_logging = false
file_logs = false
file_log_lvl = ""
log_file_path = ""
network_logs = true
network_log_lvl = "info"
network_endpoint_url = "https://test.com"
debug_extra = false
async_logging = false
machine_name = "testing-VM-01"
container_name = "testing-container-01"
relay_listen_address = "0.0.0.0:8080"
relay_output_url = "https://test.com"
relay_cors_allowed_origins = ["*"]
relay_actix_workers = 1
[network_format]
type = "JsonText"
field = "text"
[relay_output_format]
type = "JsonText"
field = "text"
```
