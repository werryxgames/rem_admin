# rem_admin
RemAdmin - Remote Administration Tool with command line and graphical user interfaces.

To use CLI, enable `controller-cli` feature, to use GUI, enable `controller-gui`.

GUI has intuitive interface.

# CLI commands

`exit`/`quit`/`q` - Quit from RemAdmin CLI shell

`echo` - Display a line of text

`args` - Display given arguments

`list` - Show list of connected clients, their indexes and machine ids

`select` - Select client to operate with next commands

`test` - Test connection with selected client by sending a random number and waiting for same number response

`text` - Display message box with text to selected client

`confirm` - Display confirmation box with text to selected client

`abort` - Abort long command (`text`, `confirm`, `prompt`, `cmd`) immediately

`moveto` - Move cursor of selected client to absolute position

`moveby` - Move cursor of selected client relatively to current position

`type` - Simulate typing string of text on keyboard of selected client

`clipget` - Get content of clipboard from selected client

`clipset` - Set content of clipboard from selected client

`prompt` - Display message box with text input to selected client

`cmd` - Execute console command to selected client

`screenshot` - Capture screen of selected client and save it to a file

RemAP (protocol) description is in [Protocol.md](Protocol.md)

Tested on Linux (Ubuntu), Windows 10 and Windows 8.1
