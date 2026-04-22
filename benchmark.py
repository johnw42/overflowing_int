import os
import argparse
import shlex
import shutil
import re
import json

TOP_DIR = os.path.dirname(__file__)

REVISIONS = ["xzw", "out", "msu", "tns", "xmr", "oot", "kmo", "puk"]

DEFAULT_BASELINE = REVISIONS[0]

FUNCTIONS = ["Control", "Cow", "Arc", "ArcSize", "Identity", "Enum"]
SIZES = ["10", "15", "20", "30", "40", "50", "100"]


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

        for encoding in FUNCTIONS:
            for size in SIZES:
                with open(
                    f"../target/criterion/Pi/{encoding}/{size}/{revision}/estimates.json"
                ) as f:
                    data = json.load(f)
                    throughput = 1000 * int(size) / data["mean"]["point_estimate"]
                    if throughput:
                        self.data[(encoding, size)] = float(throughput)
                    else:
                        self.data[(encoding, size)] = None


def system(cmd):
    print(f"$ {cmd}")
    if os.system(cmd) != 0:
        raise RuntimeError(f"Command failed: {cmd}")


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
        action="append",
        default=[],
        help="Group(s) to filter which benchmarks to run.",
    )
    p.add_argument(
        "--encoding",
        "-e",
        action="append",
        default=[],
        help="Encoding(s) to filter which benchmarks to run.",
    )
    p.add_argument(
        "--size",
        "-s",
        action="append",
        default=[],
        help="Size(s) to filter which benchmarks to run.",
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
    print(opts)

    bench_pattern = shlex.quote(
        f"({"|".join(opts.group) if opts.group else '[^/]+'})"
        + f"/({"|".join(opts.encoding) if opts.encoding else '[^/]+'})"
        + f"/({"|".join(opts.size) if opts.size else '[^/]+'})"
    )

    match opts.command:
        case "all":
            workspace_name = "bench-workspace"
            target_dir = f"{TOP_DIR}/../target"
            workspace_dir = f"{target_dir}/{workspace_name}"
            try:
                system(f"jj workspace add {workspace_dir}")
                os.chdir(workspace_dir)
                for rev in REVISIONS:
                    print(f"Benchmarking revision {rev}...")
                    system(f"jj edit --ignore-immutable {rev}")
                    cargo(
                        f"bench -p compact_bigint --bench={opts.bench} -- --save-baseline={rev} {bench_pattern}"
                    )
                system(
                    f"rsync -a {workspace_dir}/target/criterion/ {target_dir}/criterion/"
                )
            finally:
                os.chdir(TOP_DIR)
                system(f"jj workspace forget {workspace_name}")
                shutil.rmtree(workspace_dir, ignore_errors=True)
        case "summary":
            print("Printing summary of benchmark results...")
            rev_data = {rev: RevisionData(rev) for rev in REVISIONS}
            baseline_data = rev_data[opts.baseline]
            for encoding in opts.encoding or FUNCTIONS:
                for size in opts.size or SIZES:
                    print(f"{encoding}({size}):")
                    for rev in REVISIONS:
                        data = rev_data.get(rev)
                        if data is None:
                            continue
                        if (encoding, size) in data.data:
                            reference = baseline_data.data[(encoding, size)]
                            throughput = data.data[(encoding, size)]
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
