from __future__ import annotations

import typing

import colorama

if typing.TYPE_CHECKING:
    from aoi.base import Session


UNDEFINED = object()
EMOJI = "\U0001f365"

cmds: dict[str, str] = {
    ":q | :quit": "Stops the cli session.",
    ":h | :help": "Shows this help message.",
    ":r | :recent": "List recently used commands.",
}


def error_print(msg: str) -> None:
    print(colorama.Fore.LIGHTMAGENTA_EX, msg, colorama.Style.RESET_ALL, sep="")


def process_commands(data: str, session: Session) -> typing.Any:
    if data in (":q", ":quit"):
        session.stop()
    elif data in (":h", ":help"):
        [
            print(
                colorama.Style.BRIGHT,
                colorama.Fore.GREEN,
                f"[{cmd}]",
                colorama.Style.RESET_ALL,
                ": ",
                colorama.Fore.CYAN,
                info,
                colorama.Style.RESET_ALL,
                sep="",
                end="\n",
            )
            for cmd, info in cmds.items()
        ]
    elif data.startswith(":recent"):
        if data[7:] and data[7:].strip().isnumeric():
            print("\n".join(session.recent_queries[-int(data[7:]) :]))
        else:
            print("\n".join(session.recent_queries[-5:]))

    elif data.startswith(":r"):
        if data[3:] and data[3:].strip().isnumeric():
            print("\n".join(session.recent_queries[-int(data[3:]) :]))
        else:
            print("\n".join(session.recent_queries[-5:]))

    else:
        error_print(f"No command named {data} found, use :h to get a list of all commands")
