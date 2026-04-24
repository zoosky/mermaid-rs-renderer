#!/usr/bin/env python3
"""Flowchart redesign visual/metric gate.

This script renders a curated set of hard flowchart fixtures, computes layout and
SVG quality metrics using the existing benchmark helpers, and writes an HTML
report with PNG snapshots. It can also compare the current working tree against a
baseline git ref by rendering the same fixtures from a temporary worktree.

Examples:

  # Current tree only
  python3 scripts/flowchart_redesign_gate.py --out-dir tmp/flowchart-redesign-gate

  # Before/after against the pushed arrowhead fix
  python3 scripts/flowchart_redesign_gate.py \
      --before-ref 87b45a4 \
      --out-dir tmp/flowchart-redesign-gate
"""

from __future__ import annotations

import argparse
import html
import importlib.util
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]

DEFAULT_FIXTURES = [
    "tests/fixtures/flowchart/bidirectional_labels.mmd",
    "benches/fixtures/flowchart_parallel_edges_bundle.mmd",
    "benches/fixtures/flowchart_parallel_label_stack.mmd",
    "benches/fixtures/flowchart_selfloop_bidi.mmd",
    "benches/fixtures/flowchart_label_collision.mmd",
    "benches/fixtures/flowchart_ports_heavy.mmd",
    "benches/fixtures/flowchart_medium.mmd",
    "benches/fixtures/flowchart_lanes_crossfeed.mmd",
    "benches/fixtures/flowchart_subgraph_boundary_intrusion.mmd",
]

SUMMARY_METRICS = [
    "score",
    "label_overlap_count",
    "label_overlap_area",
    "label_edge_overlap_count",
    "edge_label_owned_path_gap_bad_ratio",
    "edge_label_owned_anchor_offset_bad_ratio",
    "edge_crossings",
    "svg_edge_crossings",
    "edge_node_crossings",
    "edge_bends",
    "parallel_edge_overlap_pair_count",
    "parallel_edge_separation_bad_ratio",
    "port_target_side_mismatch_ratio",
    "port_direction_misalignment_ratio",
    "endpoint_off_boundary_ratio",
    "subgraph_boundary_intrusion_ratio",
    "flow_backtracking_edge_ratio",
]

STRICT_METRICS = {
    "label_overlap_count",
    "label_edge_overlap_count",
    "edge_crossings",
    "svg_edge_crossings",
    "edge_node_crossings",
    "parallel_edge_overlap_pair_count",
}


def run(cmd: list[str], *, cwd: Path = ROOT, check: bool = False) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(cmd, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if check and proc.returncode != 0:
        joined = " ".join(cmd)
        raise RuntimeError(f"command failed ({joined})\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}")
    return proc


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)  # type: ignore[call-arg]
    return module


def rel_key(path: Path) -> str:
    try:
        rel = path.resolve().relative_to(ROOT)
    except ValueError:
        rel = path
    return "__".join(rel.with_suffix("").parts).replace(" ", "_")


def ensure_png(svg_path: Path, png_path: Path, workdir: Path) -> None:
    converter = shutil.which("rsvg-convert")
    if converter:
        run([converter, str(svg_path), "-o", str(png_path)], cwd=workdir, check=True)
        return
    # Fallback: if rsvg-convert is unavailable, the caller still gets SVG/metrics.
    png_path.write_bytes(b"")


def render_case(workdir: Path, fixture_rel: str, out_dir: Path) -> dict[str, Any]:
    fixture = workdir / fixture_rel
    key = rel_key(ROOT / fixture_rel)
    svg_path = out_dir / f"{key}.svg"
    png_path = out_dir / f"{key}.png"
    layout_path = out_dir / f"{key}.layout.json"
    out_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--",
        "-i",
        str(fixture),
        "-o",
        str(svg_path),
        "-e",
        "svg",
        "--dumpLayout",
        str(layout_path),
    ]
    proc = run(cmd, cwd=workdir)
    if proc.returncode != 0:
        return {
            "fixture": fixture_rel,
            "key": key,
            "error": (proc.stderr or proc.stdout).strip()[:2000],
        }

    ensure_png(svg_path, png_path, workdir)
    return {
        "fixture": fixture_rel,
        "key": key,
        "svg": str(svg_path),
        "png": str(png_path),
        "layout": str(layout_path),
    }


def compute_case_metrics(case: dict[str, Any], layout_score, quality_bench) -> dict[str, Any]:
    if "error" in case:
        return {"error": case["error"]}
    layout_path = Path(case["layout"])
    svg_path = Path(case["svg"])
    try:
        data, nodes, edges = layout_score.load_layout(layout_path)
        metrics = layout_score.compute_metrics(data, nodes, edges)
        metrics["score"] = layout_score.weighted_score(metrics)
        _, _, svg_edges = quality_bench.load_mermaid_svg_graph(svg_path)
        metrics.update(quality_bench.compute_label_metrics(svg_path, nodes, svg_edges, "flowchart"))
        svg_metrics = quality_bench.compute_svg_edge_path_metrics(svg_edges)
        metrics.update(svg_metrics)
        metrics.update(quality_bench.compute_layout_anchor_metrics(data.get("edges", [])))
        return metrics
    except Exception as exc:  # pragma: no cover, diagnostic script
        return {"error": f"metric computation failed: {exc}"}


def render_suite(workdir: Path, fixtures: list[str], out_dir: Path) -> dict[str, dict[str, Any]]:
    layout_score = load_module("layout_score", ROOT / "scripts" / "layout_score.py")
    quality_bench = load_module("quality_bench", ROOT / "scripts" / "quality_bench.py")

    results: dict[str, dict[str, Any]] = {}
    for fixture in fixtures:
        print(f"rendering {fixture} in {workdir}", file=sys.stderr)
        case = render_case(workdir, fixture, out_dir)
        case["metrics"] = compute_case_metrics(case, layout_score, quality_bench)
        results[fixture] = case
    return results


def metric_delta(before: dict[str, Any] | None, after: dict[str, Any], metric: str) -> str:
    if before is None:
        value = after.get(metric)
        return "" if value is None else f"{value:.3f}" if isinstance(value, float) else str(value)
    base = before.get(metric)
    cur = after.get(metric)
    if base is None or cur is None:
        return ""
    if isinstance(base, (int, float)) and isinstance(cur, (int, float)):
        delta = cur - base
        sign = "+" if delta > 0 else ""
        return f"{base:.3f} -> {cur:.3f} ({sign}{delta:.3f})"
    return f"{base} -> {cur}"


def classify_metric(before: dict[str, Any] | None, after: dict[str, Any], metric: str) -> str:
    if before is None:
        return ""
    base = before.get(metric)
    cur = after.get(metric)
    if not isinstance(base, (int, float)) or not isinstance(cur, (int, float)):
        return ""
    if metric in STRICT_METRICS and cur > base:
        return "regress"
    if cur < base:
        return "improve"
    if cur > base:
        return "worse"
    return "same"


def write_report(
    out_dir: Path,
    fixtures: list[str],
    current: dict[str, dict[str, Any]],
    before: dict[str, dict[str, Any]] | None,
) -> Path:
    rows = []
    for fixture in fixtures:
        after_case = current[fixture]
        before_case = before.get(fixture) if before else None
        after_metrics = after_case.get("metrics", {})
        before_metrics = before_case.get("metrics", {}) if before_case else None
        metric_cells = []
        for metric in SUMMARY_METRICS:
            css = classify_metric(before_metrics, after_metrics, metric)
            metric_cells.append(
                f'<tr><td><code>{html.escape(metric)}</code></td><td class="{css}">{html.escape(metric_delta(before_metrics, after_metrics, metric))}</td></tr>'
            )
        before_img = ""
        if before_case and "png" in before_case and Path(before_case["png"]).stat().st_size > 0:
            before_img = f'<div class="panel"><h4>Before</h4><img src="{Path(before_case["png"]).relative_to(out_dir)}" /></div>'
        elif before_case and "error" in before_case:
            before_img = f'<div class="panel error"><h4>Before error</h4><pre>{html.escape(before_case["error"])}</pre></div>'

        if "png" in after_case and Path(after_case["png"]).stat().st_size > 0:
            after_img = f'<div class="panel"><h4>Current</h4><img src="{Path(after_case["png"]).relative_to(out_dir)}" /></div>'
        else:
            err = after_case.get("error", after_metrics.get("error", "no PNG"))
            after_img = f'<div class="panel error"><h4>Current error</h4><pre>{html.escape(str(err))}</pre></div>'

        rows.append(
            f"""
            <section class="case">
              <h2>{html.escape(fixture)}</h2>
              <div class="images">{before_img}{after_img}</div>
              <details open><summary>Metrics</summary><table>{''.join(metric_cells)}</table></details>
            </section>
            """
        )

    payload = {
        "fixtures": fixtures,
        "current": current,
        "before": before,
    }
    (out_dir / "metrics.json").write_text(json.dumps(payload, indent=2), encoding="utf-8")

    html_text = f"""
<!doctype html>
<meta charset="utf-8" />
<title>Flowchart redesign gate</title>
<style>
body {{ font-family: system-ui, -apple-system, sans-serif; margin: 24px; color: #222; }}
.case {{ margin: 32px 0; border-top: 1px solid #ddd; padding-top: 20px; }}
.images {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(360px, 1fr)); gap: 16px; align-items: start; }}
.panel {{ border: 1px solid #ddd; padding: 10px; background: #fafafa; overflow-x: auto; }}
.panel img {{ max-width: 100%; height: auto; background: white; }}
.error {{ background: #fff4f4; }}
pre {{ white-space: pre-wrap; }}
table {{ border-collapse: collapse; margin-top: 8px; }}
td {{ border: 1px solid #ddd; padding: 4px 8px; }}
.improve {{ color: #087f23; font-weight: 600; }}
.regress {{ color: #b00020; font-weight: 700; }}
.worse {{ color: #9a6700; }}
.same {{ color: #666; }}
code {{ background: #f2f2f2; padding: 1px 4px; border-radius: 3px; }}
</style>
<h1>Flowchart redesign gate</h1>
<p>Curated visual and metric gate for the flowchart layout redesign.</p>
<p>Raw data: <code>metrics.json</code></p>
{''.join(rows)}
"""
    report = out_dir / "report.html"
    report.write_text(html_text, encoding="utf-8")
    return report


def create_worktree(ref: str) -> Path:
    tmp = Path(tempfile.mkdtemp(prefix="mmdr-flowchart-gate-"))
    worktree = tmp / "repo"
    run(["git", "worktree", "add", "--detach", str(worktree), ref], cwd=ROOT, check=True)
    return worktree


def remove_worktree(worktree: Path) -> None:
    run(["git", "worktree", "remove", "--force", str(worktree)], cwd=ROOT)
    shutil.rmtree(worktree.parent, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Render curated flowchart redesign gate report")
    parser.add_argument("--out-dir", default=str(ROOT / "tmp" / "flowchart-redesign-gate"))
    parser.add_argument("--before-ref", help="optional git ref to compare against")
    parser.add_argument("--fixture", action="append", default=[], help="fixture path relative to repo root, repeatable")
    parser.add_argument("--open", action="store_true", help="open report with xdg-open after generation")
    args = parser.parse_args()

    fixtures = args.fixture or DEFAULT_FIXTURES
    out_dir = Path(args.out_dir).resolve()
    if out_dir.exists():
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True)

    before = None
    worktree = None
    try:
        if args.before_ref:
            worktree = create_worktree(args.before_ref)
            before = render_suite(worktree, fixtures, out_dir / "before")
        current = render_suite(ROOT, fixtures, out_dir / "current")
        report = write_report(out_dir, fixtures, current, before)
        print(report)
        if args.open:
            opener = shutil.which("xdg-open") or shutil.which("open")
            if opener:
                run([opener, str(report)])
        return 0
    finally:
        if worktree is not None:
            remove_worktree(worktree)


if __name__ == "__main__":
    raise SystemExit(main())
