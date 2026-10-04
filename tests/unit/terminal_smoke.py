"""Exercise the built app in a Unix PTY or a disposable Windows console."""

from contextlib import closing
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time
import tomllib


TIMEOUT = 15
TITLE = "Cross-platform task"


def run_unix(command, settings):
    import fcntl
    import pty
    import select
    import struct
    import termios

    master, slave = pty.openpty()
    original_mode = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
    output = bytearray()
    process = None
    try:
        process = subprocess.Popen(
            command, stdin=slave, stdout=slave, stderr=slave,
            start_new_session=True, env={**os.environ, "TERM": "xterm-256color"},
        )

        def read_output():
            if select.select([master], [], [], 0.1)[0]:
                output.extend(os.read(master, 65536))

        def wait_for(text, start):
            deadline = time.monotonic() + TIMEOUT
            while text not in output[start:]:
                read_output()
                if process.poll() is not None or time.monotonic() > deadline:
                    raise AssertionError(f"Missing terminal output {text!r}: {output!r}")

        def send_and_wait(keys, text):
            start = len(output)
            os.write(master, keys)
            wait_for(text, start)

        wait_for(b"argv-todo", 0)
        if settings:
            send_and_wait(b"\x1b", b"configuration")
            send_and_wait(b"j\r", b"split")
            send_and_wait(b"\x1b", b"argv-todo")
            os.write(master, b"q")
        else:
            # Include Vim command letters in one paste transaction.
            os.write(master, b"i\x1b[200~" + TITLE.encode() + b"\x1b[201~\rxq")

        deadline = time.monotonic() + TIMEOUT
        while process.poll() is None:
            read_output()
            if time.monotonic() > deadline:
                raise AssertionError(f"App did not quit: {output!r}")
        while select.select([master], [], [], 0)[0]:
            read_output()
        assert process.returncode == 0, output
        assert termios.tcgetattr(slave) == original_mode, "Raw mode was not restored"
        for sequence in [b"\x1b[?2004l", b"\x1b[0 q", b"\x1b[?1049l", b"\x1b[?25h"]:
            assert sequence in output, f"Missing cleanup sequence: {sequence!r}"
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait()
        os.close(master)
        os.close(slave)


def run_windows(command, settings):
    import ctypes
    from ctypes import wintypes
    import msvcrt

    class KeyEvent(ctypes.Structure):
        _fields_ = [
            ("down", wintypes.BOOL), ("repeat", wintypes.WORD),
            ("virtual_key", wintypes.WORD), ("scan_code", wintypes.WORD),
            ("character", wintypes.WCHAR), ("modifiers", wintypes.DWORD),
        ]

    class EventData(ctypes.Union):
        _fields_ = [("key", KeyEvent), ("padding", ctypes.c_byte * 16)]

    class InputRecord(ctypes.Structure):
        _fields_ = [("event_type", wintypes.WORD), ("event", EventData)]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.AllocConsole.restype = wintypes.BOOL
    kernel.FreeConsole.restype = wintypes.BOOL
    kernel.GetConsoleMode.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetConsoleMode.restype = wintypes.BOOL
    kernel.WriteConsoleInputW.argtypes = [
        wintypes.HANDLE, ctypes.POINTER(InputRecord), wintypes.DWORD,
        ctypes.POINTER(wintypes.DWORD),
    ]
    kernel.WriteConsoleInputW.restype = wintypes.BOOL

    # Detach only this helper process from any runner console.
    kernel.FreeConsole()
    if not kernel.AllocConsole():
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        with open("CONIN$", "r+b", buffering=0) as source, \
                open("CONOUT$", "wb", buffering=0) as destination:
            handle = msvcrt.get_osfhandle(source.fileno())

            def console_mode():
                mode = wintypes.DWORD()
                if not kernel.GetConsoleMode(handle, ctypes.byref(mode)):
                    raise ctypes.WinError(ctypes.get_last_error())
                return mode.value

            original_mode = console_mode()
            with subprocess.Popen(
                command, stdin=source, stdout=destination, stderr=subprocess.PIPE,
                close_fds=False,
            ) as process:
                try:
                    deadline = time.monotonic() + TIMEOUT
                    while console_mode() & 7:
                        if process.poll() is not None:
                            raise AssertionError(process.stderr.read().decode(errors="replace"))
                        if time.monotonic() > deadline:
                            raise AssertionError("App did not enter raw mode")
                        time.sleep(0.05)

                    keys = "\x1bj\r\x1bq" if settings else f"i{TITLE}\rxq"
                    records = []
                    for character in keys:
                        for down in [True, False]:
                            record = InputRecord()
                            record.event_type = 1  # KEY_EVENT
                            virtual_key = {"\r": 0x0D, "\x1b": 0x1B}.get(character, 0)
                            record.event.key = KeyEvent(
                                down, 1, virtual_key, 0, character,
                                0x10 if character.isupper() else 0,  # SHIFT_PRESSED
                            )
                            records.append(record)
                    events = (InputRecord * len(records))(*records)
                    written = wintypes.DWORD()
                    if not kernel.WriteConsoleInputW(handle, events, len(events), ctypes.byref(written)):
                        raise ctypes.WinError(ctypes.get_last_error())
                    assert written.value == len(events), "Console input was incomplete"
                    _, error = process.communicate(timeout=TIMEOUT)
                    assert process.returncode == 0, error.decode(errors="replace")
                    assert console_mode() == original_mode, "Console input mode was not restored"
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.wait()
    finally:
        kernel.FreeConsole()


def main():
    binary = Path(sys.argv[1]).resolve()
    if os.name == "nt":
        binary = binary.with_suffix(".exe")
    assert binary.is_file(), f"Build the app first: {binary}"
    with tempfile.TemporaryDirectory(prefix="argv-todo smoke test ") as temporary:
        folder = Path(temporary) / "task storage"
        database = folder / "tasks.sql"
        command = [str(binary), "--db", str(database)]
        for option in ["--help", "--version"]:
            subprocess.run(command + [option], check=True, capture_output=True, timeout=TIMEOUT)
            assert not folder.exists(), f"{option} unexpectedly created storage"

        run_terminal = run_windows if os.name == "nt" else run_unix
        run_terminal(command, settings=False)
        with closing(sqlite3.connect(database)) as connection:
            tasks = connection.execute("SELECT title, done FROM todos").fetchall()
        assert tasks == [(TITLE, 1)], f"Task creation or completion failed: {tasks!r}"

        run_terminal(command, settings=True)
        with (folder / "config.toml").open("rb") as config:
            assert tomllib.load(config)["task_view"] == "split", "Setting did not persist"
        assert "# Task layouts:" in (folder / "config.toml").read_text(), "Config comments were lost"
    print("Terminal startup, input, storage, settings, and cleanup passed.")


if __name__ == "__main__":
    main()
