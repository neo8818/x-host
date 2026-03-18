import json
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "hosts_block_probe.py"

OLD_BLOCK = textwrap.dedent(
    """\
    #Github Hosts Start
    1.1.1.1 old.example.com
    #Github Hosts End
    """
)

NEW_BLOCK = textwrap.dedent(
    """\
    #Github Hosts Start
    2.2.2.2 new.example.com
    #Github Hosts End
    """
)


class HostsBlockProbeCliTests(unittest.TestCase):
    def test_replaces_existing_github_block_and_reports_observations(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            hosts_path = tmp_path / "hosts"
            block_path = tmp_path / "block.txt"

            hosts_path.write_text(
                textwrap.dedent(
                    f"""\
                    127.0.0.1 localhost
                    {OLD_BLOCK}
                    10.0.0.1 internal.example.com
                    """
                ),
                encoding="utf-8",
            )
            block_path.write_text(NEW_BLOCK, encoding="utf-8")

            completed = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--hosts-path",
                    str(hosts_path),
                    "--block-file",
                    str(block_path),
                    "--observe-count",
                    "3",
                    "--observe-interval-ms",
                    "1",
                ],
                capture_output=True,
                text=True,
                check=False,
            )

            self.assertEqual(
                completed.returncode,
                0,
                msg=f"stdout={completed.stdout}\nstderr={completed.stderr}",
            )

            payload = json.loads(completed.stdout)
            self.assertEqual(payload["status"], "ok")
            self.assertEqual(payload["mode"], "replace")
            self.assertEqual(payload["observations_count"], 3)
            self.assertEqual(len(payload["observations"]), 3)
            self.assertTrue(all(item["github_start"] == 1 for item in payload["observations"]))
            self.assertTrue(all(item["github_end"] == 1 for item in payload["observations"]))

            content = hosts_path.read_text(encoding="utf-8")
            self.assertIn("2.2.2.2 new.example.com", content)
            self.assertNotIn("1.1.1.1 old.example.com", content)
            self.assertEqual(content.count("#Github Hosts Start"), 1)
            self.assertEqual(content.count("#Github Hosts End"), 1)


if __name__ == "__main__":
    unittest.main()
