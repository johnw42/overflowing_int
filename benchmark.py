import os
import argparse
import shutil
import re
import json

TOP_DIR = os.path.dirname(__file__)

BASELINE = "lmu"

REVISIONS = [
    "lmu",
    "tyq",
    "uxw",
    "vuq",
    "nws",
    "nsy",
    "kpw",
    "tkr",
    "mwu",
]

FUNCTIONS = ["Control", "Cow", "Rc", "RcIsize", "Trivial", "Box"]
SIZES = [10, 15, 20, 30, 40, 50, 100]


def cargo(cmd):
    os.system(
        f"cargo --config target-dir='\"{TOP_DIR}/../target/bench-workspace\"' {cmd}"
    )


def current_revision():
    with os.popen("jj status") as f:
        m = re.search(r"Working copy  \(@\) : ([a-z]{8}) ", f.read())
        assert m
        return m.group(1)


class RevisionData:
    def __init__(self, revision):
        self.revision = revision
        self.data = {}

        for function in FUNCTIONS:
            for size in SIZES:
                name = f"Pi/{function}/{size}"
                with open(
                    f"../target/criterion/Pi/{function}/{size}/{revision}/estimates.json"
                ) as f:
                    data = json.load(f)
                    throughput = 1000 * size / data["mean"]["point_estimate"]
                    if throughput:
                        self.data[(function, size)] = float(throughput)
                    else:
                        self.data[(function, size)] = None


def main():
    os.chdir(TOP_DIR)

    p = argparse.ArgumentParser()
    s = p.add_subparsers(dest="command")
    s.add_parser("all", help="Run benchmarks for all revisions.")
    s.add_parser("summary", help="Print a summary of the benchmark results.")
    s.add_parser("run", help="Run benchmarks for the current revision.")
    p.add_argument(
        "--function",
        "-f",
        default=".*",
        help="Regex to filter which functions to benchmark.",
    )
    p.add_argument(
        "--size",
        "-s",
        default=".*",
        help="Regex to filter which input sizes to benchmark.",
    )

    opts = p.parse_args()

    match opts.command:
        case "all":
            try:
                os.system("jj workspace add target/bench-workspace")
                for rev in REVISIONS:
                    print(f"Testing revision {rev}...")
                    os.system(f"jj edit {rev}")
                    cargo(
                        f"bench -p compact_bigint --bench=pi -- --save-baseline={rev} 'Pi/({opts.function})/({opts.size})'"
                    )
            finally:
                os.chdir(TOP_DIR)
                os.system("jj workspace forget target/bench-workspace")
                shutil.rmtree("target/bench-workspace", ignore_errors=True)
        case "summary":
            print("Printing summary of benchmark results...")
            rev_data = {rev: RevisionData(rev) for rev in REVISIONS}
            baseline_data = rev_data[BASELINE]
            for function in FUNCTIONS:
                for size in SIZES:
                    print(f"{function}({size}):")
                    for rev in REVISIONS:
                        data = rev_data.get(rev)
                        if data is None:
                            continue
                        if (function, size) in data.data:
                            reference = baseline_data.data[(function, size)]
                            throughput = data.data[(function, size)]
                            percent = 100 * throughput / reference
                            if throughput is not None:
                                print(
                                    f"  {rev}: {throughput:.2f} ME/s ({percent:.2f}%)"
                                )
        case "run":
            os.system(
                f"cargo bench -p compact_bigint --bench=pi -- --save-baseline={current_revision()[:3]} 'Pi/({opts.function})/({opts.size})'"
            )


main()
