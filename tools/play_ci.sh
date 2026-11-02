#!/usr/bin/env bash
# Downloads the game the last successful Actions build made (with Kinetics in it) and starts it in
# Horizon's game folder, with the resources the launcher prepared, as the launcher starts a version.
#   tools/play_ci.sh            # newest build
#   tools/play_ci.sh RUN_ID     # a given run
set -euo pipefail
repo=$(gh repo view --json nameWithOwner -q .nameWithOwner)
run=${1:-$(gh run list -R "$repo" -w build -s success -L 1 --json databaseId -q '.[0].databaseId')}
[ -n "$run" ] || { echo "no successful build yet: gh run list -R $repo" >&2; exit 1; }
dir=~/.local/share/horizon-kinetics-ci/$run
if [ ! -x "$dir/horizon-client" ]; then
  mkdir -p "$dir" && gh run download -R "$repo" "$run" -n horizon-client -D "$dir"
  chmod +x "$dir/horizon-client"
fi
launcher=~/.local/share/horizon-launcher
tag=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["selected"])' "$launcher/settings.json")
resources=$(ls -d "$launcher/versions/$tag/.local/horizon-client/"*/ | head -1)
echo "game $dir/horizon-client, resources $resources"
cd ~/.local/share/horizon
exec "$dir/horizon-client" --resources "$resources"
