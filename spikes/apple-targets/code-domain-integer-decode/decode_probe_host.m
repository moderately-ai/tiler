// Dispatch host for the code-domain integer decode probe.
//
// This is a sibling of `numerical_probe_host.m` and deliberately not a widening
// of it. That host dispatches one buffer of same-width scalars over an
// eight-element operand vector and prints every returned pattern on stdout. The
// question here has a different shape: four buffers of three different element
// types, a 65,536-cell grid per case, and a comparison against an exact rational
// reference the harness computes rather than against a hand-stated expectation.
// Printing 65,536 patterns per case would put roughly a megabyte of stdout
// between the GPU and the record for every case; the results are written to a
// file instead and the harness reads the bytes back.
//
// It obtains each library one of the two ways Tiler's Metal story has, for the
// same reason the numerical host does:
//
//   library  load a metallib the harness already built offline through
//            `xcrun metal` and `xcrun metallib`
//   source   compile MSL in this process through `newLibraryWithSource:options:`
//            with an explicit `MTLCompileOptions`
//
// Both modes then take the identical path to the GPU, so a difference between
// them is a difference between the two compilers rather than between two
// dispatch procedures.
//
// # What this program does not interpret
//
// It never parses or formats a decimal literal, and it never reads a returned
// value as a number. The code and zero-point buffers arrive as raw bytes the
// harness wrote, the scale arrives as an eight-hex-digit `binary32` pattern, and
// the results leave as raw little-endian bytes. Every judgement about what those
// bits mean is made by the harness against an exact rational evaluation, so no
// rounding by this process's libc can stand between the GPU and the record.
//
// Exit codes are the harness's classification channel, matching the numerical
// host's so the two are read the same way:
//
//   0  every manifest entry dispatched and every result file was written
//   2  the arguments or the manifest were malformed (a harness defect, never a
//      skip)
//   3  no default Metal device resolved, the only self-skip this host reports
//   4  the toolchain and device resolved and something else failed, which is a
//      defect the harness must surface rather than skip
//
// Output is one `key=value` line per fact on stdout, in manifest order.

#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

#include <mach-o/dyld.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum {
    kDecodeExitOk = 0,
    kDecodeExitUsage = 2,
    kDecodeExitNoDevice = 3,
    kDecodeExitFailure = 4,
};

/// The pattern the output buffer is seeded with before every dispatch.
///
/// An element no kernel wrote must be distinguishable from one written as a
/// zero, and the decode writes a zero for every cell whose code equals its zero
/// point — a whole diagonal of the grid, and the exact cells the registered
/// exceptional contract is about. As a `binary32` this pattern is roughly
/// `-1.25e19`; the harness holds it to the stronger requirement that it is
/// absent from the exact reference of every case it dispatches, so a returned
/// sentinel can only mean an unwritten element.
static const uint32_t kDecodeSentinel = 0xdeadbeefu;

/// One manifest line, fully resolved before the device is touched.
///
/// ARC forbids an Objective-C pointer in a plain C struct, so this is a class
/// rather than the record it would otherwise be.
@interface DecodeEntry : NSObject
@property(nonatomic, copy) NSString *key;
@property(nonatomic, assign) BOOL fromSource;
@property(nonatomic, copy) NSString *path;
@property(nonatomic, copy) NSString *function;
@property(nonatomic, assign) uint32_t scale;
@property(nonatomic, copy) NSString *outputPath;
@property(nonatomic, strong) MTLCompileOptions *options;
@end

@implementation DecodeEntry
@end

static int decode_fail(NSString *stage, NSError *error) {
    fprintf(stderr, "decode_probe_host: %s failed: %s\n", stage.UTF8String,
            error == nil ? "no error object" : error.localizedDescription.UTF8String);
    return kDecodeExitFailure;
}

static int decode_usage(NSString *detail) {
    fprintf(stderr, "decode_probe_host: %s\n", detail.UTF8String);
    fprintf(stderr, "usage: decode_probe_host batch <manifest.tsv> <codes.bin> "
                    "<zero-points.bin>\n");
    fprintf(stderr, "       the two binaries are raw unsigned bytes of equal length, one element "
                    "per grid cell\n");
    fprintf(stderr, "       each manifest line is tab separated and is one of:\n");
    fprintf(stderr, "         <case-key>\tlibrary\t<metallib>\t<function>\t<scale-hex>\t<out>\n");
    fprintf(stderr,
            "         <case-key>\tsource\t<source.metal>\t<function>\t<scale-hex>\t<out>"
            "\t<options>\n");
    fprintf(stderr, "       <scale-hex> is exactly eight hexadecimal digits of a binary32 "
                    "pattern\n");
    fprintf(stderr, "       <options> is a comma-separated key=value list; every key and value "
                    "must be recognized:\n");
    fprintf(stderr, "         math=safe|relaxed|fast  fpfun=fast|precise  lang=3.1|3.2|4.0  "
                    "opt=default|size\n");
    return kDecodeExitUsage;
}

/// Splits a comma-separated `key=value` list, rejecting a malformed or repeated key.
///
/// Returns nil rather than a partial dictionary, for the reason the numerical
/// host states: a caller that accepted a partial parse would compile with some
/// selections defaulted and the record would name a configuration the library
/// was not built with.
static NSDictionary<NSString *, NSString *> *decode_option_list(NSString *text) {
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
/// `mathFloatingPointFunctions` defaults to `Fast` while the governed profile
/// pins `precise`, so a silently ignored selection would compile something the
/// record then misnames. Every key fails closed.
static BOOL decode_apply_option(MTLCompileOptions *options, NSString *key, NSString *value) {
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
        } else if ([value isEqualToString:@"4.0"]) {
            options.languageVersion = MTLLanguageVersion4_0;
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
static NSString *decode_applied_options(MTLCompileOptions *options) {
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
    } else if (options.languageVersion == MTLLanguageVersion4_0) {
        language = @"4.0";
    }
    NSString *optimization = @"unknown";
    switch (options.optimizationLevel) {
        case MTLLibraryOptimizationLevelDefault: optimization = @"default"; break;
        case MTLLibraryOptimizationLevelSize: optimization = @"size"; break;
    }
    return [NSString stringWithFormat:@"math=%@,fpfun=%@,lang=%@,opt=%@", math, functions, language,
                                      optimization];
}

/// The image-path substrings that name a Metal runtime compiler.
///
/// Both are listed for the reason `numerical_probe_host.m` records: on the
/// measured row `GPUCompiler.framework` is what dyld loads and no image whose
/// path contains `MTLCompiler` is loaded at all, so matching only the expected
/// name would report nothing.
static const char *const kDecodeCompilerImageMarkers[] = {"GPUCompiler", "MTLCompiler"};

static void decode_report_compiler_images(void) {
    uint32_t count = _dyld_image_count();
    size_t markers = sizeof(kDecodeCompilerImageMarkers) / sizeof(kDecodeCompilerImageMarkers[0]);
    for (uint32_t index = 0; index < count; index += 1) {
        const char *name = _dyld_get_image_name(index);
        if (name == NULL) {
            continue;
        }
        for (size_t marker = 0; marker < markers; marker += 1) {
            if (strstr(name, kDecodeCompilerImageMarkers[marker]) != NULL) {
                printf("runtime-compiler-image=%s\n", name);
                break;
            }
        }
    }
}

/// Parses exactly eight hexadecimal digits into a `binary32` pattern.
static BOOL decode_parse_pattern(NSString *text, uint32_t *out) {
    if (text.length != 8) {
        return NO;
    }
    const char *bytes = text.UTF8String;
    char *end = NULL;
    unsigned long long value = strtoull(bytes, &end, 16);
    if (end == bytes || *end != '\0' || value > UINT32_MAX) {
        return NO;
    }
    *out = (uint32_t)value;
    return YES;
}

/// Reads one manifest into fully resolved entries, or returns nil.
///
/// Every selection in every entry is resolved here, before the device is
/// touched, so a malformed manifest is a usage error rather than a run that
/// dispatches some cases and then reports a configuration nobody asked for.
static NSArray<DecodeEntry *> *decode_parse_manifest(NSString *text, NSString **complaint) {
    NSMutableArray<DecodeEntry *> *entries = [NSMutableArray array];
    NSMutableSet<NSString *> *seen = [NSMutableSet set];
    NSMutableSet<NSString *> *outputs = [NSMutableSet set];
    NSUInteger number = 0;
    for (NSString *line in [text componentsSeparatedByString:@"\n"]) {
        number += 1;
        if (line.length == 0) {
            continue;
        }
        NSArray<NSString *> *fields = [line componentsSeparatedByString:@"\t"];
        if (fields.count < 6) {
            *complaint = [NSString stringWithFormat:@"manifest line %lu has %lu fields",
                                                    (unsigned long)number,
                                                    (unsigned long)fields.count];
            return nil;
        }
        DecodeEntry *entry = [DecodeEntry new];
        entry.key = fields[0];
        entry.path = fields[2];
        entry.function = fields[3];
        entry.outputPath = fields[5];
        uint32_t scale = 0;
        if (!decode_parse_pattern(fields[4], &scale)) {
            *complaint = [NSString stringWithFormat:@"malformed scale pattern %@ for %@", fields[4],
                                                    entry.key];
            return nil;
        }
        entry.scale = scale;
        if ([seen containsObject:entry.key]) {
            *complaint = [NSString stringWithFormat:@"manifest repeats case key %@", entry.key];
            return nil;
        }
        [seen addObject:entry.key];
        // Two entries sharing an output path would leave one case reading the
        // other's bytes, which is exactly the substitution the whole comparison
        // is meant to detect and would be invisible in the record.
        if ([outputs containsObject:entry.outputPath]) {
            *complaint = [NSString stringWithFormat:@"manifest repeats output path %@",
                                                    entry.outputPath];
            return nil;
        }
        [outputs addObject:entry.outputPath];
        if ([fields[1] isEqualToString:@"library"]) {
            entry.fromSource = NO;
            if (fields.count != 6) {
                *complaint = [NSString stringWithFormat:@"library entry %@ has extra fields",
                                                        entry.key];
                return nil;
            }
        } else if ([fields[1] isEqualToString:@"source"]) {
            entry.fromSource = YES;
            if (fields.count != 7) {
                *complaint = [NSString stringWithFormat:@"source entry %@ needs an option list",
                                                        entry.key];
                return nil;
            }
            NSDictionary<NSString *, NSString *> *selections = decode_option_list(fields[6]);
            if (selections == nil) {
                *complaint = [NSString stringWithFormat:@"malformed option list for %@", entry.key];
                return nil;
            }
            entry.options = [MTLCompileOptions new];
            for (NSString *key in selections) {
                if (!decode_apply_option(entry.options, key, selections[key])) {
                    *complaint = [NSString stringWithFormat:@"unrecognized compile option %@=%@",
                                                            key, selections[key]];
                    return nil;
                }
            }
            for (NSString *required in @[ @"math", @"fpfun", @"lang", @"opt" ]) {
                if (selections[required] == nil) {
                    *complaint = [NSString stringWithFormat:@"compile option %@ was not given for %@",
                                                            required, entry.key];
                    return nil;
                }
            }
        } else {
            *complaint = [NSString stringWithFormat:@"unknown compilation mode: %@", fields[1]];
            return nil;
        }
        [entries addObject:entry];
    }
    if (entries.count == 0) {
        *complaint = @"the manifest contained no entries";
        return nil;
    }
    return entries;
}

/// Obtains one entry's library, dispatches the whole grid, and writes the results.
///
/// Each entry builds its own library, pipeline, buffers, and command buffer, so
/// the only state shared across a manifest is the device and the queue.
static int decode_run_entry(id<MTLDevice> device, id<MTLCommandQueue> queue, DecodeEntry *entry,
                            NSData *codes, NSData *zeroPoints) {
    const NSUInteger cells = codes.length;
    printf("case=%s\n", entry.key.UTF8String);
    printf("compilation=%s\n", entry.fromSource ? "source" : "library");
    printf("scale=%08x\n", entry.scale);

    NSError *error = nil;
    id<MTLLibrary> library = nil;
    if (entry.fromSource) {
        NSString *source = [NSString stringWithContentsOfFile:entry.path
                                                     encoding:NSUTF8StringEncoding
                                                        error:&error];
        if (source == nil) {
            return decode_fail(@"source read", error);
        }
        library = [device newLibraryWithSource:source options:entry.options error:&error];
        printf("applied=%s\n", decode_applied_options(entry.options).UTF8String);
    } else {
        library = [device newLibraryWithURL:[NSURL fileURLWithPath:entry.path] error:&error];
    }
    if (library == nil) {
        return decode_fail(entry.fromSource ? @"runtime compilation" : @"library load", error);
    }
    id<MTLFunction> function = [library newFunctionWithName:entry.function];
    if (function == nil) {
        fprintf(stderr, "decode_probe_host: function lookup returned nil: %s\n",
                entry.function.UTF8String);
        return kDecodeExitFailure;
    }
    id<MTLComputePipelineState> pipeline =
        [device newComputePipelineStateWithFunction:function error:&error];
    if (pipeline == nil) {
        return decode_fail(@"pipeline creation", error);
    }

    id<MTLBuffer> codeBuffer = [device newBufferWithBytes:codes.bytes
                                                   length:cells
                                                  options:MTLResourceStorageModeShared];
    id<MTLBuffer> zeroBuffer = [device newBufferWithBytes:zeroPoints.bytes
                                                   length:cells
                                                  options:MTLResourceStorageModeShared];
    uint32_t scalePattern = entry.scale;
    id<MTLBuffer> scaleBuffer = [device newBufferWithBytes:&scalePattern
                                                    length:sizeof(scalePattern)
                                                   options:MTLResourceStorageModeShared];
    NSMutableData *seed = [NSMutableData dataWithLength:cells * sizeof(uint32_t)];
    uint32_t *seeded = (uint32_t *)seed.mutableBytes;
    for (NSUInteger index = 0; index < cells; index += 1) {
        seeded[index] = kDecodeSentinel;
    }
    id<MTLBuffer> output = [device newBufferWithBytes:seed.bytes
                                               length:seed.length
                                              options:MTLResourceStorageModeShared];
    if (codeBuffer == nil || zeroBuffer == nil || scaleBuffer == nil || output == nil) {
        fprintf(stderr, "decode_probe_host: shared buffer allocation returned nil\n");
        return kDecodeExitFailure;
    }

    id<MTLCommandBuffer> commands = [queue commandBuffer];
    id<MTLComputeCommandEncoder> encoder = [commands computeCommandEncoder];
    if (commands == nil || encoder == nil) {
        fprintf(stderr, "decode_probe_host: command encoding returned nil\n");
        return kDecodeExitFailure;
    }
    [encoder setComputePipelineState:pipeline];
    [encoder setBuffer:codeBuffer offset:0 atIndex:0];
    [encoder setBuffer:zeroBuffer offset:0 atIndex:1];
    [encoder setBuffer:scaleBuffer offset:0 atIndex:2];
    [encoder setBuffer:output offset:0 atIndex:3];
    NSUInteger width = pipeline.maxTotalThreadsPerThreadgroup;
    if (width > cells) {
        width = cells;
    }
    [encoder dispatchThreads:MTLSizeMake(cells, 1, 1) threadsPerThreadgroup:MTLSizeMake(width, 1, 1)];
    [encoder endEncoding];
    [commands commit];
    [commands waitUntilCompleted];

    // AGENTS.md requires exact command-buffer terminal success before host
    // validation readback. A buffer that errored or was not completed makes the
    // shared allocation's contents meaningless, not merely suspect.
    if (commands.status != MTLCommandBufferStatusCompleted) {
        fprintf(stderr, "decode_probe_host: command buffer terminal status was %ld\n",
                (long)commands.status);
        return decode_fail(@"command buffer", commands.error);
    }
    if (commands.error != nil) {
        return decode_fail(@"command buffer", commands.error);
    }

    const uint32_t *results = (const uint32_t *)output.contents;
    for (NSUInteger index = 0; index < cells; index += 1) {
        if (results[index] == kDecodeSentinel) {
            fprintf(stderr, "decode_probe_host: %s cell %lu was never written\n",
                    entry.key.UTF8String, (unsigned long)index);
            return kDecodeExitFailure;
        }
    }
    NSData *bytes = [NSData dataWithBytes:output.contents length:cells * sizeof(uint32_t)];
    if (![bytes writeToFile:entry.outputPath options:NSDataWritingAtomic error:&error]) {
        return decode_fail(@"result write", error);
    }
    printf("output=%s\n", entry.outputPath.UTF8String);
    printf("cells=%lu\n", (unsigned long)cells);
    return kDecodeExitOk;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 5 || strcmp(argv[1], "batch") != 0) {
            return decode_usage(@"expected: batch <manifest.tsv> <codes.bin> <zero-points.bin>");
        }

        NSError *error = nil;
        NSData *codes = [NSData dataWithContentsOfFile:@(argv[3]) options:0 error:&error];
        if (codes == nil) {
            return decode_usage([NSString stringWithFormat:@"codes unreadable: %@",
                                                           error.localizedDescription]);
        }
        NSData *zeroPoints = [NSData dataWithContentsOfFile:@(argv[4]) options:0 error:&error];
        if (zeroPoints == nil) {
            return decode_usage([NSString stringWithFormat:@"zero points unreadable: %@",
                                                           error.localizedDescription]);
        }
        // Resolved before the device is touched. Unequal lengths would dispatch
        // one thread per code over a shorter zero-point buffer, which is a read
        // past the end rather than a measurement.
        if (codes.length == 0 || codes.length != zeroPoints.length) {
            return decode_usage([NSString
                stringWithFormat:@"codes and zero points are %lu and %lu bytes",
                                 (unsigned long)codes.length, (unsigned long)zeroPoints.length]);
        }

        NSString *manifest = [NSString stringWithContentsOfFile:@(argv[2])
                                                       encoding:NSUTF8StringEncoding
                                                          error:&error];
        if (manifest == nil) {
            return decode_usage([NSString stringWithFormat:@"manifest unreadable: %@",
                                                           error.localizedDescription]);
        }
        NSString *complaint = nil;
        NSArray<DecodeEntry *> *entries = decode_parse_manifest(manifest, &complaint);
        if (entries == nil) {
            return decode_usage(complaint);
        }

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            fprintf(stderr, "decode_probe_host: no default Metal device resolved\n");
            return kDecodeExitNoDevice;
        }
        printf("device=%s\n", device.name.UTF8String);
        printf("registry-id=%llu\n", (unsigned long long)device.registryID);
        printf("gpu-family-apple9=%s\n",
               [device supportsFamily:MTLGPUFamilyApple9] ? "supported" : "unsupported");

        id<MTLCommandQueue> queue = [device newCommandQueue];
        if (queue == nil) {
            fprintf(stderr, "decode_probe_host: command queue creation returned nil\n");
            return kDecodeExitFailure;
        }
        for (DecodeEntry *entry in entries) {
            int status = decode_run_entry(device, queue, entry, codes, zeroPoints);
            if (status != kDecodeExitOk) {
                return status;
            }
        }
        decode_report_compiler_images();
        return kDecodeExitOk;
    }
}
