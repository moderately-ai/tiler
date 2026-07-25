// Dispatch host for the Apple numerical-behaviour probe.
//
// The probe's compile-side observations can be read out of emitted LLVM IR, but
// the returned bit pattern of a subnormal operand cannot: it is a property of
// the GPU executing the compiled kernel. This host is the smallest program that
// obtains libraries, dispatches one thread per element over a shared buffer of
// caller-supplied bit patterns, and prints what came back.
//
// It obtains each library one of two ways, because Tiler's Metal story has two
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
// # Why one invocation dispatches a whole manifest
//
// This program is built once per artifact family and run inside that family's
// execution environment. For the iOS Simulator that means `simctl spawn`, whose
// per-launch cost is about 290 ms on the measured host — measured against
// `/usr/bin/true`, so it is launch overhead and not Metal work. One launch per
// case would put roughly 13 s of pure process spawning into every repository
// gate run for that family alone. A manifest amortizes it to one launch, and
// each entry still gets its own library, pipeline, buffers, command buffer, and
// terminal-status check, so nothing about the per-case procedure is shared but
// the device and the queue.
//
// It reads every scalar as a raw unsigned integer of the element's own width on
// both sides deliberately. Parsing or formatting a decimal literal anywhere in
// the path would let the host's own libc rounding stand between the GPU and the
// recorded measurement, and a `half` would additionally be rounded twice.
//
// # Why an entry names its dtype
//
// The element width decides the buffer size, which operand vector the entry is
// dispatched over, the sentinel an unwritten element is distinguishable by, and
// the width a result is printed at. None of that is recoverable from a metallib
// or from MSL source this program does not parse, so a manifest line states it
// and an unknown or absent dtype is a usage error. A host that guessed would
// read a correctly dispatched 16-bit kernel back as half as many 32-bit values
// and print eight-digit patterns that no kernel produced.
//
// Exit codes are the harness's classification channel:
//
//   0  every manifest entry dispatched and every result line was printed
//   2  the arguments or the manifest were malformed (a harness defect, never a
//      skip)
//   3  no default Metal device resolved, which is the device-side analogue of
//      the toolchain-unavailable skip and the only self-skip this host reports
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
    kProbeExitOk = 0,
    kProbeExitUsage = 2,
    kProbeExitNoDevice = 3,
    kProbeExitFailure = 4,
};

/// One scalar format this host can dispatch, and everything that differs about it.
///
/// `sentinel` seeds the output buffer so an element no kernel wrote is
/// distinguishable from one written as a zero. It has to be representable in the
/// element width, which is why it belongs to the dtype rather than being one
/// constant; the harness holds every value here to the stronger requirement that
/// no probe kernel can produce it.
typedef struct {
    const char *name;
    uint32_t bits;
    uint32_t sentinel;
} ProbeDtype;

static const ProbeDtype kProbeDtypes[] = {
    {"f32", 32, 0xdeadbeefu},
    {"f16", 16, 0x0000deadu},
};

static const ProbeDtype *probe_dtype_named(NSString *name) {
    size_t count = sizeof(kProbeDtypes) / sizeof(kProbeDtypes[0]);
    for (size_t index = 0; index < count; index += 1) {
        if ([name isEqualToString:@(kProbeDtypes[index].name)]) {
            return &kProbeDtypes[index];
        }
    }
    return NULL;
}

/// One dtype's operand vector, parsed once and shared by every entry that names it.
@interface ProbeOperands : NSObject
@property(nonatomic, assign) const ProbeDtype *dtype;
@property(nonatomic, strong) NSMutableData *elements;
@property(nonatomic, assign) NSUInteger count;
@end

@implementation ProbeOperands
@end

/// One manifest line, fully resolved before the device is touched.
///
/// ARC forbids an Objective-C pointer in a plain C struct, so this is a class
/// rather than the record it would otherwise be.
@interface ProbeEntry : NSObject
@property(nonatomic, copy) NSString *key;
@property(nonatomic, assign) const ProbeDtype *dtype;
@property(nonatomic, assign) BOOL fromSource;
@property(nonatomic, copy) NSString *path;
@property(nonatomic, copy) NSString *function;
@property(nonatomic, strong) MTLCompileOptions *options;
@property(nonatomic, copy) NSString *archivePath;
@end

@implementation ProbeEntry
@end

/// Whether the element accessors below implement this width.
///
/// The dtype table is the only producer of a width, so this guards adding a row
/// to it rather than guarding input. A dtype whose width had no accessor would
/// fall through to the 32-bit path and silently misreport every element of every
/// case measured for it, which is precisely the class of failure this program's
/// sentinel and terminal-status checks exist to make impossible. `main`
/// validates the whole table once before anything is dispatched.
static BOOL probe_width_supported(uint32_t bits) {
    return bits == 16 || bits == 32;
}

/// The largest pattern an element of `bits` width can hold.
static uint32_t probe_width_limit(uint32_t bits) {
    return bits >= 32 ? UINT32_MAX : (uint32_t)((1u << bits) - 1u);
}

/// Reads one element of `bits` width out of a packed buffer.
static uint32_t probe_element(const void *base, NSUInteger index, uint32_t bits) {
    if (bits == 16) {
        return ((const uint16_t *)base)[index];
    }
    return ((const uint32_t *)base)[index];
}

/// Writes one element of `bits` width into a packed buffer.
static void probe_set_element(void *base, NSUInteger index, uint32_t bits, uint32_t value) {
    if (bits == 16) {
        ((uint16_t *)base)[index] = (uint16_t)value;
        return;
    }
    ((uint32_t *)base)[index] = value;
}

static int probe_fail(NSString *stage, NSError *error) {
    fprintf(stderr, "numerical_probe_host: %s failed: %s\n", stage.UTF8String,
            error == nil ? "no error object" : error.localizedDescription.UTF8String);
    return kProbeExitFailure;
}

static int probe_usage(NSString *detail) {
    fprintf(stderr, "numerical_probe_host: %s\n", detail.UTF8String);
    fprintf(stderr, "usage: numerical_probe_host batch <manifest.tsv> "
                    "<dtype>=<hex-operand>[,<hex-operand>...]...\n");
    fprintf(stderr, "       one operand group per dtype; every dtype a manifest entry names "
                    "must have one\n");
    fprintf(stderr, "       each manifest line is tab separated and is one of:\n");
    fprintf(stderr, "         <case-key>\t<dtype>\tlibrary\t<metallib>\t<function>\n");
    fprintf(stderr,
            "         <case-key>\t<dtype>\tsource\t<source.metal>\t<function>\t<options>\n");
    fprintf(stderr, "       <dtype> is one of: f32 f16\n");
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
/// A *returnable* failure, that is. In the iOS Simulator runtime this call site
/// does not return at all: `+[MTLLoader sliceIDForDevice:legacyDriverVersion:
/// airntDriverVersion:]` fails an assertion and aborts the process. That is why
/// the harness probes archive support with a one-entry manifest of its own
/// before requesting an archive in a manifest that carries measurements.
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

/// The image-path substrings that name a Metal runtime compiler.
///
/// Both are listed because the framework carrying the compiler is not stable
/// across OS versions. On the measured row `GPUCompiler.framework` is what dyld
/// loads into a process that compiles MSL, and no image whose path contains
/// `MTLCompiler` is loaded at all even though that framework is present on disk;
/// matching only the expected name would have reported nothing and left the
/// runtime compiler unidentified.
static const char *const kCompilerImageMarkers[] = {"GPUCompiler", "MTLCompiler"};

/// Prints the resolved path of every loaded image that names the runtime compiler.
///
/// This is how a family's *runtime* compiler is identified when the binary
/// archive that carries its version string cannot be written. dyld reports the
/// path it actually loaded, so inside a simulator it names the runtime root's
/// framework and on the host it names the system one, and the two are
/// distinguishable without trusting either to be the one a document expects.
static void report_compiler_images(void) {
    uint32_t count = _dyld_image_count();
    size_t markers = sizeof(kCompilerImageMarkers) / sizeof(kCompilerImageMarkers[0]);
    for (uint32_t index = 0; index < count; index += 1) {
        const char *name = _dyld_get_image_name(index);
        if (name == NULL) {
            continue;
        }
        for (size_t marker = 0; marker < markers; marker += 1) {
            if (strstr(name, kCompilerImageMarkers[marker]) != NULL) {
                printf("runtime-compiler-image=%s\n", name);
                break;
            }
        }
    }
}

/// Reads one manifest into fully resolved entries, or returns nil.
///
/// Every selection in every entry is resolved here, before the device is
/// touched, so a malformed manifest is a usage error rather than a run that
/// dispatches some cases and then reports a configuration nobody asked for.
static NSArray<ProbeEntry *> *parse_manifest(NSString *text, NSString **complaint) {
    NSMutableArray<ProbeEntry *> *entries = [NSMutableArray array];
    NSMutableSet<NSString *> *seen = [NSMutableSet set];
    NSUInteger number = 0;
    for (NSString *line in [text componentsSeparatedByString:@"\n"]) {
        number += 1;
        if (line.length == 0) {
            continue;
        }
        NSArray<NSString *> *fields = [line componentsSeparatedByString:@"\t"];
        if (fields.count < 5) {
            *complaint = [NSString stringWithFormat:@"manifest line %lu has %lu fields",
                                                    (unsigned long)number,
                                                    (unsigned long)fields.count];
            return nil;
        }
        ProbeEntry *entry = [ProbeEntry new];
        entry.key = fields[0];
        entry.dtype = probe_dtype_named(fields[1]);
        entry.path = fields[3];
        entry.function = fields[4];
        if (entry.dtype == NULL) {
            *complaint = [NSString stringWithFormat:@"unknown dtype %@ for %@", fields[1],
                                                    entry.key];
            return nil;
        }
        if ([seen containsObject:entry.key]) {
            *complaint = [NSString stringWithFormat:@"manifest repeats case key %@", entry.key];
            return nil;
        }
        [seen addObject:entry.key];
        if ([fields[2] isEqualToString:@"library"]) {
            entry.fromSource = NO;
            if (fields.count != 5) {
                *complaint = [NSString stringWithFormat:@"library entry %@ has extra fields",
                                                        entry.key];
                return nil;
            }
        } else if ([fields[2] isEqualToString:@"source"]) {
            entry.fromSource = YES;
            if (fields.count != 6) {
                *complaint = [NSString stringWithFormat:@"source entry %@ needs an option list",
                                                        entry.key];
                return nil;
            }
            NSDictionary<NSString *, NSString *> *selections = parse_option_list(fields[5]);
            if (selections == nil) {
                *complaint = [NSString stringWithFormat:@"malformed option list for %@", entry.key];
                return nil;
            }
            entry.options = [MTLCompileOptions new];
            for (NSString *key in selections) {
                if ([key isEqualToString:@"archive"]) {
                    entry.archivePath = selections[key];
                    continue;
                }
                if (!apply_compile_option(entry.options, key, selections[key])) {
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
            *complaint = [NSString stringWithFormat:@"unknown compilation mode: %@", fields[2]];
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

/// Obtains one entry's library, dispatches it, and prints what came back.
///
/// Each entry builds its own library, pipeline, buffers, and command buffer, so
/// the only state shared across a manifest is the device and the queue.
static int run_entry(id<MTLDevice> device, id<MTLCommandQueue> queue, ProbeEntry *entry,
                     ProbeOperands *operands) {
    const uint32_t bits = entry.dtype->bits;
    const NSUInteger count = operands.count;
    printf("case=%s\n", entry.key.UTF8String);
    printf("compilation=%s\n", entry.fromSource ? "source" : "library");
    printf("dtype=%s\n", entry.dtype->name);

    NSError *error = nil;
    id<MTLLibrary> library = nil;
    if (entry.fromSource) {
        NSString *source = [NSString stringWithContentsOfFile:entry.path
                                                     encoding:NSUTF8StringEncoding
                                                        error:&error];
        if (source == nil) {
            return probe_fail(@"source read", error);
        }
        library = [device newLibraryWithSource:source options:entry.options error:&error];
        printf("applied=%s\n", applied_options(entry.options).UTF8String);
    } else {
        library = [device newLibraryWithURL:[NSURL fileURLWithPath:entry.path] error:&error];
    }
    if (library == nil) {
        return probe_fail(entry.fromSource ? @"runtime compilation" : @"library load", error);
    }
    id<MTLFunction> function = [library newFunctionWithName:entry.function];
    if (function == nil) {
        fprintf(stderr, "numerical_probe_host: function lookup returned nil: %s\n",
                entry.function.UTF8String);
        return kProbeExitFailure;
    }
    id<MTLComputePipelineState> pipeline =
        [device newComputePipelineStateWithFunction:function error:&error];
    if (pipeline == nil) {
        return probe_fail(@"pipeline creation", error);
    }
    if (entry.archivePath != nil) {
        NSString *unavailable = serialize_archive(device, function, entry.archivePath);
        if (unavailable == nil) {
            printf("archive=%s\n", entry.archivePath.UTF8String);
        } else {
            printf("archive-unavailable=%s\n", unavailable.UTF8String);
        }
    }

    NSUInteger bytes = count * (bits / 8);
    id<MTLBuffer> input = [device newBufferWithBytes:operands.elements.bytes
                                              length:bytes
                                             options:MTLResourceStorageModeShared];
    // The output buffer is seeded with a pattern no probe kernel can produce,
    // so a kernel that never wrote an element is distinguishable from one that
    // wrote a zero. The pattern belongs to the dtype because it has to fit the
    // element width; truncating one wide pattern could land on a value a kernel
    // does produce.
    NSMutableData *sentinelData = [NSMutableData dataWithLength:bytes];
    void *sentinel = sentinelData.mutableBytes;
    for (NSUInteger index = 0; index < count; index += 1) {
        probe_set_element(sentinel, index, bits, entry.dtype->sentinel);
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
    [encoder dispatchThreads:MTLSizeMake(count, 1, 1) threadsPerThreadgroup:MTLSizeMake(width, 1, 1)];
    [encoder endEncoding];
    [commands commit];
    [commands waitUntilCompleted];

    // AGENTS.md requires exact command-buffer terminal success before host
    // validation readback. A buffer that errored or was not completed makes the
    // shared allocation's contents meaningless, not merely suspect.
    if (commands.status != MTLCommandBufferStatusCompleted) {
        fprintf(stderr, "numerical_probe_host: command buffer terminal status was %ld\n",
                (long)commands.status);
        return probe_fail(@"command buffer", commands.error);
    }
    if (commands.error != nil) {
        return probe_fail(@"command buffer", commands.error);
    }

    const void *results = output.contents;
    for (NSUInteger index = 0; index < count; index += 1) {
        uint32_t value = probe_element(results, index, bits);
        if (value == entry.dtype->sentinel) {
            fprintf(stderr, "numerical_probe_host: %s element %lu was never written\n",
                    entry.key.UTF8String, (unsigned long)index);
            return kProbeExitFailure;
        }
        // Printed at the element's own width, so a 16-bit result cannot be read
        // back as a zero-extended 32-bit one.
        printf("result=%0*x\n", (int)(bits / 4), value);
    }
    return kProbeExitOk;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc < 4 || strcmp(argv[1], "batch") != 0) {
            return probe_usage(@"expected: batch <manifest.tsv> <dtype>=<hex-operand>,...");
        }

        // The dtype table is checked against the accessors once, before anything
        // is parsed or dispatched. A row whose width they do not implement, or
        // whose sentinel does not fit its own element, would not fail loudly
        // anywhere later: the first reads back every result through the wrong
        // path and the second makes an unwritten element indistinguishable from
        // a written one.
        size_t declared = sizeof(kProbeDtypes) / sizeof(kProbeDtypes[0]);
        for (size_t index = 0; index < declared; index += 1) {
            const ProbeDtype *dtype = &kProbeDtypes[index];
            if (!probe_width_supported(dtype->bits)
                || dtype->sentinel > probe_width_limit(dtype->bits)) {
                return probe_usage([NSString
                    stringWithFormat:@"dtype %s declares a %u-bit element or a sentinel this host "
                                     @"does not implement",
                                     dtype->name, dtype->bits]);
            }
        }

        // Every operand group is parsed into elements of its own dtype's width
        // before anything is dispatched, so a pattern that does not fit is a
        // usage error rather than a value silently truncated into a buffer.
        NSMutableDictionary<NSString *, ProbeOperands *> *vectors = [NSMutableDictionary
            dictionary];
        for (int argument = 3; argument < argc; argument += 1) {
            NSString *group = @(argv[argument]);
            NSRange separator = [group rangeOfString:@"="];
            if (separator.location == NSNotFound || separator.location == 0) {
                return probe_usage([NSString stringWithFormat:@"malformed operand group: %@",
                                                              group]);
            }
            NSString *name = [group substringToIndex:separator.location];
            const ProbeDtype *dtype = probe_dtype_named(name);
            if (dtype == NULL) {
                return probe_usage([NSString stringWithFormat:@"unknown operand dtype: %@", name]);
            }
            if (vectors[name] != nil) {
                return probe_usage([NSString stringWithFormat:@"repeated operand group: %@",
                                                              name]);
            }
            NSArray<NSString *> *patterns = [[group substringFromIndex:separator.location + 1]
                componentsSeparatedByString:@","];
            ProbeOperands *parsed = [ProbeOperands new];
            parsed.dtype = dtype;
            parsed.count = patterns.count;
            parsed.elements = [NSMutableData dataWithLength:patterns.count * (dtype->bits / 8)];
            unsigned long long limit = probe_width_limit(dtype->bits);
            for (NSUInteger index = 0; index < patterns.count; index += 1) {
                const char *text = patterns[index].UTF8String;
                char *end = NULL;
                unsigned long long value = strtoull(text, &end, 16);
                if (end == text || *end != '\0' || value > limit) {
                    return probe_usage([NSString stringWithFormat:@"malformed %@ operand: %s",
                                                                  name, text]);
                }
                probe_set_element(parsed.elements.mutableBytes, index, dtype->bits,
                                  (uint32_t)value);
            }
            vectors[name] = parsed;
        }

        NSError *error = nil;
        NSString *manifest = [NSString stringWithContentsOfFile:@(argv[2])
                                                       encoding:NSUTF8StringEncoding
                                                          error:&error];
        if (manifest == nil) {
            return probe_usage([NSString stringWithFormat:@"manifest unreadable: %@",
                                                          error.localizedDescription]);
        }
        NSString *complaint = nil;
        NSArray<ProbeEntry *> *entries = parse_manifest(manifest, &complaint);
        if (entries == nil) {
            return probe_usage(complaint);
        }
        // Resolved before the device is touched, for the same reason the compile
        // options are: an entry whose dtype has no operand group must be a usage
        // error and not a run that dispatches some cases and then stops.
        for (ProbeEntry *entry in entries) {
            if (vectors[@(entry.dtype->name)] == nil) {
                return probe_usage([NSString stringWithFormat:@"no %s operand group for %@",
                                                              entry.dtype->name, entry.key]);
            }
        }

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            fprintf(stderr, "numerical_probe_host: no default Metal device resolved\n");
            return kProbeExitNoDevice;
        }
        printf("device=%s\n", device.name.UTF8String);
        printf("registry-id=%llu\n", (unsigned long long)device.registryID);

        id<MTLCommandQueue> queue = [device newCommandQueue];
        if (queue == nil) {
            fprintf(stderr, "numerical_probe_host: command queue creation returned nil\n");
            return kProbeExitFailure;
        }
        for (ProbeEntry *entry in entries) {
            int status = run_entry(device, queue, entry, vectors[@(entry.dtype->name)]);
            if (status != kProbeExitOk) {
                return status;
            }
        }
        report_compiler_images();
        return kProbeExitOk;
    }
}
