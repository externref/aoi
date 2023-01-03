from __future__ import annotations

import sqlite3
import typing

import colorama

from aoi._misc import UNDEFINED, error_print

if typing.TYPE_CHECKING:
    from aoi.base import Session


connection: sqlite3.Connection = UNDEFINED


def connect(path: str) -> None:
    global connection
    try:
        connection = sqlite3.connect(path)
    except sqlite3.OperationalError:
        print(error_print(f"[Operational Error:] Unable to connect to the path: {path}"))
        raise KeyboardInterrupt


def _success() -> None:
    print(colorama.Style.BRIGHT, colorama.Fore.GREEN, "Executed successfully.", colorama.Style.RESET_ALL, sep="")


def run_sql(query: str, session: Session) -> typing.Any:
    session.recent_queries.append(query)
    try:
        cursor = connection.execute(query)
        print(data) if (data := cursor.fetchall()) else _success()
    except sqlite3.OperationalError as e:
        error_print(f"[Operationl Error:] {e.args[0]}")
