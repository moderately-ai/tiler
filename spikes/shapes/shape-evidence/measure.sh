#!/usr/bin/env bash
set -euo pipefail

experiment_dir="$(cd "$(dirname "$0")" && pwd)"
exec python3 "${experiment_dir}/measure.py" baseline
