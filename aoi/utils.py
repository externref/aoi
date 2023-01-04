from __future__ import annotations

import enum
import typing

if typing.TYPE_CHECKING:
    from aoi.base import Session


UNDEFINED = object()
EMOJI = "\U0001f365"


def success_print(msg: str) -> None:
    print(ANSIBuilder().set_cursor(ANSI.GREEN_TEXT).write(msg))


def error_print(msg: str) -> None:
    print(ANSIBuilder().set_cursor(ANSI.PINK_TEXT).write(msg))


class CommandHandler:
    COMMANDS: dict[str, str] = {
        ":q | :quit": "Stops the cli session.",
        ":h | :help": "Shows this help message.",
        ":r | :recent": "List recently used commands.",
        ":t | :tables": "List names of the tables in database.",
    }

    @classmethod
    def send_help(cls) -> None:
        for cmd, info in cls.COMMANDS.items():
            print(
                ANSIBuilder()
                .set_cursor(ANSI.BOLD_FORMAT, ANSI.GREEN_TEXT)
                .write(f"[{cmd}]")
                .set_cursor(ANSI.NORMAL_FORMAT)
                .write(": ")
                .set_cursor(ANSI.CYAN_TEXT)
                .write(info)
            )

    @staticmethod
    def send_recent(n: int, session: Session) -> None:
        builder = ANSIBuilder().set_cursor(ANSI.YELLOW_TEXT)
        [builder.write(query, newline=True) for query in session.recent_queries[-n:]]
        print(builder)

    @staticmethod
    def send_tables(session: Session) -> None:
        print(
            ANSIBuilder()
            .set_cursor(ANSI.GREEN_TEXT)
            .write("Tables in ")
            .set_cursor(ANSI.BOLD_FORMAT, ANSI.YELLOW_TEXT)
            .write(f"[{session.file}]: ")
            .set_cursor(ANSI.NORMAL_FORMAT, ANSI.BLUE_TEXT)
            .write(", ".join([t[0] for t in tables]) if (tables := session.connection.table_names()) else "None")
        )

    @classmethod
    def process_command(cls, data: str, session: Session) -> None:
        if data in (":q", ":quit"):
            session.stop()
        elif data in (":h", ":help"):
            cls.send_help()
        elif data in (":t", ":tables"):
            cls.send_tables(session)
        elif data.startswith(":recent"):
            if data[7:] and data[7:].strip().isnumeric():
                cls.send_recent(int(data[7:]), session)
            else:
                cls.send_recent(5, session)
        elif data.startswith(":r"):
            if data[3:] and data[3:].strip().isnumeric():
                cls.send_recent(int(data[3:]), session)
            else:
                cls.send_recent(5, session)
        else:
            error_print(f"No command named {data} found, use :h to get a list of all commands")


class ANSI(enum.IntEnum):
    NORMAL_FORMAT = 0
    BOLD_FORMAT = 1
    UNDERLINE_FORMAT = 4
    GRAY_TEXT = 30
    RED_TEXT = 31
    GREEN_TEXT = 32
    YELLOW_TEXT = 33
    BLUE_TEXT = 34
    PINK_TEXT = 35
    CYAN_TEXT = 36
    WHITE_TEXT = 37
    FIREFLY_DARK_BLUE_BACKGROUND = 40
    ORANGE_BACKGROUND = 41
    MARBLE_BLUE_BACKGROUND = 42
    GREYISH_TURQUOISE_BACKGROUND = 43
    GRAY_BACKGROUND = 44
    INDIGO_BACKGROUND = 45
    LIGHT_GRAY_BACKGROUND = 46
    WHITE_BACKGROUND = 0


class ANSIBuilder:
    bucket: list[str]
    current_cursor: str

    def __init__(self) -> None:
        self.bucket = []
        self.current_cursor = ""

    def __str__(self) -> str:
        return self.get_str()

    def set_cursor(self, *args: ANSI | int) -> ANSIBuilder:
        self.current_cursor = (
            f"\033[{';'.join(map(lambda arg: str(arg.value) if isinstance(arg, ANSI) else str(arg), args))}m"
        )
        self.bucket.append(self.current_cursor)
        return self

    def write(self, text: str, *, newline: bool = False) -> ANSIBuilder:
        self.bucket.append(text + ("\n" if newline is True else ""))

        return self

    def reset(self) -> ANSIBuilder:
        self.current_cursor = "\033[0m"
        self.bucket.append(self.current_cursor)
        return self

    def get_str(self) -> str:
        self.set_cursor(ANSI.NORMAL_FORMAT)
        return "".join(self.bucket)
