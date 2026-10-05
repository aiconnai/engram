#!/usr/bin/env python3
"""Fake writer for the Engram sandbox adapter (task H3). Stdlib only; runs INSIDE the container.

This is NOT an agent. It is a deliberately hostile stand-in that tries, on request, the things a
compromised writer would try, and reports each attempt as one JSON line on stdout so the
supervisor-side tests can assert that every escape was refused or contained:

  benign                      control: write inside /work and /tmp (must succeed)
  write_outside               write outside the worktree, through symlinks, /proc, /sys
  privilege                   setuid(0), mknod, mount, capability and no-new-privileges state
  tamper_tcb                  modify / delete / rename / add files under the read-only /tcb mount
  network [host]              TCP, UDP, DNS and interface enumeration
  read_host_secrets CANARY P  look for credential env vars, the canary value and host paths P...
  forkbomb [limit]            fork until the pids cap (or `limit`) stops it, then clean up
  daemon_child                leave a heartbeat child behind and exit (must not survive the run)
  outlive_timeout             sleep far past any timeout
  tamper_run_dir HOSTPATH     forge evidence in /work and write to the host run_dir path
  memory_bomb MB              allocate until the memory cap kills the process
  exit_code N                 exit with status N
  spam_output BYTES           write BYTES of output (log cap test)
  write_file REL TEXT         (runner, H4) write TEXT to /work/REL, creating parent directories
  delete_file REL             (runner, H4) delete /work/REL
  claim_pass                  (runner, H4) forge evidence / receipt / outcome files in /work and print PASS lines

Each result line: {"attempt": name, "outcome": "refused" | "succeeded" | "info", ...}.
"""

from __future__ import annotations

import errno
import json
import os
import signal
import socket
import sys
import time

CREDENTIAL_ENV = (
    "SSH_AUTH_SOCK", "SSH_ASKPASS", "GIT_ASKPASS", "GITHUB_TOKEN", "GH_TOKEN", "NPM_TOKEN",
    "AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN", "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY", "GOOGLE_APPLICATION_CREDENTIALS", "KUBECONFIG", "DOCKER_HOST", "DOCKER_CONFIG",
)


def emit(**fields):
    print(json.dumps(fields, sort_keys=True), flush=True)


def attempt(name, fn):
    """Run fn; an OSError means the environment refused it, anything else returned means it worked."""
    try:
        detail = fn()
    except OSError as exc:
        emit(attempt=name, outcome="refused", errno=errno.errorcode.get(exc.errno or 0, str(exc.errno)))
    else:
        emit(attempt=name, outcome="succeeded", detail=detail)


def write_file(path, data="x"):
    def _do():
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(data)
        return path
    return _do


def b_benign(_args):
    attempt("write_worktree", write_file("/work/benign.txt", "ok"))
    attempt("write_tmp", write_file("/tmp/benign.txt", "ok"))
    emit(attempt="identity", outcome="info", uid=os.getuid(), gid=os.getgid(), cwd=os.getcwd())


def b_write_outside(_args):
    attempt("write_worktree_control", write_file("/work/inside.txt"))
    for path in ("/etc/engram-escape", "/usr/local/engram-escape", "/engram-escape", "/work/../engram-escape",
                 "/var/engram-escape", "/var/tmp/engram-escape", "/root/engram-escape", "/proc/sys/kernel/hostname", "/sys/kernel/engram"):
        attempt(f"write:{path}", write_file(path))
    # A symlink planted inside the writable worktree must not become a path out of it.
    def via_symlink():
        link = "/work/escape-link"
        if os.path.lexists(link):
            os.unlink(link)
        os.symlink("/etc", link)
        with open(link + "/engram-escape", "w", encoding="utf-8") as handle:
            handle.write("x")
    attempt("write_via_symlink_to_etc", via_symlink)
    attempt("chmod_system_file", lambda: os.chmod("/etc/passwd", 0o777))
    attempt("unlink_system_file", lambda: os.unlink("/etc/hostname"))


def b_privilege(_args):
    emit(attempt="identity", outcome="info", uid=os.getuid(), euid=os.geteuid())
    status = {}
    with open("/proc/self/status", encoding="utf-8") as handle:
        for line in handle:
            key, _, value = line.partition(":")
            if key in ("CapEff", "CapPrm", "CapBnd", "NoNewPrivs", "Seccomp"):
                status[key] = value.strip()
    emit(attempt="proc_status", outcome="info", **status)
    attempt("setuid_0", lambda: os.setuid(0))
    attempt("mknod_dev", lambda: os.mknod("/tmp/engram-null", 0o600 | 0o020000, os.makedev(1, 3)))
    attempt("chroot", lambda: os.chroot("/tmp"))

    def mount():
        import ctypes
        libc = ctypes.CDLL(None, use_errno=True)
        if libc.mount(b"tmpfs", b"/tmp", b"tmpfs", 0, None) != 0:
            err = ctypes.get_errno()
            raise OSError(err, os.strerror(err))
    attempt("mount_tmpfs", mount)
    attempt("exec_from_tmpfs", lambda: os.execv("/tmp/engram-exec", ["/tmp/engram-exec"]))


def b_tamper_tcb(_args):
    attempt("read_manifest_control", lambda: open("/tcb/manifest.json", encoding="utf-8").read()[:20])
    attempt("write_manifest", write_file("/tcb/manifest.json", "{}"))
    attempt("append_registry", lambda: open("/tcb/registry.json", "a", encoding="utf-8").write("x"))
    attempt("unlink_manifest", lambda: os.unlink("/tcb/manifest.json"))
    attempt("rename_manifest", lambda: os.rename("/tcb/manifest.json", "/tcb/manifest.old"))
    attempt("chmod_manifest", lambda: os.chmod("/tcb/manifest.json", 0o777))
    attempt("create_in_tcb", write_file("/tcb/engram-new-file"))
    attempt("write_tool", write_file("/tcb/tools/fake_writer.py", "pass"))
    attempt("truncate_manifest", lambda: os.truncate("/tcb/manifest.json", 0))


def b_network(args):
    target = args[0] if args else "1.1.1.1"

    def tcp():
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(3)
        try:
            sock.connect((target, 443))
        finally:
            sock.close()

    def udp():
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(3)
        try:
            sock.sendto(b"\x00" * 12, ("8.8.8.8", 53))
            sock.recvfrom(512)
        finally:
            sock.close()

    attempt("tcp_connect_external", tcp)
    attempt("udp_dns_external", udp)
    attempt("resolve_hostname", lambda: socket.getaddrinfo("example.com", 443))
    attempt("connect_docker_socket_tcp", lambda: socket.create_connection(("host.docker.internal", 2375), timeout=3))
    emit(attempt="interfaces", outcome="info", interfaces=sorted(os.listdir("/sys/class/net")))
    emit(attempt="docker_sock_present", outcome="info", present=os.path.exists("/var/run/docker.sock"))


def b_read_host_secrets(args):
    canary, paths = (args[0] if args else ""), args[1:]
    present = sorted(name for name in CREDENTIAL_ENV if name in os.environ)
    emit(attempt="credential_env", outcome="info", present=present)
    emit(attempt="canary_in_env", outcome="info",
         leaked=bool(canary) and any(canary in value for value in os.environ.values()))
    emit(attempt="env_names", outcome="info", names=sorted(os.environ))
    emit(attempt="home", outcome="info", home=os.environ.get("HOME"))
    for path in paths:
        emit(attempt=f"host_path_visible:{path}", outcome="info", visible=os.path.exists(path))
    for path in (os.path.expanduser("~/.ssh"), "/root/.ssh", "/var/run/docker.sock", "/run/secrets"):
        emit(attempt=f"path_visible:{path}", outcome="info", visible=os.path.exists(path))


def b_forkbomb(args):
    limit = int(args[0]) if args else 400
    children, error = [], None
    for _ in range(limit):
        try:
            pid = os.fork()
        except OSError as exc:
            error = errno.errorcode.get(exc.errno or 0, str(exc.errno))
            break
        if pid == 0:
            time.sleep(60)
            os._exit(0)
        children.append(pid)
    emit(attempt="fork_until_refused", outcome="refused" if error else "succeeded",
         forked=len(children), errno=error)
    for pid in children:
        try:
            os.kill(pid, signal.SIGKILL)
        except OSError:
            pass
    for pid in children:
        try:
            os.waitpid(pid, 0)
        except OSError:
            pass


def b_daemon_child(_args):
    if os.fork() == 0:
        os.setsid()
        if os.fork() == 0:
            while True:
                with open("/work/heartbeat", "w", encoding="utf-8") as handle:
                    handle.write(str(time.time()))
                time.sleep(0.1)
        os._exit(0)
    time.sleep(0.5)
    emit(attempt="daemon_child_started", outcome="info")


def b_outlive_timeout(_args):
    emit(attempt="sleeping", outcome="info")
    time.sleep(600)


def b_tamper_run_dir(args):
    host_run_dir = args[0]
    forged = json.dumps({"status": "passed", "exit_code": 0, "forged": True})
    attempt("forge_outcome_in_worktree", write_file("/work/outcome.json", forged))
    attempt("write_host_run_dir_outcome", write_file(os.path.join(host_run_dir, "outcome.json"), forged))
    attempt("write_host_run_dir_log", write_file(os.path.join(host_run_dir, "logs", "00-forged.stdout.log"), forged))
    attempt("mkdir_host_run_dir", lambda: os.makedirs(host_run_dir, exist_ok=True))
    emit(attempt="host_run_dir_visible", outcome="info", visible=os.path.exists(host_run_dir))


def b_memory_bomb(args):
    target_mb, held = int(args[0]), []
    for _ in range(target_mb):
        held.append(bytearray(1024 * 1024))
        for index in range(0, len(held[-1]), 4096):
            held[-1][index] = 1
    emit(attempt="memory_bomb", outcome="succeeded", mb=len(held))


def b_exit_code(args):
    sys.exit(int(args[0]))


def b_spam_output(args):
    remaining, line = int(args[0]), "x" * 1023 + "\n"
    while remaining > 0:
        sys.stdout.write(line)
        remaining -= len(line)
    sys.stdout.flush()


def _work_path(rel):
    path = os.path.normpath(os.path.join("/work", rel))
    if not path.startswith("/work/"):
        raise SystemExit(f"refusing path outside /work: {rel!r}")
    return path


def b_write_file(args):
    path = _work_path(args[0])
    os.makedirs(os.path.dirname(path), exist_ok=True)
    attempt("write_file", write_file(path, args[1]))


def b_delete_file(args):
    attempt("delete_file", lambda: os.unlink(_work_path(args[0])))


def b_claim_pass(_args):
    forged = json.dumps({"verdict": "pass", "status": "passed", "forged": True})
    for name in ("evidence.json", "receipt.json", "outcome.json"):
        attempt(f"forge:{name}", write_file(os.path.join("/work", name), forged))
    print("GATE_STATUS: PASS", flush=True)
    print("RUN_STATUS: PASSED", flush=True)


BEHAVIORS = {
    "benign": b_benign, "write_outside": b_write_outside, "privilege": b_privilege, "tamper_tcb": b_tamper_tcb,
    "network": b_network, "read_host_secrets": b_read_host_secrets, "forkbomb": b_forkbomb,
    "daemon_child": b_daemon_child, "outlive_timeout": b_outlive_timeout, "tamper_run_dir": b_tamper_run_dir,
    "memory_bomb": b_memory_bomb, "exit_code": b_exit_code, "spam_output": b_spam_output,
    "write_file": b_write_file, "delete_file": b_delete_file, "claim_pass": b_claim_pass,
}


def main(argv):
    if len(argv) < 2 or argv[1] not in BEHAVIORS:
        print("usage: fake_writer.py <" + "|".join(sorted(BEHAVIORS)) + "> [args...]", file=sys.stderr)
        return 2
    BEHAVIORS[argv[1]](argv[2:])
    emit(attempt="done", outcome="info", behavior=argv[1])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
