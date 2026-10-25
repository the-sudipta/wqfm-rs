# Paths and limits of the wQFM-rs benchmark (sourced by the bench scripts).
# Paths are as seen from WSL Ubuntu 24.04; every method runs there.

BENCH=${WQFM_BENCH:-/mnt/d/_external/wqfm_bench}  # downloads, built tools, inputs, results
REPO=/mnt/d/wqfm-rs                 # this repository
WQFM_SRC=/mnt/d/_external/wQFM-2020 # wQFM at commit 7bfdf8e (jar, script, helper)

TOOLS=$BENCH/tools_built
INPUTS=$BENCH/inputs
RESULTS=$BENCH/results

# Limits of every run (PREREGISTRATION.md section 5 and Amendment 1).
TIME_LIMIT=21600               # seconds (6 hours)
JAVA_HEAP=6g                   # -Xmx for the Java methods
FILE_LIMIT_KB=$((20 * 1024 * 1024))  # largest file a run may write (20 GB)
