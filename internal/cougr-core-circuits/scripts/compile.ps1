Param(
    [string]$Circom = "circom"
)

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$CircomDir = Join-Path $Root "..\circom" | Resolve-Path -Relative
$OutDir = Join-Path $Root "..\artifacts" | Resolve-Path -Relative

if (-not (Get-Command $Circom -ErrorAction SilentlyContinue)) {
    Write-Error "circom not found - install circom and ensure it's on PATH"
    exit 1
}

New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

${circuits} = @("hidden_cards","fog_of_war","fair_dice","sealed_bid")
foreach ($c in ${circuits}) {
    Write-Host ("compiling {0}..." -f $c)
    & $Circom (Join-Path $CircomDir ("{0}.circom" -f $c)) -l (Join-Path $Root "..\node_modules\circomlib\circuits") -l $CircomDir --r1cs --wasm --sym -o $OutDir
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "artifacts written to $OutDir"
