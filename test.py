# This is the test runner of Optic
# 
# This script compiles the code samples and checks the output

from pathlib import Path
import filecmp
import difflib
import subprocess
import sys

BASE_DIR = Path("./tests")
SAMPLE_DIR = BASE_DIR / "samples"
EXPECTED_DIR = BASE_DIR / "expected"
OUTPUT_FILE = BASE_DIR / "out.txt"
LOG_FILE = Path("failures.log")
OPTIC_PATH = Path("./target/release/optic")

LOG_FILE.unlink(missing_ok=True)

def log_failure(sample: str, target: str, details: str):
    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(f"==== FAIL: {sample} ({target}) ====\n")
        f.write(details)
        f.write("\n" + "="*40 + "\n\n")

def test_file(sample: Path, target: str, rewrite: bool):
    expected = Path(f"{target}/{sample.stem}.txt")

    result = subprocess.run([str(OPTIC_PATH), sample.as_posix(), "-o", str(OUTPUT_FILE)]) 

    if result.returncode != 0:
        print(f"    [fail] {sample.name} : compilation error")
        return False
    
    if rewrite:
        expected.write_text(OUTPUT_FILE.read_text())
        print(f"    [rewrite] {sample.name}")
        return True
    
    if OUTPUT_FILE.read_text() != expected.read_text():
        print(f"    [fail] {sample.name} : different output than expected")

        diff = "".join(difflib.unified_diff(
            expected.read_text().splitlines(keepends=True),
            OUTPUT_FILE.read_text().splitlines(keepends=True),
            fromfile="Expected", tofile="Actual"
        ))

        log_failure(sample.name, target, diff)
        return False

    print(f"    [pass] {sample.name}")
    return True

if __name__ == "__main__":
    tests = SAMPLE_DIR.glob("*.opt")
    targets = EXPECTED_DIR.glob("*")
    rewrite = False

    if len(sys.argv) > 1:
        if sys.argv[1] == "--rewrite":
            rewrite = True

    all_passed = False
    for target in targets:
        print(f"target {target.stem}:")
        all_passed = all(test_file(t, target, rewrite) for t in tests) 

    OUTPUT_FILE.unlink(missing_ok=True)

    sys.exit(0 if all_passed else 1)       