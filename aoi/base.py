import argparse
import sys

import colorama

from aoi._misc import process_commands
from aoi.sqlite import connect, run_sql

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

        connect(file)

    def start(self) -> None:
        self.running = True
        print(
            colorama.Fore.BLUE,
            colorama.Style.BRIGHT,
            "Welcome to ",
            colorama.Fore.CYAN,
            "\U0001f365 aoi",
            colorama.Fore.BLUE,
            "!",
            colorama.Style.RESET_ALL,
            f" [connected to: \033[1;33m{self.file}\033[0m]",
            sep="",
            end="\n",
        )
        print(
            colorama.Fore.LIGHTBLUE_EX,
            "type :h for list of inbuilt commands.",
            colorama.Style.RESET_ALL,
            sep="",
        )

        while self.running is True:
            data = input("> ")
            if data.startswith(":"):
                process_commands(data, self)
                continue
            while not data.strip().endswith(";"):
                data += f" {input('- ')}"

            run_sql(data, self)
        raise KeyboardInterrupt

    def stop(self) -> None:
        self.running = False


def main() -> None:
    args: Args = parser.parse_args()
    file = args.connect or ":memory:"
    if args.version and args.connect is None:
        print(
            colorama.Fore.BLUE,
            colorama.Style.BRIGHT,
            "aoi\U0001f365  version: ",
            colorama.Fore.GREEN,
            colorama.Style.NORMAL,
            "0.1.0\n",
            colorama.Fore.BLUE,
            colorama.Style.BRIGHT,
            "Python version: ",
            colorama.Style.NORMAL,
            colorama.Fore.GREEN,
            sys.version,
            colorama.Style.RESET_ALL,
            sep="",
        )
        return
    try:
        (session := Session(file)).start()
    except KeyboardInterrupt:
        print(
            "\n" if session.running else "",
            colorama.Fore.BLUE,
            colorama.Style.BRIGHT,
            "Thank you for using ",
            colorama.Fore.CYAN,
            "\U0001f365 aoi",
            colorama.Fore.BLUE,
            "!",
            sep="",
        )
