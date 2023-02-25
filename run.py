#!/usr/bin/env python3

import argparse
import typing
import subprocess
import logging
import shutil
import shlex
import os

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger("MaelstromInvoker")


DESCRIPTION = """
Run the maelstrom tests for a given fly.io dist-sys challenge.

Currently, the following challenges are implemented:

    01: Echo
    02: Unique ID Generation
    03: Broadcast

"""

IMPLEMENTED_CHALLENGES = {1: "Echo", 2: "Unique ID Generation", 3: "Broadcast"}

CHALLENGE_WORKLOADS = {1: "echo", 2: "unique-ids", 3: "broadcast"}


def parse_args():
    parser = argparse.ArgumentParser(prog="MaelstromInvoker", description=DESCRIPTION)
    parser.add_argument(
        "challenge",
        metavar="CHALLENGE_ID",
        nargs="+",
        type=int,
        help="The ids of the challenges to run.",
    )

    parser.add_argument(
        "--time-limit",
        type=int,
        help="The time limit to delegate to maelstrom.",
        default=5,
    )

    args = parser.parse_args()
    if any(
        challenge_id not in IMPLEMENTED_CHALLENGES for challenge_id in args.challenge
    ):
        raise ValueError(
            f"Invalid or unimplemented challenge id. Recognized challenge ids are: {str(IMPLEMENTED_CHALLENGES)}"
        )
    return args


def build_binaries():
    return subprocess.run(["cargo", "build", "--release"], check=True)


def copy_binaries_to_maelstrom_solutions(challenges: typing.List[int]):
    """Copy over the generated Rust binaries to maelstrom's solutions directory."""

    SRC_DIR = "target/release"
    DEST_DIR = "maelstrom/solutions"

    for file_path in os.listdir(SRC_DIR):
        src_file_path = os.path.join(SRC_DIR, file_path)

        # Copy over the executables.
        if os.path.isfile(src_file_path) and os.access(src_file_path, os.X_OK):
            if not any(
                file_path.startswith(f"{challenge_id:02d}")
                for challenge_id in challenges
            ):
                continue
            dest_file_path = os.path.join(DEST_DIR, file_path)
            shutil.copy(src_file_path, dest_file_path)
            logging.debug("Copied %s to %s", src_file_path, dest_file_path)


def run_test(challenge: int, time_limit: int):
    """Run maelstrom for a given challenge."""

    command = None
    BIN_DIR = "maelstrom/solutions"

    for file_path in os.listdir(BIN_DIR):
        file_name = file_path
        file_path = os.path.join(BIN_DIR, file_path)
        is_file = os.path.isfile(file_path)
        is_executable = os.access(file_path, os.X_OK)
        is_challenge = file_name.startswith(f"{challenge:02d}")

        if is_file and is_executable and is_challenge:
            command = f"./maelstrom test -w {CHALLENGE_WORKLOADS[challenge]} --bin solutions/{file_name} --time-limit {time_limit:01d}"
            break

    if command is None:
        raise ValueError(f"Couldn't find a test for challenge: {str(challenge)}")

    logger.info(
        'Running maelstrom test workload %s for challenge "%s" (id: %s)',
        CHALLENGE_WORKLOADS[challenge],
        IMPLEMENTED_CHALLENGES[challenge],
        challenge,
    )
    result = subprocess.run(shlex.split(command), check=True, cwd="maelstrom")
    return result


def main():
    args = parse_args()
    logger.info("Arguments: %s", args)
    build_binaries()
    copy_binaries_to_maelstrom_solutions(args.challenge)
    for challenge_id in args.challenge:
        run_test(challenge_id, args.time_limit)


if __name__ == "__main__":
    main()
