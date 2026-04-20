import os
import argparse
import shlex
import shutil
import re
import json

TOP_DIR = os.path.dirname(__file__)

DEFAULT_BASELINE = "uny"

REVISIONS = [
    "lmu",
    "tyq",
    "uxw",
    "vuq",
    "nws",
    "nsy",
    "kpw",
    "tkr",
    "ply",
    "pxr",
    "uny",
]

FUNCTIONS = ["Control", "Cow", "Arc", "ArcIsize", "Identity", "Enum"]
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


def system(cmd):
    print(f"$ {cmd}")
    return os.system(cmd)


def popen(cmd):
    print(f"$ {cmd}")
    return os.popen(cmd)


def main():
    os.chdir(TOP_DIR)

    p = argparse.ArgumentParser()
    s = p.add_subparsers(dest="command")
    s.add_parser("all", help="Run benchmarks for all revisions.")
    s.add_parser("summary", help="Print a summary of the benchmark results.")
    s.add_parser("profile", help="Profile the benchmarks.")
    s.add_parser("run", help="Run benchmarks for the current revision.")
    p.add_argument(
        "--group",
        "-g",
        help="Regex to filter which groups to benchmark.",
    )
    p.add_argument(
        "--encoding",
        "-e",
        help="Regex to filter which encodings to benchmark.",
    )
    p.add_argument(
        "--size",
        "-s",
        help="Regex to filter which input sizes to benchmark.",
    )
    p.add_argument(
        "--baseline",
        "-b",
        default=DEFAULT_BASELINE,
        help="Revision to use as baseline.",
    )
    p.add_argument(
        "--save-baseline",
        default=current_revision()[:3],
        help="Revision to save benchmark results under.",
    )
    p.add_argument(
        "--bench",
        "-B",
        default="pi",
        help="Name of the benchmark to run (default: 'pi').",
    )

    opts = p.parse_args()

    bench_pattern = shlex.quote(
        f"({opts.group or '[^/]+'})/({opts.encoding or '[^/]+'})/({opts.size or '[^/]+'})"
    )

    match opts.command:
        case "all":
            try:
                system("jj workspace add target/bench-workspace")
                for rev in REVISIONS:
                    print(f"Testing revision {rev}...")
                    system(f"jj edit --ignore-immutable {rev}")
                    cargo(
                        f"bench -p compact_bigint --bench={opts.bench} -- --save-baseline={rev} {bench_pattern}"
                    )
            finally:
                os.chdir(TOP_DIR)
                system("jj workspace forget target/bench-workspace")
                shutil.rmtree("target/bench-workspace", ignore_errors=True)
        case "summary":
            print("Printing summary of benchmark results...")
            rev_data = {rev: RevisionData(rev) for rev in REVISIONS}
            baseline_data = rev_data[opts.baseline]
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
                            fraction = throughput / reference
                            delta_percent = 100 * (fraction - 1)
                            if throughput is not None:
                                print(
                                    f"  {rev}: {throughput:.2f} ME/s ({delta_percent:+.2f}%)"
                                )
        case "profile":
            jq_output = os.popen(
                "cargo bench --no-run --message-format=json | jq -r 'select(.executable != null) | .executable'"
            ).read()
            m = re.search(
                rf"(?m)^(?:/\S+)*/target/release/deps/{opts.bench}-[0-9a-f]+$",
                jq_output,
            )
            print(repr(jq_output))
            assert m, jq_output
            executable = m.group(0)
            system(
                f"perf record --call-graph dwarf -- {executable} --bench --profile-time 5 {bench_pattern}"
            )
            data_path = f"perf.{current_revision()[:3]}.data"
            try:
                os.remove(data_path)
            except FileNotFoundError:
                pass
            os.rename("perf.data", data_path)
            print(f"Saved profile data to {data_path}")
        case "run" | None:
            system(
                f"cargo bench -p compact_bigint --bench={opts.bench} -- --save-baseline={opts.save_baseline} {bench_pattern}"
            )


main()
