import Foundation
import Metal
import Dispatch

func oneLine(_ error: Error) -> String {
    String(describing: error).replacingOccurrences(of: "\n", with: " ")
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
    print("corrupt_library=unexpected-success")
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

do {
    let library = try device.makeLibrary(source: source, options: nil)
    print("source_library=success")

    if library.makeFunction(name: "missing_entry") == nil {
        print("missing_function=lookup-miss")
    } else {
        print("missing_function=unexpected-success")
    }

    if let valid = library.makeFunction(name: "valid_compute") {
        do {
            _ = try device.makeComputePipelineState(function: valid)
            print("valid_pipeline=success")
        } catch {
            print("valid_pipeline=pipeline-error error=\(oneLine(error))")
        }
    } else {
        print("valid_function=unexpected-lookup-miss")
    }

    if let vertex = library.makeFunction(name: "vertex_only") {
        do {
            _ = try device.makeComputePipelineState(function: vertex)
            print("wrong_stage_pipeline=unexpected-success")
        } catch {
            print("wrong_stage_pipeline=pipeline-error error=\(oneLine(error))")
        }
    } else {
        print("vertex_function=unexpected-lookup-miss")
    }
} catch {
    print("source_library=compile-error error=\(oneLine(error))")
    exit(3)
}
