cargo fmt --all
cargo clippy

$crates = @(
  "karlsen-wrpc-wasm",
  "karlsen-wallet-cli-wasm",
  "karlsen-wasm",
  "karlsen-cli",
  "kaspa-os",
  "karlsen-daemon"
)

$env:AR="llvm-ar"
foreach ($crate in $crates)
{
  Write-Output "`ncargo clippy -p $crate --target wasm32-unknown-unknown"
  cargo clippy -p $crate --target wasm32-unknown-unknown
  $status=$LASTEXITCODE
  if($status -ne 0) {
    Write-Output "`n--> wasm32 check of $crate failed`n"
    break
  }
}
$env:AR=""