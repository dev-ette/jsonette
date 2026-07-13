import os
import sys
import time
import subprocess
from typing import Any
import platform
import re

BUDGETS = {
    "binary_size_mb": 15.0,
    "gen_50mb_sec": 10.0,
    "fmt_50mb_sec": 5.0,
    "query_50mb_sec": 2.0,
    "explore_50mb_sec": 2.0,
}


def run_cmd(cmd: str):
    is_mac = platform.system() == "Darwin"
    time_cmd = "/usr/bin/time -l " if is_mac else "/usr/bin/time -v "

    start = time.time()
    res = subprocess.run(time_cmd + cmd, shell=True, capture_output=True, text=True)
    elapsed = time.time() - start

    if res.returncode != 0:
        print(f"Command failed: {cmd}\n{res.stderr}")
        sys.exit(1)

    ram_mb = 0.0
    if is_mac:
        match = re.search(r"(\d+)\s+maximum resident set size", res.stderr)
        if match:
            ram_mb = int(match.group(1)) / (1024 * 1024)
    else:
        match = re.search(r"Maximum resident set size \(kbytes\):\s+(\d+)", res.stderr)
        if match:
            ram_mb = int(match.group(1)) / 1024

    return elapsed, ram_mb, res.stdout


def main():
    print("Building CLI...")
    run_cmd("cargo build --release -p jsonette")

    bin_path = "./target/release/jsonette"
    bin_size_mb = os.path.getsize(bin_path) / (1024 * 1024)
    print(f"Binary size: {bin_size_mb:.2f} MB")

    sizes = [
        ("1MB", 1000000),
        ("10MB", 10000000),
        ("50MB", 50000000),
    ]

    results: list[dict[str, Any]] = []

    for name, size in sizes:
        out_file = f"{name.lower()}.json"

        print(f"[{name}] Generating...")
        gen_time, gen_ram, _ = run_cmd(
            f"{bin_path} generate -c .github/perf_schema.json -s {size} -o {out_file}"
        )

        print(f"[{name}] Formatting...")
        fmt_time, fmt_ram, _ = run_cmd(f"{bin_path} format {out_file} -o /dev/null")

        print(f"[{name}] Query Length...")
        _, _, out_str = run_cmd(f"{bin_path} explore -n 1 '$' {out_file}")
        count = 100
        for line in out_str.split("\n"):
            if "Length:" in line:
                try:
                    count = int(line.split()[1])
                except Exception:
                    pass

        mid = count // 2
        last = count - 1

        print(f"[{name}] Query First...")
        q_first, q_first_ram, _ = run_cmd(
            f"{bin_path} query '$[0].id' {out_file} > /dev/null"
        )
        print(f"[{name}] Query Mid...")
        q_mid, q_mid_ram, _ = run_cmd(
            f"{bin_path} query '$[{mid}].id' {out_file} > /dev/null"
        )
        print(f"[{name}] Query Last...")
        q_last, q_last_ram, _ = run_cmd(
            f"{bin_path} query '$[{last}].id' {out_file} > /dev/null"
        )

        print(f"[{name}] Explore Root...")
        e_root, e_root_ram, _ = run_cmd(
            f"{bin_path} explore -n 1 '$' {out_file} > /dev/null"
        )
        print(f"[{name}] Explore First...")
        e_first, e_first_ram, _ = run_cmd(
            f"{bin_path} explore -n 1 '$[0].friends' {out_file} > /dev/null"
        )
        print(f"[{name}] Explore Mid...")
        e_mid, e_mid_ram, _ = run_cmd(
            f"{bin_path} explore -n 1 '$[{mid}].friends' {out_file} > /dev/null"
        )
        print(f"[{name}] Explore Last...")
        e_last, e_last_ram, _ = run_cmd(
            f"{bin_path} explore -n 1 '$[{last}].friends' {out_file} > /dev/null"
        )

        print(f"[{name}] Convert to YAML...")
        c_yaml, c_yaml_ram, _ = run_cmd(f"{bin_path} convert -f json -t yaml {out_file} > /dev/null || true")
        print(f"[{name}] Convert to TOML...")
        c_toml, c_toml_ram, _ = run_cmd(f"{bin_path} convert -f json -t toml {out_file} > /dev/null")
        print(f"[{name}] Convert to XML...")
        c_xml, c_xml_ram, _ = run_cmd(f"{bin_path} convert -f json -t xml {out_file} > /dev/null")

        results.append(
            {
                "name": name,
                "gen": gen_time,
                "fmt": fmt_time,
                "q_first": q_first,
                "q_mid": q_mid,
                "q_last": q_last,
                "e_root": e_root,
                "e_first": e_first,
                "e_mid": e_mid,
                "e_last": e_last,
                "c_yaml": c_yaml,
                "c_toml": c_toml,
                "c_xml": c_xml,
                "peak_ram": max(
                    gen_ram,
                    fmt_ram,
                    q_first_ram,
                    q_mid_ram,
                    q_last_ram,
                    e_root_ram,
                    e_first_ram,
                    e_mid_ram,
                    e_last_ram,
                    c_yaml_ram,
                    c_toml_ram,
                    c_xml_ram,
                ),
            }
        )

    # Generate Markdown Report
    report = [
        "# Performance Metrics",
        "",
        f"**Binary Size:** {bin_size_mb:.2f} MB (Budget: {BUDGETS['binary_size_mb']} MB)",
        "",
    ]

    report.append(
        "| Workload | Gen (s) | Fmt (s) | Q Mid | Exp Root | C TOML (s) | C XML (s) | Peak RAM (MB) |"
    )
    report.append("|---|---|---|---|---|---|---|---|")

    for r in results:
        report.append(
            f"| {r['name']} | {r['gen']:.3f} | {r['fmt']:.3f} | {r['q_mid']:.3f} | {r['e_root']:.3f} | {r['c_toml']:.3f} | {r['c_xml']:.3f} | {r['peak_ram']:.1f} |"
        )

    report_md = "\n".join(report)
    print("\n" + report_md + "\n")

    with open("perf_report.md", "w") as f:
        f.write(report_md)

    # Check Budgets
    failed = False
    if bin_size_mb > BUDGETS["binary_size_mb"]:
        print("FAIL: Binary size exceeded budget")
        failed = True

    r50 = results[-1]
    if r50["gen"] > BUDGETS["gen_50mb_sec"]:
        print("FAIL: 50MB Generation exceeded budget")
        failed = True
    if r50["fmt"] > BUDGETS["fmt_50mb_sec"]:
        print("FAIL: 50MB Formatting exceeded budget")
        failed = True
    if max(r50["q_first"], r50["q_mid"], r50["q_last"]) > BUDGETS["query_50mb_sec"]:
        print("FAIL: 50MB Query exceeded budget")
        failed = True

    if failed:
        sys.exit(1)
    else:
        print("All budgets passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()
