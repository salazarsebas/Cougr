Param(
    [string]$Circom = "circom"
)

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$CircomDir = Join-Path $Root "..\circom" | Resolve-Path -Relative
$OutDir = Join-Path $Root "..\artifacts" | Resolve-Path -Relative

$DeckSize = if ($env:DECK_SIZE) { $env:DECK_SIZE } else { "52" }
$HandSize = if ($env:HAND_SIZE) { $env:HAND_SIZE } else { "5" }

if (-not (Get-Command $Circom -ErrorAction SilentlyContinue)) {
    Write-Error "circom not found - install circom and ensure it's on PATH"
    exit 1
}

New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

$HiddenCardsFile = Join-Path $CircomDir "hidden_cards.circom"
(Get-Content $HiddenCardsFile) -replace 'HiddenCards\(\d+,\s*\d+\)', "HiddenCards($DeckSize, $HandSize)" | Set-Content $HiddenCardsFile

${circuits} = @("hidden_cards","fog_of_war","fair_dice","sealed_bid")
foreach ($c in ${circuits}) {
    Write-Host ("compiling {0}..." -f $c)
    & $Circom (Join-Path $CircomDir ("{0}.circom" -f $c)) -l (Join-Path $Root "..\node_modules\circomlib\circuits") -l $CircomDir --r1cs --wasm --sym -o $OutDir
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "artifacts written to $OutDir"
