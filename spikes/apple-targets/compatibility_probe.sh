#!/bin/zsh
set -u
set -o pipefail
setopt extendedglob

# Bounded Metal artifact-family and reproducibility probe. The script never
# downloads components or changes the selected developer directory. A
# successful exit means both compilation and the evidence record validated.

script_dir="${0:A:h}"
repo_root="${script_dir:h:h}"
result_root=${1:-$(mktemp -d "${TMPDIR:-/tmp}/tiler-apple-artifact-probe.XXXXXX")}
record_path="$result_root/record.tsv"

mkdir -p \
  "$result_root/src-a" "$result_root/src-b" \
  "$result_root/out-a" "$result_root/out-b" "$result_root/sdk"
: >"$record_path"

normalize() {
  local value="$1"
  value=${value//$'\n'/ }
  value=${value//$'\t'/ }
  value=${value##[[:space:]]##}
  value=${value%%[[:space:]]##}
  print -r -- "$value"
}

record() {
  local key="$1"
  local value
  value=$(normalize "$2")
  print -r -- "$key"$'\t'"$value" | tee -a "$record_path"
}

fail() {
  record "probe.failure" "$1"
  print -u2 -r -- "compatibility probe failed: $1"
  exit 4
}

capture() {
  local destination="$1"
  local pattern="$2"
  shift 2
  local output
  output=$("$@" 2>&1) || fail "$destination command failed: $*"
  output=$(normalize "$output")
  [[ -n "$output" ]] || fail "$destination was empty"
  [[ "$output" =~ "$pattern" ]] || fail "$destination was malformed: $output"
  record "$destination" "$output"
}

record_digest() {
  local key="$1"
  local file_path="$2"
  local output
  output=$(shasum -a 256 "$file_path" 2>&1) \
    || fail "could not hash $file_path: $output"
  output=${output%% *}
  [[ "$output" =~ '^[0-9a-f]{64}$' ]] \
    || fail "malformed SHA-256 for $file_path: $output"
  record "$key" "$output"
}

for command_name in xcode-select xcodebuild xcrun sw_vers uname plutil shasum cmp uv; do
  command -v "$command_name" >/dev/null 2>&1 || fail "required command unavailable: $command_name"
done

kernel_a="$result_root/src-a/copy.metal"
kernel_b="$result_root/src-b/copy.metal"
cp "$script_dir/copy.metal" "$kernel_a" || fail "could not copy source A"
cp "$script_dir/copy.metal" "$kernel_b" || fail "could not copy source B"

record "schema" "tiler.apple-target-compatibility/v1"
record "probe.result_root" "."
record_digest "probe.source_sha256" "$script_dir/copy.metal"
record "probe.compiler_flags" "-std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off"
capture "host.date_utc" '^[0-9]{4}-[0-9]{2}-[0-9]{2}T' date -u +%Y-%m-%dT%H:%M:%SZ
capture "host.developer_dir" '^/' xcode-select -p
capture "host.xcode" '^Xcode [^ ]+ Build version [^ ]+' zsh -c 'xcodebuild -version | tr "\n" " "'
capture "host.metal_toolchain_component" '.+' zsh -c 'xcodebuild -showComponent MetalToolchain | tr "\n" " "'
capture "host.xcrun" '[0-9]+' xcrun --version
capture "host.os_version" '^[0-9]+([.][0-9]+)+' sw_vers -productVersion
capture "host.os_build" '^[A-Za-z0-9]+$' sw_vers -buildVersion
capture "host.machine" '^(arm64|x86_64)$' uname -m

for sdk in macosx iphoneos iphonesimulator; do
  sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path 2>&1) || fail "sdk.$sdk.path command failed"
  [[ -d "$sdk_path" ]] || fail "sdk.$sdk.path was not a directory: $sdk_path"
  record "sdk.$sdk.path" "$sdk_path"
  capture "sdk.$sdk.version" '^[0-9]+([.][0-9]+)+$' xcrun --sdk "$sdk" --show-sdk-version
  capture "sdk.$sdk.build" '^[A-Za-z0-9]+$' xcrun --sdk "$sdk" --show-sdk-build-version
  settings_path="$result_root/sdk/$sdk.settings.txt"
  plutil -p "$sdk_path/SDKSettings.plist" \
    | grep -E 'DefaultDeploymentTarget|MinimumDeploymentTarget|MaximumDeploymentTarget|LLVMTargetTriple(Environment|Sys|Vendor)' \
    >"$settings_path" || fail "sdk.$sdk settings were unavailable"
  [[ -s "$settings_path" ]] || fail "sdk.$sdk settings were empty"
  record "sdk.$sdk.settings_file" "sdk/$sdk.settings.txt"
  record_digest "sdk.$sdk.settings_sha256" "$settings_path"
done

metal_path=$(xcrun --sdk macosx --find metal 2>&1) || fail "Metal compiler lookup failed: $metal_path"
metallib_path=$(xcrun --sdk macosx --find metallib 2>&1) || fail "metallib linker lookup failed: $metallib_path"
[[ -x "$metal_path" ]] || fail "Metal compiler was not executable: $metal_path"
[[ -x "$metallib_path" ]] || fail "metallib linker was not executable: $metallib_path"
record "tool.metal.path" "$metal_path"
record "tool.metallib.path" "$metallib_path"
capture "tool.metal.version" '(metal|Metal).*[0-9]' xcrun --sdk macosx metal --version
capture "tool.metallib.version" '(AIR-LLD|metallib|metalfe).*[0-9]' xcrun --sdk macosx metallib --version
record_digest "tool.metal.sha256" "$metal_path"
record_digest "tool.metallib.sha256" "$metallib_path"

families=(
  'macosx|macos13|air64-apple-macos13.0'
  'macosx|macos14|air64-apple-macos14.0'
  'iphoneos|ios16|air64-apple-ios16.0'
  'iphoneos|ios17|air64-apple-ios17.0'
  'iphonesimulator|iossim16|air64-apple-ios16.0-simulator'
  'iphonesimulator|iossim17|air64-apple-ios17.0-simulator'
)

compile_one() {
  local run="$1"
  local sdk="$2"
  local label="$3"
  local target="$4"
  local output_dir="$5"
  local kernel="$6"
  local air="$output_dir/$label.air"
  local library="$output_dir/$label.metallib"
  local log="$output_dir/$label.log"
  local prefix="matrix.$label.$run"

  record "$prefix.sdk" "$sdk"
  record "$prefix.target" "$target"
  record "$prefix.command.metal" "ZERO_AR_DATE=1 xcrun --sdk $sdk metal -target $target -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off -c <source> -o <air>"
  print -r -- "command=ZERO_AR_DATE=1 xcrun --sdk $sdk metal -target $target -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off -c $kernel -o $air" >"$log"
  if ! ZERO_AR_DATE=1 xcrun --sdk "$sdk" metal \
      -target "$target" -std=metal3.1 -O2 \
      -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise \
      -ffp-contract=off -c "$kernel" -o "$air" >>"$log" 2>&1; then
    fail "$prefix Metal compilation failed; see $log"
  fi
  record "$prefix.command.metallib" "ZERO_AR_DATE=1 xcrun --sdk $sdk metallib <air> -o <metallib>"
  print -r -- "command=ZERO_AR_DATE=1 xcrun --sdk $sdk metallib $air -o $library" >>"$log"
  if ! ZERO_AR_DATE=1 xcrun --sdk "$sdk" metallib \
      "$air" -o "$library" >>"$log" 2>&1; then
    fail "$prefix metallib link failed; see $log"
  fi
  record_digest "$prefix.air_sha256" "$air"
  record_digest "$prefix.metallib_sha256" "$library"
  record_digest "$prefix.log_sha256" "$log"
}

for spec in $families; do
  parts=( ${(s:|:)spec} )
  compile_one a "$parts[1]" "$parts[2]" "$parts[3]" "$result_root/out-a" "$kernel_a"
done

for spec in $families; do
  parts=( ${(s:|:)spec} )
  compile_one b "$parts[1]" "$parts[2]" "$parts[3]" "$result_root/out-b" "$kernel_b"
  for suffix in air metallib; do
    first="$result_root/out-a/$parts[2].$suffix"
    second="$result_root/out-b/$parts[2].$suffix"
    if cmp -s "$first" "$second"; then
      record "repro.$parts[2].$suffix.byte_identical" "true"
    else
      record "repro.$parts[2].$suffix.byte_identical" "false"
    fi
  done
done

uv run --project "$repo_root" --locked python \
  "$script_dir/validate_compatibility_record.py" "$record_path" \
  || fail "completed record failed validation"
record "probe.status" "validated"
print -r -- "validated_result=$result_root"
