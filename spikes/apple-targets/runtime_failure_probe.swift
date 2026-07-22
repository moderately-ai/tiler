import Dispatch
import Foundation
import Metal

enum UnexpectedStage: String, CaseIterable {
    case corruptLibraryAccepted = "corrupt-library-accepted"
    case sourceLibraryRejected = "source-library-rejected"
    case missingFunctionFound = "missing-function-found"
    case validFunctionMissing = "valid-function-missing"
    case validPipelineRejected = "valid-pipeline-rejected"
    case vertexFunctionMissing = "vertex-function-missing"
    case wrongStagePipelineAccepted = "wrong-stage-pipeline-accepted"
}

func oneLine(_ error: Error) -> String {
    String(describing: error).replacingOccurrences(of: "\n", with: " ")
}

func unexpected(_ stage: UnexpectedStage, _ detail: String = "") -> Never {
    let suffix = detail.isEmpty ? "" : " detail=\(detail)"
    print("probe=unexpected stage=\(stage.rawValue)\(suffix)")
    exit(1)
}

let environment = ProcessInfo.processInfo.environment
if let requested = environment["TILER_APPLE_RUNTIME_INJECT"] {
    guard let stage = UnexpectedStage(rawValue: requested) else {
        print("probe=invalid-injection value=\(requested)")
        exit(64)
    }
    unexpected(stage, "injected")
}

if CommandLine.arguments == [CommandLine.arguments[0], "--list-injections"] {
    for stage in UnexpectedStage.allCases {
        print(stage.rawValue)
    }
    exit(0)
}

guard CommandLine.arguments.count == 1 else {
    print("usage: \(CommandLine.arguments[0]) [--list-injections]")
    exit(64)
}

guard let device = MTLCreateSystemDefaultDevice() else {
    print("device=unavailable")
    exit(2)
}
print("device=\(device.name)")

let corruptBytes = Data([0x4d, 0x54, 0x4c, 0x42, 0x00, 0x01, 0x02, 0x03])
let corrupt = corruptBytes.withUnsafeBytes { DispatchData(bytes: $0) }
do {
    _ = try device.makeLibrary(data: corrupt)
    unexpected(.corruptLibraryAccepted)
} catch {
    print("corrupt_library=load-error error=\(oneLine(error))")
}

let source = """
#include <metal_stdlib>
using namespace metal;

kernel void valid_compute(device float *values [[buffer(0)]],
                          uint id [[thread_position_in_grid]]) {
    values[id] += 1.0f;
}

vertex float4 vertex_only(uint id [[vertex_id]]) {
    return float4(float(id), 0.0f, 0.0f, 1.0f);
}
"""

let library: MTLLibrary
do {
    library = try device.makeLibrary(source: source, options: nil)
    print("source_library=success")
} catch {
    unexpected(.sourceLibraryRejected, oneLine(error))
}

if library.makeFunction(name: "missing_entry") == nil {
    print("missing_function=lookup-miss")
} else {
    unexpected(.missingFunctionFound)
}

guard let valid = library.makeFunction(name: "valid_compute") else {
    unexpected(.validFunctionMissing)
}
do {
    _ = try device.makeComputePipelineState(function: valid)
    print("valid_pipeline=success")
} catch {
    unexpected(.validPipelineRejected, oneLine(error))
}

guard let vertex = library.makeFunction(name: "vertex_only") else {
    unexpected(.vertexFunctionMissing)
}
do {
    _ = try device.makeComputePipelineState(function: vertex)
    unexpected(.wrongStagePipelineAccepted)
} catch {
    print("wrong_stage_pipeline=pipeline-error error=\(oneLine(error))")
}

print("probe=validated")
