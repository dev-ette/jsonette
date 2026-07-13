import os
import sys
import time
import subprocess
import json

BUDGETS = {
    "binary_size_mb": 15.0,
    "gen_50mb_sec": 10.0,
    "fmt_50mb_sec": 5.0,
    "query_50mb_sec": 2.0,
    "explore_50mb_sec": 2.0,
}

def run_cmd(cmd):
    start = time.time()
    res = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    elapsed = time.time() - start
    if res.returncode != 0:
        print(f"Command failed: {cmd}\n{res.stderr}")
        sys.exit(1)
    return elapsed, res.stdout

def main():
    print("Building CLI...")
    run_cmd("cargo build --release -p jsonette")
    
    bin_path = "./target/release/jsonette"
    bin_size_mb = os.path.getsize(bin_path) / (1024 * 1024)
    print(f"Binary size: {bin_size_mb:.2f} MB")
    
    sizes = [("1MB", 1000000), ("10MB", 10000000), ("50MB", 50000000)]
    
    results = []
    
    for name, size in sizes:
        out_file = f"{name.lower()}.json"
        
        print(f"[{name}] Generating...")
        gen_time, _ = run_cmd(f"{bin_path} generate -c .github/perf_schema.json -s {size} -o {out_file}")
        
        print(f"[{name}] Formatting...")
        fmt_time, _ = run_cmd(f"{bin_path} format {out_file} -o /dev/null")
        
        print(f"[{name}] Query Length...")
        _, out_str = run_cmd(f"{bin_path} explore -n 1 '$' {out_file}")
        count = 100
        for line in out_str.split('\n'):
            if "Length:" in line:
                try: count = int(line.split()[1])
                except: pass
            
        mid = count // 2
        last = count - 1
        
        print(f"[{name}] Query First...")
        q_first, _ = run_cmd(f"{bin_path} query '$[0].id' {out_file} > /dev/null")
        print(f"[{name}] Query Mid...")
        q_mid, _ = run_cmd(f"{bin_path} query '$[{mid}].id' {out_file} > /dev/null")
        print(f"[{name}] Query Last...")
        q_last, _ = run_cmd(f"{bin_path} query '$[{last}].id' {out_file} > /dev/null")
        
        print(f"[{name}] Explore Root...")
        e_root, _ = run_cmd(f"{bin_path} explore -n 1 '$' {out_file} > /dev/null")
        print(f"[{name}] Explore First...")
        e_first, _ = run_cmd(f"{bin_path} explore -n 1 '$[0].friends' {out_file} > /dev/null")
        print(f"[{name}] Explore Mid...")
        e_mid, _ = run_cmd(f"{bin_path} explore -n 1 '$[{mid}].friends' {out_file} > /dev/null")
        print(f"[{name}] Explore Last...")
        e_last, _ = run_cmd(f"{bin_path} explore -n 1 '$[{last}].friends' {out_file} > /dev/null")
        
        results.append({
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
        })
        
    # Generate Markdown Report
    report = ["# Performance Metrics", "", f"**Binary Size:** {bin_size_mb:.2f} MB (Budget: {BUDGETS['binary_size_mb']} MB)", ""]
    
    report.append("| Workload | Gen (s) | Fmt (s) | Q First | Q Mid | Q Last | Exp Root | Exp First | Exp Mid | Exp Last |")
    report.append("|---|---|---|---|---|---|---|---|---|---|")
    
    for r in results:
        report.append(f"| {r['name']} | {r['gen']:.3f} | {r['fmt']:.3f} | {r['q_first']:.3f} | {r['q_mid']:.3f} | {r['q_last']:.3f} | {r['e_root']:.3f} | {r['e_first']:.3f} | {r['e_mid']:.3f} | {r['e_last']:.3f} |")
        
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

if __name__ == '__main__':
    main()
