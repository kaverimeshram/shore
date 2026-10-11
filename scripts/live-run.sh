#!/usr/bin/env bash
# SHORE live mainnet run helper for kamal. It never signs or sends anything: Phantom does the one transfer.
#   scripts/live-run.sh check <phantom-pubkey>   preflight: API up, ZEC and SOL balances via SHORE's /api/rpc
#   scripts/live-run.sh status <deposit-address>  1Click status for the shield (the page also shows it)
set -euo pipefail
API=${SHORE_API:-https://airlock-zec.fly.dev}
MINT=A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS
rpc() { curl -fsS -m 20 -H 'content-type: application/json' -d "$1" "$API/api/rpc"; }
case "${1:-}" in
check)
  pk=${2:?phantom pubkey}
  zec=$(rpc "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getTokenAccountsByOwner\",\"params\":[\"$pk\",{\"mint\":\"$MINT\"},{\"encoding\":\"jsonParsed\"}]}" |
    python3 -c 'import json,sys;v=json.load(sys.stdin)["result"]["value"];print(sum(int(a["account"]["data"]["parsed"]["info"]["tokenAmount"]["amount"]) for a in v))')
  sol=$(rpc "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getBalance\",\"params\":[\"$pk\"]}" | python3 -c 'import json,sys;print(json.load(sys.stdin)["result"]["value"])')
  echo "ZEC on Solana: $zec base units ($(python3 -c "print($zec/1e8)") ZEC); need >= 133334 (0.00133334), suggested 140000 (0.0014), cap 2000000"
  echo "SOL: $sol lamports ($(python3 -c "print($sol/1e9)") SOL); need ~0.003 SOL (fee + possible token-account rent at the deposit address)"
  ok=1; [ "$zec" -ge 133334 ] || { echo "NOT READY: not enough ZEC"; ok=0; }; [ "$sol" -ge 3000000 ] || { echo "NOT READY: top up SOL"; ok=0; }
  [ $ok = 1 ] && echo "READY. Open https://shore-zec.pages.dev/shore/?live=1 in the browser with Phantom." ;;
status)
  curl -fsS -m 20 "$API/api/status/${2:?deposit address}"; echo ;;
*) sed -n 2,5p "$0"; exit 2 ;;
esac
