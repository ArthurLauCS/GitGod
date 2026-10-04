param(
    [Parameter(Mandatory)][string]$Before,
    [Parameter(Mandatory)][string]$After,
    [Parameter(Mandatory)][string]$Repo,
    [Parameter(Mandatory)][string]$File,
    [ValidateSet('untracked', 'staged')][string]$Mode = 'untracked'
)
$ErrorActionPreference = 'Stop'
# 先各预热一次，再交替测量 5 次；内存仅统计引擎进程，不含 Git 子进程。
foreach ($round in 0..5) {
    foreach ($variant in @(@{Name='before';Exe=$Before}, @{Name='after';Exe=$After})) {
        $start = [System.Diagnostics.ProcessStartInfo]::new((Resolve-Path $variant.Exe).Path)
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardInput = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $start.Environment['GITGOD_BENCH_HOLD'] = '1'
        foreach ($arg in @((Resolve-Path $Repo).Path, $File, $Mode)) { $start.ArgumentList.Add($arg) }
        $process = [System.Diagnostics.Process]::Start($start)
        $result = $process.StandardOutput.ReadLine()
        $process.Refresh()
        $peak = $process.PeakWorkingSet64
        $process.StandardInput.WriteLine()
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) { throw $process.StandardError.ReadToEnd() }
        if ($round -gt 0) {
            [pscustomobject]@{Variant=$variant.Name;Round=$round;Mode=$Mode;PeakMiB=[math]::Round($peak / 1MB, 2);Result=$result}
        }
        $process.Dispose()
    }
}
