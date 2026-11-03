#!/usr/bin/env python3
"""Summarise and plot an Orbital performance CSV.

Usage:
    python performance_report.py path/to/performance.csv
    python performance_report.py performance.csv --output report.png
"""

from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


COUNTER_COLUMNS = [
    "shader_modules_delta",
    "pipelines_delta",
    "unique_wgsl",
    "material_hits_delta",
    "material_misses_delta",
    "sky_updates_delta",
]


def load_csv(path: Path) -> pd.DataFrame:
    frame = pd.read_csv(path)
    required = {"elapsed_s", "fps", "cdt_ms"}
    missing = required.difference(frame.columns)
    if missing:
        raise ValueError(f"{path} is missing required columns: {sorted(missing)}")
    return frame.sort_values("elapsed_s").reset_index(drop=True)


def stage_columns(frame: pd.DataFrame) -> list[str]:
    reserved = {"elapsed_s", "fps", "cdt_ms", *COUNTER_COLUMNS}
    return [column for column in frame.columns if column not in reserved]


def summary(frame: pd.DataFrame) -> pd.Series:
    stages = stage_columns(frame)
    total = frame["total"] if "total" in frame else frame["cdt_ms"]
    fps = frame["fps"].astype(float)

    result = {
        "samples": len(frame),
        "duration_s": float(frame["elapsed_s"].max() - frame["elapsed_s"].min()),
        "fps_mean": float(fps.mean()),
        "fps_median": float(fps.median()),
        "fps_p95": float(np.nanpercentile(fps, 95)),
        "fps_p99": float(np.nanpercentile(fps, 99)),
        "frame_total_mean_ms": float(total.mean()),
        "frame_total_p95_ms": float(np.nanpercentile(total, 95)),
        "frame_total_p99_ms": float(np.nanpercentile(total, 99)),
        "cdt_mean_ms": float(frame["cdt_ms"].mean()),
        "cdt_p95_ms": float(np.nanpercentile(frame["cdt_ms"], 95)),
    }

    if stages:
        means = frame[stages].mean(numeric_only=True).sort_values(ascending=False)
        result["slowest_stage"] = str(means.index[0])
        result["slowest_stage_mean_ms"] = float(means.iloc[0])
        result["slowest_stage_share_pct"] = (
            float(means.iloc[0] / total.mean() * 100.0) if total.mean() else 0.0
        )

    return pd.Series(result)


def print_analysis(frame: pd.DataFrame) -> None:
    stages = stage_columns(frame)
    total = frame["total"] if "total" in frame else frame["cdt_ms"]
    fps = frame["fps"].astype(float)

    print("Summary")
    print("=======")
    print(summary(frame).to_string())

    if stages:
        means = frame[stages].mean(numeric_only=True).sort_values(ascending=False)
        print("\nMean stage cost")
        print("===============")
        for name, value in means.head(10).items():
            share = value / total.mean() * 100.0 if total.mean() else 0.0
            print(f"{name:24s} {value:8.3f} ms  ({share:5.1f}%)")

    spike_threshold = max(float(total.median()) * 2.0, 1e-6)
    spikes = int((total > spike_threshold).sum())
    fps_cv = float(fps.std() / fps.mean() * 100.0) if fps.mean() else 0.0

    print("\nAutomated observations")
    print("======================")
    print(f"Frame-time spikes above {spike_threshold:.2f} ms: {spikes}")
    print(f"FPS coefficient of variation: {fps_cv:.1f}%")

    if fps_cv > 10.0:
        print("FPS is noticeably variable; inspect the frame-time and stage graphs.")
    if spikes > 0:
        print("Spikes are present; compare their timestamps with stage peaks.")


def make_plots(frame: pd.DataFrame, output: Path) -> None:
    stages = stage_columns(frame)
    cpu_stages = [name for name in stages if not name.startswith("gpu_") and name != "total"]
    gpu_stages = [name for name in stages if name.startswith("gpu_")]
    total = frame["total"] if "total" in frame else frame["cdt_ms"]
    elapsed = frame["elapsed_s"]

    figure, axes = plt.subplots(3, 2, figsize=(15, 12))
    figure.suptitle("Orbital performance report", fontsize=14)

    ax = axes[0][0]
    ax.plot(elapsed, frame["fps"], color="#4aa3ff", linewidth=1.2)
    ax.set_title("FPS")
    ax.set_ylabel("frames/s")
    ax.grid(alpha=0.25)

    ax = axes[0][1]
    ax.plot(elapsed, total, label="frame total", color="#ffb347", linewidth=1.1)
    ax.plot(elapsed, frame["cdt_ms"], label="CDT", color="#d16bff", linewidth=0.9)
    ax.set_title("Frame time")
    ax.set_ylabel("ms")
    ax.legend()
    ax.grid(alpha=0.25)

    ax = axes[1][0]
    if cpu_stages:
        means = frame[cpu_stages].mean(numeric_only=True).sort_values()
        ax.barh(means.index, means.values, color="#55c98a")
        ax.set_title("CPU stage mean")
        ax.set_xlabel("ms")
        ax.tick_params(axis="y", labelsize=8)
    else:
        ax.set_visible(False)

    ax = axes[1][1]
    if gpu_stages:
        means = frame[gpu_stages].mean(numeric_only=True).sort_values()
        ax.barh(means.index, means.values, color="#ff8c69")
        ax.set_title("GPU stage mean")
        ax.set_xlabel("ms")
        ax.tick_params(axis="y", labelsize=8)
    else:
        ax.set_visible(False)

    ax = axes[2][0]
    if cpu_stages:
        top = frame[cpu_stages].mean(numeric_only=True).sort_values(ascending=False).head(6)
        for name in top.index:
            ax.plot(elapsed, frame[name], label=name, linewidth=0.9)
        ax.set_title("Top CPU stages over time")
        ax.set_ylabel("ms")
        ax.legend(fontsize=7, ncol=2)
        ax.grid(alpha=0.25)
    else:
        ax.set_visible(False)

    ax = axes[2][1]
    counters = [name for name in COUNTER_COLUMNS if name in frame]
    if counters:
        for name in counters:
            ax.plot(elapsed, frame[name], label=name, linewidth=0.9)
        ax.set_title("Cache / creation counters")
        ax.legend(fontsize=7, ncol=2)
        ax.grid(alpha=0.25)
    else:
        ax.set_visible(False)

    for row in axes:
        for ax in row:
            if ax.get_visible():
                ax.set_xlabel("elapsed_s")

    figure.tight_layout(rect=(0, 0, 1, 0.97))
    figure.savefig(output, dpi=150)
    plt.close(figure)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("csv", type=Path, help="Path to Logs/performance.csv")
    parser.add_argument(
        "--output",
        type=Path,
        help="PNG output path (default: performance_report.png next to the CSV)",
    )
    args = parser.parse_args()

    frame = load_csv(args.csv)
    output = args.output or args.csv.with_name("performance_report.png")
    output.parent.mkdir(parents=True, exist_ok=True)
    print_analysis(frame)
    make_plots(frame, output)
    print(f"\nWrote {output}")


if __name__ == "__main__":
    main()
