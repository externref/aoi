# 🍥 aoi
A simple python based sqlite3 CLI.

---

## Usage 
```sh
$python -m aoi [-c "path to connect"]
```
using the -c/--connect option will connect the application to the provided db file path.

If no option is provided, `:memory:` ( in memory database ) will be used.

## Additional Features

Apart from normal sqlite quries you can run the following commands within the CLI:

`:h/:help`: Get help for commands.

`:q/:quit`: Exit the CLI.

`:r/:recent [amount=5]`: Show last [amount=5] queries.

---
## Requirements
* Python (3.8 or later)

Installing aoi in your environment using pip, poetry or any favourable package manager
```sh
# pip
$python -m pip install git+https://github.com/sarthhh/aoi.git
# poetry
$poetry add git+https://github.com/sarthhh/aoi.git
```