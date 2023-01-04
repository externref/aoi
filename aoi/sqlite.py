from __future__ import annotations

import sqlite3
import typing

from aoi.utils import error_print, success_print

if typing.TYPE_CHECKING:
    from aoi.base import Session


class Connection:
    def __init__(self, path: str) -> None:
        self.path = path

    def connect(self) -> None:
        try:
            self.connection = sqlite3.connect(self.path)
        except sqlite3.OperationalError:
            print(error_print(f"[Operational Error:] Unable to connect to the path: {self.path}"))
            raise KeyboardInterrupt

    def table_names(self) -> tuple[str]:
        return self.connection.execute("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name;").fetchall()

    def run_sql(self, query: str, session: Session) -> typing.Any:
        session.recent_queries.append(query)
        try:
            cursor = self.connection.execute(query)
            print(data) if (data := cursor.fetchall()) else success_print(
                "\u2705 Executed query."
                if (not query.lower().startswith("select"))
                else "\U0001f645 No Records found."
            )
        except sqlite3.OperationalError as e:
            error_print(f"[Operationl Error:] {e.args[0]}")
