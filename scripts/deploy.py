#!/usr/bin/env python3
"""Small portable helper script to build, deply and run a Rust application
on a remote machine, e.g. a Raspberry Pi"""
import argparse
import os
import sys
import platform
import time
from typing import Final


# This script can easily be adapted to other remote machines, Linux boards and
# remote configurations by tweaking / hardcoding these parameter, which generally are constant
# for a given board
DEFAULT_USER_NAME: Final = "pi"
DEFAULT_ADDRESS: Final = "192.168.1.236"
DEFAULT_TOOLCHAIN: Final = "arm-unknown-linux-gnueabi"
DEFAULT_APP_NAME: Final = "piano_visualizer"
DEFAULT_TARGET_FOLDER: Final = "/home/pi"
DEFAULT_DEBUG_PORT: Final = "17777"
DEFAULT_GDB_APP = "gdb-multiarch"


def main():
    bld_deploy_run(parse_arguments())


def parse_arguments():
    desc = (
        "Rust Remote Deployment Helper."
        "Builds the image and can optionally transfer and run "
        "it on the target system as well."
    )
    parser = argparse.ArgumentParser(description=desc)
    parser.add_argument(
        "--args",
        help="Additional arguments to pass to the built binary",
    )
    parser.add_argument(
        "-u",
        "--user",
        default=f"{DEFAULT_USER_NAME}",
        help=f"Username for ssh access. Default: {DEFAULT_USER_NAME}",
    )
    parser.add_argument(
        "-P",
        "--password",
        help="Password for ssh access",
    )
    parser.add_argument(
        "-a",
        "--address",
        default=f"{DEFAULT_ADDRESS}",
        help=f"Remote SSH address. Default: {DEFAULT_ADDRESS}",
    )
    parser.add_argument(
        "--tc",
        default=f"{DEFAULT_TOOLCHAIN}",
        help=f"Target toolchain. Default: {DEFAULT_TOOLCHAIN}",
    )

    parser.add_argument(
        "--app",
        default=f"{DEFAULT_APP_NAME}",
        help=f"Target appname. Default: {DEFAULT_APP_NAME}",
    )
    parser.add_argument(
        "--source",
        help=f"Target destination path. Default: Built from other arguments",
    )
    parser.add_argument(
        "--dest",
        default=f"{DEFAULT_TARGET_FOLDER}",
        help=f"Target destination path. Default: {DEFAULT_TARGET_FOLDER}",
    )
    parser.add_argument(
        "other", nargs=argparse.REMAINDER, help="Argument forwarded to cargo build"
    )

    parser.add_argument(
        "-b",
        "--build",
        action="store_true",
        help="Build application",
    )
    parser.add_argument(
        "-t",
        "--transfer",
        action="store_true",
        help="Transfer application to remote machine",
    )
    parser.add_argument(
        "-r",
        "--run",
        action="store_true",
        help="Run application on remote machine",
    )
    parser.add_argument(
        "-d",
        "--debug",
        action="store_true",
        help="Run gdbserver on remote machine for remote debugging"
    )
    parser.add_argument(
        "-s",
        "--start",
        action="store_true",
        help="Start local GDB session, connecting to the remote GDB server"
    )
    parser.add_argument(
        "--gdb",
        default="gdb-multiarch",
        help="GDB application to use"
    )
    parser.add_argument(
        "-p",
        "--port",
        default=f"{DEFAULT_DEBUG_PORT}",
        help="Port to use for remote debugging"
    )
    parser.add_argument(
        "--additional-files",
        action="append",
        help="Additional files to transfer to remote machine, space separated"
    )
    parser.add_argument(
        "--sudo",
        action="store_true",
        help="Use sudo to run commands on remote machine"
    )
    parser.add_argument(
        "-e",
        "--sshenv",
        action="store_true",
        help="Take password from environmental variable",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Supply --release to build command",
    )
    parser.add_argument(
        "-f",
        "--sshfile",
        help="SSH key file. Otherwise, use password from environmental variable SSHPASS",
    )
    return parser.parse_args()


def bld_deploy_run(args):
    cargo_opts = ""
    build_folder = "debug"
    if args.release:
        cargo_opts += "--release"
        build_folder = "release"
    for other in args.other:
        cargo_opts += f"{other}"
    sshpass_args = ""
    if args.password:
        sshpass_args += f"-p {args.password}"
    elif args.sshfile:
        sshpass_args = f"-f {args.sshfile}"
    elif args.sshenv:
        sshpass_args = "-e"
    ssh_target_ident = f"{args.user}@{args.address}"
    sshpass_cmd = ""
    if platform.system() != "Windows":
        sshpass_cmd = f"sshpass {sshpass_args}"
    if not args.source:
        source_path = f"{os.getcwd()}/target/{args.tc}/{build_folder}/{args.app}"
    else:
        source_path = args.source
    build_cmd = f"cargo build {cargo_opts}"
    if args.build:
        print(f"Running build command: {build_cmd}")
        os.system(build_cmd)
    if args.transfer:
        if not os.path.exists(source_path):
            print(f"No application found at {source_path}")
            sys.exit(1)
        scp_target_dest = f'{ssh_target_ident}:"{args.dest}"'
        transfer_cmd = f"{sshpass_cmd} ssh {ssh_target_ident} \"sudo killall -q {args.app}\"; {sshpass_cmd} scp {source_path} {args.additional_files and ' '.join(args.additional_files)} {scp_target_dest}"
        print(f"Running transfer command: {transfer_cmd}")
        os.system(transfer_cmd)
    if args.run:
        run_cmd = f"{sshpass_cmd} ssh {ssh_target_ident} \"{args.sudo and 'sudo'} {args.dest}/{args.app} {args.args and args.args}\""
        print(f"Running target application: {run_cmd}")
        os.system(run_cmd)
    elif args.debug:
        # Kill all running gdbserver applications  first
        # Then start the GDB server
        debug_shell_cmd = f"'sudo killall -q gdbserver; gdbserver *:{args.port} \"{args.sudo and 'sudo'} {args.dest}/{args.app}\"'"
        # Execute the command above and also set up port forwarding. This allows to connect
        # to localhost:17777 on the local development machine
        debug_cmd = f"{sshpass_cmd} ssh -f -L {args.port}:localhost:{args.port} {ssh_target_ident} {debug_shell_cmd}"
        print(f"Running debug command: {debug_cmd}")
        os.system(debug_cmd)
        if args.start:
            time.sleep(0.2)
            start_cmd = f"{args.gdb} -q -x gdb.gdb {source_path}"
            print(f"Running start command: {start_cmd}")
            os.system(start_cmd)


if __name__ == "__main__":
    main()
