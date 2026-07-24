// Dispatch host for the Apple numerical-behaviour probe.
//
// The probe's compile-side observations can be read out of emitted LLVM IR, but
// the returned bit pattern of a subnormal operand cannot: it is a property of
// the GPU executing the compiled kernel. This host is the smallest program that
// obtains one library, dispatches one thread per element over a shared buffer of
// caller-supplied bit patterns, and prints what came back.
//
// It obtains that library two ways, because Tiler's Metal story has two
// compilation stages and an artifact's declared numerical realization must be
// true of whichever one actually runs:
//
//   library  load a metallib the harness already built offline through
//            `xcrun metal` and `xcrun metallib`
//   source   compile MSL in this process through `newLibraryWithSource:options:`
//            with an explicit `MTLCompileOptions`
//
// Both modes then take the identical path to the GPU, so a difference between
// them is a difference between the two compilers and not between two dispatch
// procedures.
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

static int probe_usage(NSString *detail) {
    fprintf(stderr, "numerical_probe_host: %s\n", detail.UTF8String);
    fprintf(stderr, "usage: numerical_probe_host library <metallib> <function> <hex-operand>...\n");
    fprintf(stderr,
            "       numerical_probe_host source <source.metal> <function> <options> "
            "<hex-operand>...\n");
    fprintf(stderr, "       <options> is a comma-separated key=value list; every key and value "
                    "must be recognized:\n");
    fprintf(stderr, "         math=safe|relaxed|fast  fpfun=fast|precise  lang=3.1|3.2  "
                    "opt=default|size  [archive=<path>]\n");
    return kProbeExitUsage;
}

/// Splits a comma-separated `key=value` list, rejecting a malformed or repeated key.
///
/// Returns nil rather than a partial dictionary. A caller that accepted a
/// partial parse would compile with some selections defaulted, which is the one
/// failure this program must not have: the record would name a configuration
/// the library was not built with.
static NSDictionary<NSString *, NSString *> *parse_option_list(NSString *text) {
    NSMutableDictionary<NSString *, NSString *> *parsed = [NSMutableDictionary dictionary];
    for (NSString *entry in [text componentsSeparatedByString:@","]) {
        NSRange separator = [entry rangeOfString:@"="];
        if (separator.location == NSNotFound || separator.location == 0) {
            return nil;
        }
        NSString *key = [entry substringToIndex:separator.location];
        NSString *value = [entry substringFromIndex:separator.location + 1];
        if (value.length == 0 || parsed[key] != nil) {
            return nil;
        }
        parsed[key] = value;
    }
    return parsed;
}

/// Applies one selection, or returns NO for an unrecognized key or value.
///
/// Failing closed matters more here than anywhere else in this file. A silently
/// ignored selection would leave the property at its API default — which for
/// `mathFloatingPointFunctions` is `Fast`, not the `precise` the offline row
/// pins — and the record would then compare two differently configured
/// compilations while naming them the same case.
static BOOL apply_compile_option(MTLCompileOptions *options, NSString *key, NSString *value) {
    if ([key isEqualToString:@"math"]) {
        if ([value isEqualToString:@"safe"]) {
            options.mathMode = MTLMathModeSafe;
        } else if ([value isEqualToString:@"relaxed"]) {
            options.mathMode = MTLMathModeRelaxed;
        } else if ([value isEqualToString:@"fast"]) {
            options.mathMode = MTLMathModeFast;
        } else {
            return NO;
        }
        return YES;
    }
    if ([key isEqualToString:@"fpfun"]) {
        if ([value isEqualToString:@"fast"]) {
            options.mathFloatingPointFunctions = MTLMathFloatingPointFunctionsFast;
        } else if ([value isEqualToString:@"precise"]) {
            options.mathFloatingPointFunctions = MTLMathFloatingPointFunctionsPrecise;
        } else {
            return NO;
        }
        return YES;
    }
    if ([key isEqualToString:@"lang"]) {
        if ([value isEqualToString:@"3.1"]) {
            options.languageVersion = MTLLanguageVersion3_1;
        } else if ([value isEqualToString:@"3.2"]) {
            options.languageVersion = MTLLanguageVersion3_2;
        } else {
            return NO;
        }
        return YES;
    }
    if ([key isEqualToString:@"opt"]) {
        if ([value isEqualToString:@"default"]) {
            options.optimizationLevel = MTLLibraryOptimizationLevelDefault;
        } else if ([value isEqualToString:@"size"]) {
            options.optimizationLevel = MTLLibraryOptimizationLevelSize;
        } else {
            return NO;
        }
        return YES;
    }
    return NO;
}

/// Renders what the compile options object holds, read back from the properties.
///
/// The harness records this rather than the argument it passed, so a selection
/// that did not take is visible in the record instead of assumed.
static NSString *applied_options(MTLCompileOptions *options) {
    NSString *math = @"unknown";
    switch (options.mathMode) {
        case MTLMathModeSafe: math = @"safe"; break;
        case MTLMathModeRelaxed: math = @"relaxed"; break;
        case MTLMathModeFast: math = @"fast"; break;
    }
    NSString *functions = @"unknown";
    switch (options.mathFloatingPointFunctions) {
        case MTLMathFloatingPointFunctionsFast: functions = @"fast"; break;
        case MTLMathFloatingPointFunctionsPrecise: functions = @"precise"; break;
    }
    NSString *language = @"unknown";
    if (options.languageVersion == MTLLanguageVersion3_1) {
        language = @"3.1";
    } else if (options.languageVersion == MTLLanguageVersion3_2) {
        language = @"3.2";
    }
    NSString *optimization = @"unknown";
    switch (options.optimizationLevel) {
        case MTLLibraryOptimizationLevelDefault: optimization = @"default"; break;
        case MTLLibraryOptimizationLevelSize: optimization = @"size"; break;
    }
    return [NSString stringWithFormat:@"math=%@,fpfun=%@,lang=%@,opt=%@", math, functions, language,
                                      optimization];
}

/// Serializes the pipeline built from `function` into a binary archive at `path`.
///
/// This is the only route by which anything compile-side survives the runtime
/// compilation path: the container the driver writes embeds the compiler's own
/// version string and the module's `air.compile.*` option names. It is a weaker
/// artifact than the emitted LLVM IR the offline path reads — the harness can
/// only test the container for the presence of a byte sequence, never resolve
/// which strings the module attached to its `air.compile_options` node — so a
/// failure here is reported and does not fail the dispatch.
///
/// Returns nil on success, or an explanation of why no archive was written.
static NSString *serialize_archive(id<MTLDevice> device, id<MTLFunction> function, NSString *path) {
    NSError *error = nil;
    MTLBinaryArchiveDescriptor *descriptor = [MTLBinaryArchiveDescriptor new];
    id<MTLBinaryArchive> archive = [device newBinaryArchiveWithDescriptor:descriptor error:&error];
    if (archive == nil) {
        return [NSString stringWithFormat:@"archive creation returned nil: %@",
                                          error.localizedDescription];
    }
    MTLComputePipelineDescriptor *pipeline = [MTLComputePipelineDescriptor new];
    pipeline.computeFunction = function;
    if (![archive addComputePipelineFunctionsWithDescriptor:pipeline error:&error]) {
        return [NSString stringWithFormat:@"archive add failed: %@", error.localizedDescription];
    }
    if (![archive serializeToURL:[NSURL fileURLWithPath:path] error:&error]) {
        return [NSString stringWithFormat:@"archive serialize failed: %@",
                                          error.localizedDescription];
    }
    return nil;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc < 2) {
            return probe_usage(@"no compilation mode given");
        }
        NSString *mode = @(argv[1]);
        BOOL fromSource = [mode isEqualToString:@"source"];
        if (!fromSource && ![mode isEqualToString:@"library"]) {
            return probe_usage([NSString stringWithFormat:@"unknown compilation mode: %@", mode]);
        }
        int firstOperand = fromSource ? 5 : 4;
        if (argc <= firstOperand) {
            return probe_usage(@"too few arguments for the selected compilation mode");
        }
        NSString *inputPath = @(argv[2]);
        NSString *functionName = @(argv[3]);
        NSUInteger count = (NSUInteger)(argc - firstOperand);

        NSMutableData *operandData = [NSMutableData dataWithLength:count * sizeof(uint32_t)];
        uint32_t *operands = (uint32_t *)operandData.mutableBytes;
        for (NSUInteger index = 0; index < count; index += 1) {
            const char *text = argv[firstOperand + (int)index];
            char *end = NULL;
            unsigned long long value = strtoull(text, &end, 16);
            if (end == text || *end != '\0' || value > 0xffffffffULL) {
                return probe_usage(
                    [NSString stringWithFormat:@"malformed hex operand: %s", text]);
            }
            operands[index] = (uint32_t)value;
        }

        // Every selection is resolved before the device is touched, so a
        // malformed invocation is a usage error rather than a dispatch that
        // reports results for a configuration nobody asked for.
        MTLCompileOptions *options = nil;
        NSString *archivePath = nil;
        if (fromSource) {
            NSDictionary<NSString *, NSString *> *selections = parse_option_list(@(argv[4]));
            if (selections == nil) {
                return probe_usage(@"malformed compile-option list");
            }
            options = [MTLCompileOptions new];
            for (NSString *key in selections) {
                if ([key isEqualToString:@"archive"]) {
                    archivePath = selections[key];
                    continue;
                }
                if (!apply_compile_option(options, key, selections[key])) {
                    return probe_usage([NSString
                        stringWithFormat:@"unrecognized compile option: %@=%@", key,
                                         selections[key]]);
                }
            }
            for (NSString *required in @[ @"math", @"fpfun", @"lang", @"opt" ]) {
                if (selections[required] == nil) {
                    return probe_usage([NSString
                        stringWithFormat:@"compile option %@ was not given", required]);
                }
            }
        }

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            fprintf(stderr, "numerical_probe_host: no default Metal device resolved\n");
            return kProbeExitNoDevice;
        }
        printf("device=%s\n", device.name.UTF8String);
        printf("registry-id=%llu\n", (unsigned long long)device.registryID);
        printf("compilation=%s\n", fromSource ? "source" : "library");

        NSError *error = nil;
        id<MTLLibrary> library = nil;
        if (fromSource) {
            NSString *source = [NSString stringWithContentsOfFile:inputPath
                                                         encoding:NSUTF8StringEncoding
                                                            error:&error];
            if (source == nil) {
                return probe_fail(@"source read", error);
            }
            library = [device newLibraryWithSource:source options:options error:&error];
            printf("applied=%s\n", applied_options(options).UTF8String);
        } else {
            library = [device newLibraryWithURL:[NSURL fileURLWithPath:inputPath] error:&error];
        }
        if (library == nil) {
            return probe_fail(fromSource ? @"runtime compilation" : @"library load", error);
        }
        id<MTLFunction> function = [library newFunctionWithName:functionName];
        if (function == nil) {
            fprintf(stderr, "numerical_probe_host: function lookup returned nil: %s\n",
                    functionName.UTF8String);
            return kProbeExitFailure;
        }
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (pipeline == nil) {
            return probe_fail(@"pipeline creation", error);
        }
        if (archivePath != nil) {
            NSString *unavailable = serialize_archive(device, function, archivePath);
            if (unavailable == nil) {
                printf("archive=%s\n", archivePath.UTF8String);
            } else {
                printf("archive-unavailable=%s\n", unavailable.UTF8String);
            }
        }
        id<MTLCommandQueue> queue = [device newCommandQueue];
        if (queue == nil) {
            fprintf(stderr, "numerical_probe_host: command queue creation returned nil\n");
            return kProbeExitFailure;
        }

        NSUInteger bytes = count * sizeof(uint32_t);
        id<MTLBuffer> input = [device newBufferWithBytes:operands
                                                  length:bytes
                                                 options:MTLResourceStorageModeShared];
        // The output buffer is seeded with a pattern no probe kernel can
        // produce, so a kernel that never wrote an element is distinguishable
        // from one that wrote a zero.
        NSMutableData *sentinelData = [NSMutableData dataWithLength:bytes];
        uint32_t *sentinel = (uint32_t *)sentinelData.mutableBytes;
        for (NSUInteger index = 0; index < count; index += 1) {
            sentinel[index] = 0xdeadbeefu;
        }
        id<MTLBuffer> output = [device newBufferWithBytes:sentinel
                                                   length:bytes
                                                  options:MTLResourceStorageModeShared];
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
