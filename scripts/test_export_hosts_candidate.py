import json
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "export_hosts_candidate.py"

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


class ExportHostsCandidateCliTests(unittest.TestCase):
    def test_exports_backup_and_updated_hosts_to_output_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            hosts_path = tmp_path / "hosts"
            block_path = tmp_path / "block.txt"
            output_dir = tmp_path / "exports"

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
                    "--output-dir",
                    str(output_dir),
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

            backup_path = Path(payload["backup_path"])
            updated_path = Path(payload["updated_path"])
            self.assertTrue(backup_path.exists())
            self.assertTrue(updated_path.exists())
            self.assertEqual(backup_path.parent, output_dir)
            self.assertEqual(updated_path.parent, output_dir)

            backup_content = backup_path.read_text(encoding="utf-8")
            updated_content = updated_path.read_text(encoding="utf-8")

            self.assertIn("1.1.1.1 old.example.com", backup_content)
            self.assertIn("2.2.2.2 new.example.com", updated_content)
            self.assertNotIn("1.1.1.1 old.example.com", updated_content)
            self.assertEqual(updated_content.count("#Github Hosts Start"), 1)
            self.assertEqual(updated_content.count("#Github Hosts End"), 1)
            self.assertEqual(hosts_path.read_text(encoding="utf-8"), backup_content)


if __name__ == "__main__":
    unittest.main()
