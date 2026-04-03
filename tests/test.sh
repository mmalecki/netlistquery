#!/usr/bin/env bash
dir=$(dirname "$0")
cli=$(realpath "$dir/../examples/cli.sh")
diff <($cli) "$dir/golden-output"
