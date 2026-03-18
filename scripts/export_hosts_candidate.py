import argparse
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

GITHUB_HOSTS_START = "#Github Hosts Start"
GITHUB_HOSTS_END = "#Github Hosts End"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export a backup of hosts and an updated candidate file without modifying system hosts."
    )
    parser.add_argument("--hosts-path", required=True)
    parser.add_argument("--block-file", required=True)
    parser.add_argument("--output-dir", required=True)
    return parser.parse_args()


def summarize_content(content: str) -> dict[str, object]:
    first_line = next((line for line in content.splitlines() if line.strip()), "")[:80]
    return {
        "len": len(content),
        "lines": len(content.splitlines()) if content else 0,
        "github_start": content.count(GITHUB_HOSTS_START),
        "github_end": content.count(GITHUB_HOSTS_END),
        "sha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
        "first_line": first_line,
    }


def normalize_block(block: str) -> str:
    normalized = block.replace("\r\n", "\n")
    if not normalized.endswith("\n"):
        normalized += "\n"
    return normalized


def replace_or_append_github_block(local_text: str, remote_block: str) -> tuple[str, str]:
    lines = local_text.splitlines()
    start = next((i for i, line in enumerate(lines) if line.strip() == GITHUB_HOSTS_START), None)
    end = next((i for i, line in enumerate(lines) if line.strip() == GITHUB_HOSTS_END), None)

    if start is not None and end is not None and end >= start:
        before = "\n".join(lines[:start])
        after = "\n".join(lines[end + 1 :]) if end + 1 < len(lines) else ""
        out = ""
        if before:
            out += before + "\n"
        out += remote_block
        if after:
            out += after + "\n"
        return out, "replace"

    out = local_text
    if out and not out.endswith("\n"):
        out += "\n"
    if out:
        out += "\n"
    out += remote_block
    return out, "append"


def timestamp() -> str:
    return datetime.now().strftime("%Y%m%d-%H%M%S")


def main() -> int:
    args = parse_args()
    hosts_path = Path(args.hosts_path)
    block_path = Path(args.block_file)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    original_text = hosts_path.read_text(encoding="utf-8")
    remote_block = normalize_block(block_path.read_text(encoding="utf-8"))
    updated_text, mode = replace_or_append_github_block(original_text, remote_block)

    stamp = timestamp()
    backup_path = output_dir / f"hosts.backup-{stamp}.txt"
    updated_path = output_dir / f"hosts.updated-{stamp}.txt"

    backup_path.write_text(original_text, encoding="utf-8")
    updated_path.write_text(updated_text, encoding="utf-8")

    payload = {
        "status": "ok",
        "mode": mode,
        "hosts_path": str(hosts_path),
        "block_file": str(block_path),
        "output_dir": str(output_dir),
        "backup_path": str(backup_path),
        "updated_path": str(updated_path),
        "original": summarize_content(original_text),
        "updated": summarize_content(updated_text),
    }
    json.dump(payload, sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
