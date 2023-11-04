# RemAP - Remote Administration Protocol.

## Client packets
| Name | Code | Description | Arguments |
| --- | --- | --- | --- |
| C_AUTH | 0x00 | Authorization packet from client | `u64 version`, `u64 auth_part1` |
| CE_AUTH_PART | 0x01 | Invalid authorization part from server |
| CE_AUTH_VERSION | 0x02 | Unsupported server version | `u64 min_version`, `u64 max_version` |
| C_AUTH_OK | 0x03 | Server authorized | `u128 id` |
| R_TEST_ECHO | 0x04 | Answer to **M_TEST** | `u32 number` |
| R_OK | 0x05 | Command executed successfully | `u64 cmd_id` |
| R_FAIL | 0x06 | Command executed with error | `u64 cmd_id` |
| R_FAIL_TEXT | 0x07 | Command executed with error | `String text`, `u64 cmd_id` |
| R_OK_TEXT | 0x08 | Command executed successfully with response | `String text`, `u64 cmd_id` |
| R_ABORTED | 0x09 | Command aborted as answer to **M_ABORT** | `u64 cmd_id` |
| R_BOOL | 0x0A | Command executed with boolean result | `u64 cmd_id`, `bool result` |
| R_NOT_ABORTED | 0x0B | Command already executed or wasn't executed | `u64 cmd_id`, `bool executed` |
| C_CONTROL | 0x70 | Set mode of this client from *controlled* to *controller* | `String password` |
| C_CONTROL_ALL | 0x71 | Send packet to all *controlled* clients | `[u8] packet` |
| C_CONTROL_LIST | 0x72 | List all *controlled* clients |
| C_CONTROL_ONE | 0x73 | Send packet to one *controlled* client | `u128 id`, `[u8] packet` |
| C_CONTROL_MANY | 0x74 | Send packet to multiple *controlled* clients | `Vec<u128> ids`, `[u8] packet` |
| RESERVED | 0x80 - 0xFF | Reserved |

## Server packets
| Name | Code | Description | Arguments |
| --- | --- | --- | --- |
| S_AUTH | 0x00 | Authorization packet from server | `u64 version`, `u64 auth_part2` |
| SE_AUTH_PART | 0x01 | Invalid authorization part from client |
| SE_AUTH_VERSION | 0x02 | Unsupported client version | `u64 min_version`, `u64 max_version` |
| M_TEST | 0x03 | Tests connection with client | `u32 rand_number` |
| M_GUI | 0x04 | Show GUI window | `String title`, `String content` |
| M_ABORT | 0x05 | Abort command | `u64 cmd_id` |
| M_GUI_YES_NO | 0x06 | Show GUI window with buttons `Yes` and `No` | `String title`, `String content` |
| M_MOVE_CURSOR | 0x07 | Move cursor to absolute position (`x`; `y`) | `i32 x`, `i32 y` |
| M_MOVE_CURSOR_REL | 0x08 | Move cursor by (`x`; `y`) | `i32 x`, `i32 y` |
| M_TYPE_KEYBOARD | 0x09 | Simulate char being typed by keyboard | `char c` |
| M_CLIPBOARD_GET | 0x0A | Return data from clipboard |
| M_CLIPBOARD_SET | 0x0B | Replace old data in clipboard to new | `String new_data` |
| S_CONTROL_OK | 0x70 | Mode of that client set from *controlled* to *controller* |
| SE_CONTROL_PASS | 0x71 | Incorrect control password |
| SE_CONTROL_OFF | 0x72 | Control mode is turned off for this server |
| S_CONTROL_PACKET | 0x73 | Response packet for *controller* client | `[u8] packet` |
| RESERVED | 0x80 - 0xFF | Reserved |

Default *TCP* port is *20900*.

- [x] 1. Client connects to server
	It sends **C_AUTH** to server where `version` - numeric version of RemAdmin client, `auth_part1` - first 8 bytes of predefined authentication code, that should match in server and client.

- [x] 2. Server checks
	If client version isn't supported, sends **SE_AUTH_VERSION** with supported versions range and closes connection. If client sent invalid authorization part, sends **SE_AUTH_PART** and closes connection. Otherwise sends **S_AUTH** where `version` - numeric version of RemAdmin server, `auth_part2` - last 8 bytes of predefined authentication code, that should match in server and client.

- [x] 3. Client checks
	If server version isn't supported, sends **CE_AUTH_VERSION** with supported versions range and closes connection. If client server invalid authorization part, sends **CE_AUTH_PART** and closes connection. Otherwise sends **C_AUTH_OK** where `id` - unique identifier of client.

- [x] 4. Connected
	Now server can send command packets (**M_\***) to client and client can send response packets (**R_\***).

## Implemented
### Client codes
- [x] **C_AUTH**
- [x] **CE_AUTH_PART**
- [x] **CE_AUTH_VERSION**
- [x] **C_AUTH_OK**
- [x] **R_TEST_ECHO**
- [x] **R_OK**
- [x] **R_FAIL**
- [ ] **R_FAIL_TEXT**
- [ ] **R_OK_TEXT**
- [x] **R_ABORTED**
- [x] **R_BOOL**
- [x] **R_NOT_ABORTED**
- [ ] **C_CONTROL**
- [ ] **C_CONTROL_ALL**
- [ ] **C_CONTROL_LIST**
- [ ] **C_CONTROL_ONE**
- [ ] **C_CONTROL_MANY**

### Server codes
- [x] **S_AUTH**
- [x] **SE_AUTH_PART**
- [x] **SE_AUTH_VERSION**
- [x] **M_TEST**
- [x] **M_GUI**
- [x] **M_ABORT**
- [x] **M_GUI_YES_NO**
- [x] **M_MOVE_CURSOR**
- [x] **M_MOVE_CURSOR_REL**
- [ ] **M_TYPE_KEYBOARD**
- [ ] **M_CLIPBOARD_GET**
- [ ] **M_CLIPBOARD_SET**
- [ ] **S_CONTROL_OK**
- [ ] **SE_CONTROL_PASS**
- [ ] **SE_CONTROL_OFF**
- [ ] **S_CONTROL_PACKET**