// Dispatch host for the Apple numerical-behaviour probe.
//
// The probe's compile-side observations can be read out of emitted LLVM IR, but
// the returned bit pattern of a subnormal operand cannot: it is a property of
// the GPU executing the linked library. This host is the smallest program that
// loads one precompiled metallib, dispatches one thread per element over a
// shared buffer of caller-supplied bit patterns, and prints what came back.
//
// It reads f32 as raw `uint32_t` on both sides deliberately. Parsing or
// formatting a decimal literal anywhere in the path would let the host's own
// libc rounding stand between the GPU and the recorded measurement.
//
// Exit codes are the harness's classification channel:
//
//   0  the dispatch completed and every result line was printed
//   2  the arguments were malformed (a harness defect, never a skip)
//   3  no default Metal device resolved, which is the device-side analogue of
//      the toolchain-unavailable skip and the only self-skip this host reports
//   4  the toolchain and device resolved and something else failed, which is a
//      defect the harness must surface rather than skip
//
// Output is one `key=value` line per fact on stdout, in dispatch order.

#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

enum {
    kProbeExitOk = 0,
    kProbeExitUsage = 2,
    kProbeExitNoDevice = 3,
    kProbeExitFailure = 4,
};

static int probe_fail(NSString *stage, NSError *error) {
    fprintf(stderr, "numerical_probe_host: %s failed: %s\n", stage.UTF8String,
            error == nil ? "no error object" : error.localizedDescription.UTF8String);
    return kProbeExitFailure;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc < 4) {
            fprintf(stderr, "usage: %s <metallib> <function> <hex-operand>...\n", argv[0]);
            return kProbeExitUsage;
        }
        NSString *libraryPath = @(argv[1]);
        NSString *functionName = @(argv[2]);
        NSUInteger count = (NSUInteger)(argc - 3);

        uint32_t *operands = calloc(count, sizeof(uint32_t));
        if (operands == NULL) {
            fprintf(stderr, "numerical_probe_host: could not allocate the operand vector\n");
            return kProbeExitFailure;
        }
        for (NSUInteger index = 0; index < count; index += 1) {
            const char *text = argv[3 + index];
            char *end = NULL;
            unsigned long long value = strtoull(text, &end, 16);
            if (end == text || *end != '\0' || value > 0xffffffffULL) {
                fprintf(stderr, "numerical_probe_host: malformed hex operand: %s\n", text);
                free(operands);
                return kProbeExitUsage;
            }
            operands[index] = (uint32_t)value;
        }

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            fprintf(stderr, "numerical_probe_host: no default Metal device resolved\n");
            free(operands);
            return kProbeExitNoDevice;
        }
        printf("device=%s\n", device.name.UTF8String);
        printf("registry-id=%llu\n", (unsigned long long)device.registryID);

        NSError *error = nil;
        NSURL *libraryURL = [NSURL fileURLWithPath:libraryPath];
        id<MTLLibrary> library = [device newLibraryWithURL:libraryURL error:&error];
        if (library == nil) {
            free(operands);
            return probe_fail(@"library load", error);
        }
        id<MTLFunction> function = [library newFunctionWithName:functionName];
        if (function == nil) {
            fprintf(stderr, "numerical_probe_host: function lookup returned nil: %s\n",
                    functionName.UTF8String);
            free(operands);
            return kProbeExitFailure;
        }
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (pipeline == nil) {
            free(operands);
            return probe_fail(@"pipeline creation", error);
        }
        id<MTLCommandQueue> queue = [device newCommandQueue];
        if (queue == nil) {
            fprintf(stderr, "numerical_probe_host: command queue creation returned nil\n");
            free(operands);
            return kProbeExitFailure;
        }

        NSUInteger bytes = count * sizeof(uint32_t);
        id<MTLBuffer> input = [device newBufferWithBytes:operands
                                                  length:bytes
                                                 options:MTLResourceStorageModeShared];
        // The output buffer is seeded with a pattern no probe kernel can
        // produce, so a kernel that never wrote an element is distinguishable
        // from one that wrote a zero.
        uint32_t *unwritten = calloc(count, sizeof(uint32_t));
        if (unwritten == NULL) {
            fprintf(stderr, "numerical_probe_host: could not allocate the sentinel vector\n");
            free(operands);
            return kProbeExitFailure;
        }
        for (NSUInteger index = 0; index < count; index += 1) {
            unwritten[index] = 0xdeadbeefu;
        }
        id<MTLBuffer> output = [device newBufferWithBytes:unwritten
                                                   length:bytes
                                                  options:MTLResourceStorageModeShared];
        free(unwritten);
        free(operands);
        if (input == nil || output == nil) {
            fprintf(stderr, "numerical_probe_host: shared buffer allocation returned nil\n");
            return kProbeExitFailure;
        }

        id<MTLCommandBuffer> commands = [queue commandBuffer];
        id<MTLComputeCommandEncoder> encoder = [commands computeCommandEncoder];
        if (commands == nil || encoder == nil) {
            fprintf(stderr, "numerical_probe_host: command encoding returned nil\n");
            return kProbeExitFailure;
        }
        [encoder setComputePipelineState:pipeline];
        [encoder setBuffer:input offset:0 atIndex:0];
        [encoder setBuffer:output offset:0 atIndex:1];
        NSUInteger width = pipeline.maxTotalThreadsPerThreadgroup;
        if (width > count) {
            width = count;
        }
        [encoder dispatchThreads:MTLSizeMake(count, 1, 1)
           threadsPerThreadgroup:MTLSizeMake(width, 1, 1)];
        [encoder endEncoding];
        [commands commit];
        [commands waitUntilCompleted];

        // AGENTS.md requires exact command-buffer terminal success before host
        // validation readback. A buffer that errored or was not completed makes
        // the shared allocation's contents meaningless, not merely suspect.
        if (commands.status != MTLCommandBufferStatusCompleted) {
            fprintf(stderr, "numerical_probe_host: command buffer terminal status was %ld\n",
                    (long)commands.status);
            return probe_fail(@"command buffer", commands.error);
        }
        if (commands.error != nil) {
            return probe_fail(@"command buffer", commands.error);
        }

        const uint32_t *results = (const uint32_t *)output.contents;
        for (NSUInteger index = 0; index < count; index += 1) {
            if (results[index] == 0xdeadbeefu) {
                fprintf(stderr, "numerical_probe_host: element %lu was never written\n",
                        (unsigned long)index);
                return kProbeExitFailure;
            }
            printf("result=%08x\n", results[index]);
        }
        return kProbeExitOk;
    }
}
