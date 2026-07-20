#!/bin/zsh
set -u
set -o pipefail

# Bounded Metal artifact-family and reproducibility probe. The script never
# downloads components or changes the selected developer directory.

probe_root=$(mktemp -d "${TMPDIR:-/tmp}/tiler-apple-artifact-probe.XXXXXX")
mkdir -p "$probe_root/src-a" "$probe_root/src-b" "$probe_root/out-a" "$probe_root/out-b"

script_dir="${0:A:h}"
kernel_a="$probe_root/src-a/copy.metal"
kernel_b="$probe_root/src-b/copy.metal"
cp "$script_dir/copy.metal" "$kernel_a"
cp "$script_dir/copy.metal" "$kernel_b"

print 'section=host'
print "probe_root=$probe_root"
print "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
print "developer_dir=$(xcode-select -p 2>&1)"
print "xcode=$(xcodebuild -version 2>&1 | tr '\n' ' ')"
print "xcrun=$(xcrun --version 2>&1)"
print "os=$(sw_vers -productVersion)"
print "os_build=$(sw_vers -buildVersion)"
print "machine=$(uname -m)"

for sdk in macosx iphoneos iphonesimulator; do
  print "section=sdk sdk=$sdk"
  print "path=$(xcrun --sdk "$sdk" --show-sdk-path 2>&1)"
  print "version=$(xcrun --sdk "$sdk" --show-sdk-version 2>&1)"
  print "build=$(xcrun --sdk "$sdk" --show-sdk-build-version 2>&1)"
  sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
  plutil -p "$sdk_path/SDKSettings.plist" \
    | grep -E 'DefaultDeploymentTarget|MinimumDeploymentTarget|MaximumDeploymentTarget|LLVMTargetTriple(Environment|Sys|Vendor)' \
    | sed "s/^/sdk_setting sdk=$sdk /"
done

metal_path=$(xcrun --sdk macosx --find metal 2>/dev/null || true)
metallib_path=$(xcrun --sdk macosx --find metallib 2>/dev/null || true)
print 'section=tools'
print "metal_path=$metal_path"
print "metallib_path=$metallib_path"
if [[ -n "$metal_path" ]]; then
  print "metal_launcher_sha256=$(shasum -a 256 "$metal_path" | awk '{print $1}')"
  metal_version_output=$(xcrun --sdk macosx metal --version 2>&1)
  metal_version_status=$?
  print "metal_version_status=$metal_version_status"
  print "metal_version_output=${metal_version_output//$'\n'/ }"
fi

if [[ -z "$metal_path" || -z "$metallib_path" ]]; then
  print 'toolchain_preflight=unavailable'
  print 'compile_matrix=not-run'
  print 'reproducibility=not-run'
  exit 4
fi

metal_version=$(xcrun --sdk macosx metal --version 2>&1)
metallib_version=$(xcrun --sdk macosx metallib --version 2>&1)
print "metal_version=$metal_version"
print "metallib_version=$metallib_version"
print "metal_sha256=$(shasum -a 256 "$metal_path" | awk '{print $1}')"
print "metallib_sha256=$(shasum -a 256 "$metallib_path" | awk '{print $1}')"

families=(
  'macosx|macos13|air64-apple-macos13.0'
  'macosx|macos14|air64-apple-macos14.0'
  'iphoneos|ios16|air64-apple-ios16.0'
  'iphoneos|ios17|air64-apple-ios17.0'
  'iphonesimulator|iossim16|air64-apple-ios16.0-simulator'
  'iphonesimulator|iossim17|air64-apple-ios17.0-simulator'
)

compile_one() {
  local sdk="$1"
  local label="$2"
  local target="$3"
  local output_dir="$4"
  local kernel="$5"
  local air="$output_dir/$label.air"
  local library="$output_dir/$label.metallib"
  local log="$output_dir/$label.log"

  if ! ZERO_AR_DATE=1 xcrun --sdk "$sdk" metal \
      -target "$target" -std=metal3.1 -O2 \
      -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise \
      -ffp-contract=off -c "$kernel" -o "$air" >"$log" 2>&1; then
    print "family=$label stage=metal result=failed log=$log"
    return 1
  fi
  if ! ZERO_AR_DATE=1 xcrun --sdk "$sdk" metallib \
      "$air" -o "$library" >>"$log" 2>&1; then
    print "family=$label stage=metallib result=failed log=$log"
    return 1
  fi
  print "family=$label stage=complete result=success air_sha256=$(shasum -a 256 "$air" | awk '{print $1}') metallib_sha256=$(shasum -a 256 "$library" | awk '{print $1}')"
  strings -a "$air" | grep -E 'air64-apple|metalfe|Apple LLVM|SDK Version' | sort -u | sed "s/^/metadata family=$label /" || true
}

print 'section=compile-matrix'
matrix_failed=0
for spec in $families; do
  parts=( ${(s:|:)spec} )
  compile_one "$parts[1]" "$parts[2]" "$parts[3]" "$probe_root/out-a" "$kernel_a" || matrix_failed=1
done

print 'section=reproducibility'
for spec in $families; do
  parts=( ${(s:|:)spec} )
  compile_one "$parts[1]" "$parts[2]" "$parts[3]" "$probe_root/out-b" "$kernel_b" || matrix_failed=1
  for suffix in air metallib; do
    first="$probe_root/out-a/$parts[2].$suffix"
    second="$probe_root/out-b/$parts[2].$suffix"
    if [[ -f "$first" && -f "$second" ]]; then
      if cmp -s "$first" "$second"; then
        print "family=$parts[2] artifact=$suffix byte_identical=true"
      else
        print "family=$parts[2] artifact=$suffix byte_identical=false first_sha256=$(shasum -a 256 "$first" | awk '{print $1}') second_sha256=$(shasum -a 256 "$second" | awk '{print $1}')"
      fi
    fi
  done
done

exit "$matrix_failed"
