from __future__ import annotations

import argparse
import sys

from aoi.sqlite import Connection
from aoi.utils import ANSI, ANSIBuilder, CommandHandler, error_print

parser = argparse.ArgumentParser("aoi", usage="aoi *[options]", description="SQLITE3 in CLI.", add_help=True)

parser.add_argument("-c", "--connect", help="The sqlite file to connect.")
parser.add_argument("-v", "--version", help="Check version of the application.", action="store_true")


class Args(argparse.Namespace):
    connect: str | None
    version: str | None


class Session:
    recent_queries: list[str] = []
    running: bool

    def __init__(self, file: str) -> None:
        self.file = file
        self.cmd_handler = CommandHandler()
        self.connection = Connection(file)

    def print_welcome_message(self) -> None:
        print(
            ANSIBuilder()
            .set_cursor(ANSI.BOLD_FORMAT, ANSI.BLUE_TEXT)
            .write("Welcome to ")
            .set_cursor(ANSI.CYAN_TEXT)
            .write("\U0001f365 aoi")
            .set_cursor(ANSI.BLUE_TEXT)
            .write("!")
            .set_cursor(ANSI.NORMAL_FORMAT)
            .write(f" [connected to: \033[1;33m{self.file}\033[0m]", newline=True)
            .set_cursor(ANSI.BLUE_TEXT)
            .write("type :h for list of inbuilt commands.")
        )

    def print_end_message(self) -> None:
        print(
            ANSIBuilder()
            .write("\n" if self.running else "")
            .set_cursor(ANSI.BOLD_FORMAT, ANSI.BLUE_TEXT)
            .write("Thank you for using ")
            .set_cursor(ANSI.BOLD_FORMAT, ANSI.CYAN_TEXT)
            .write("\U0001f365  aoi")
            .set_cursor(ANSI.BLUE_TEXT)
            .write("!")
        )

    def start(self) -> None:
        self.running = True
        self.print_welcome_message()
        self.connection.connect()

        while self.running is True:
            data = input("> ")
            if data.startswith(":"):
                self.cmd_handler.process_command(data, self)
                continue
            while not data.strip().endswith(";"):
                data += f" {input('- ')}"

            self.connection.run_sql(data, self)
        raise KeyboardInterrupt

    def stop(self) -> None:
        self.running = False


def main() -> None:
    args: Args = parser.parse_args()
    file = args.connect or ":memory:"
    if args.version and args.connect is None:
        print(
            ANSIBuilder()
            .set_cursor(ANSI.BOLD_FORMAT, ANSI.BLUE_TEXT)
            .write("\U0001f365 aoi version: ")
            .set_cursor(ANSI.NORMAL_FORMAT, ANSI.GREEN_TEXT)
            .write("0.1.2", newline=True)
            .set_cursor(ANSI.BOLD_FORMAT, ANSI.BLUE_TEXT)
            .write("\U0001f40d Python version: ")
            .set_cursor(ANSI.NORMAL_FORMAT, ANSI.GREEN_TEXT)
            .write(sys.version)
        )
        return
    if args.connect and args.version:
        error_print("\u26a0\ufe0f  Only one operation allowed at a time.")
        return
    try:
        (session := Session(file)).start()
    except KeyboardInterrupt:
        session.print_end_message()
