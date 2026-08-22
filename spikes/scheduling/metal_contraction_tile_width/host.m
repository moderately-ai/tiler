// Dispatch host for the contraction tile-width sweep.
//
// It reads a manifest the Python driver writes, runs one line at a time in
// manifest order, and prints `key=value` rows on stdout. Manifest order is what
// makes the timing leg A/B interleaved: the driver emits round-robin lines and
// this host never reorders them.
//
// It is a trimmed sibling of `../metal_contraction_vertical/host.m`. The
// MetalPerformanceShaders path and the split/simdgroup dispatch shapes are gone
// because this sweep has no opaque candidate and no split kernel; the tile
// dimensions arrive per case from the manifest instead of being compiled in.
// The two disciplines a reader of the retained record has to be able to check
// are unchanged and are restated here:
//
//   1. The output allocation is seeded with a finite pattern no admitted case
//      can produce (-3.0e38f; every case's exact result is bounded by 768 in
//      magnitude), so "never written" is distinguishable from "wrote zero".
//   2. `MTLCommandBufferStatusCompleted` and a nil `commandBuffer.error` are
//      required before any readback, per the repository's rule that exact
//      command-buffer terminal success precedes host validation readback.
//
// Operands are generated in-process from a SplitMix64 stream so that a 622 MB
// weight matrix never touches the filesystem, and the host prints the SHA-256
// of the exact operand bytes it generated. The generator is byte-identical to
// the probe's, which is what lets a reader hold the two records' operands
// against each other.

#import <CommonCrypto/CommonDigest.h>
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

#include <stdint.h>
#include <string.h>

static const float kOutputSeed = -3.0e38f;

typedef struct {
    uint32_t m_extent;
    uint32_t n_extent;
    uint32_t k_extent;
    uint32_t split;
} ContractionDims;

// --------------------------------------------------------------------------
// Operand generation. Mirrored exactly by `tile_width_sweep.py`, and identical
// to `../metal_contraction_vertical/host.m`.
// --------------------------------------------------------------------------

static uint64_t splitmix64(uint64_t x) {
    x += 0x9E3779B97F4A7C15ULL;
    uint64_t z = x;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}

// Yields a value of the form `m * 2^-24` with `m` an integer in
// [-2^23, 2^23). Every such value is exactly representable in binary32, so the
// operands themselves introduce no rounding and every difference a case reports
// is a difference in how the contraction was evaluated.
static float prng_value(uint64_t seed, uint64_t index) {
    const uint64_t bits = splitmix64(seed + index * 0x2545F4914F6CDD1DULL);
    const int32_t magnitude = (int32_t)((bits >> 40) & 0xFFFFFFu) - 8388608;
    return (float)magnitude * (1.0f / 16777216.0f);
}

static void fill_prng(float *destination, uint64_t count, uint64_t seed) {
    for (uint64_t index = 0; index < count; ++index) {
        destination[index] = prng_value(seed, index);
    }
}

static NSString *hex_digest(const void *bytes, size_t length) {
    unsigned char digest[CC_SHA256_DIGEST_LENGTH];
    CC_SHA256(bytes, (CC_LONG)length, digest);
    NSMutableString *out = [NSMutableString stringWithCapacity:CC_SHA256_DIGEST_LENGTH * 2];
    for (int i = 0; i < CC_SHA256_DIGEST_LENGTH; ++i) {
        [out appendFormat:@"%02x", digest[i]];
    }
    return out;
}

@interface OperandSet : NSObject
@property(nonatomic, strong) id<MTLBuffer> left;
@property(nonatomic, strong) id<MTLBuffer> right;
@property(nonatomic, copy) NSString *leftDigest;
@property(nonatomic, copy) NSString *rightDigest;
@end

@implementation OperandSet
@end

static void die(NSString *message) {
    fprintf(stderr, "tile-width host: %s\n", message.UTF8String);
    exit(2);
}

static NSArray<NSString *> *read_lines(NSString *path) {
    NSError *error = nil;
    NSString *text = [NSString stringWithContentsOfFile:path encoding:NSUTF8StringEncoding error:&error];
    if (text == nil) {
        die([NSString stringWithFormat:@"cannot read manifest %@: %@", path, error]);
    }
    NSMutableArray<NSString *> *lines = [NSMutableArray array];
    for (NSString *line in [text componentsSeparatedByString:@"\n"]) {
        NSString *trimmed = [line stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
        if (trimmed.length > 0 && ![trimmed hasPrefix:@"#"]) {
            [lines addObject:trimmed];
        }
    }
    return lines;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 4) {
            die(@"usage: host <metallib> <manifest> <work-dir>");
        }
        NSString *metallibPath = [NSString stringWithUTF8String:argv[1]];
        NSString *manifestPath = [NSString stringWithUTF8String:argv[2]];
        NSString *workDir = [NSString stringWithUTF8String:argv[3]];

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            die(@"no default Metal device");
        }

        // Stop conditions the protocol freezes: the device must be Apple9 and
        // must admit a 1024-thread threadgroup, or the W = 32 arm is not
        // structurally admissible and the sweep is not the frozen one.
        if (![device supportsFamily:MTLGPUFamilyApple9]) {
            die(@"default device does not report supportsFamily(Apple9)");
        }
        if (device.maxThreadsPerThreadgroup.width < 1024) {
            die(@"device maxThreadsPerThreadgroup.width < 1024");
        }

        printf("environment.device=%s\n", device.name.UTF8String);
        printf("environment.device_apple9=supported\n");
        printf("environment.device_max_threads_per_threadgroup=%lu\n",
               (unsigned long)device.maxThreadsPerThreadgroup.width);
        printf("environment.device_max_threadgroup_memory=%lu\n",
               (unsigned long)device.maxThreadgroupMemoryLength);
        printf("environment.device_max_buffer_length=%llu\n",
               (unsigned long long)device.maxBufferLength);

        NSError *error = nil;
        NSData *libraryBytes = [NSData dataWithContentsOfFile:metallibPath options:0 error:&error];
        if (libraryBytes == nil) {
            die([NSString stringWithFormat:@"cannot read metallib: %@", error]);
        }
        dispatch_data_t libraryData = dispatch_data_create(libraryBytes.bytes, libraryBytes.length,
                                                           dispatch_get_main_queue(),
                                                           DISPATCH_DATA_DESTRUCTOR_DEFAULT);
        id<MTLLibrary> library = [device newLibraryWithData:libraryData error:&error];
        if (library == nil) {
            die([NSString stringWithFormat:@"cannot load metallib: %@", error]);
        }
        printf("environment.metallib_sha256=%s\n",
               hex_digest(libraryBytes.bytes, libraryBytes.length).UTF8String);

        // Every function the library exports is prepared. The driver holds the
        // resulting count against the protocol's frozen population, so a
        // variant that failed to compile cannot quietly shrink the sweep.
        NSMutableDictionary<NSString *, id<MTLComputePipelineState>> *pipelines = [NSMutableDictionary dictionary];
        for (NSString *name in library.functionNames) {
            id<MTLFunction> function = [library newFunctionWithName:name];
            if (function == nil) {
                continue;
            }
            id<MTLComputePipelineState> pipeline = [device newComputePipelineStateWithFunction:function error:&error];
            if (pipeline == nil) {
                die([NSString stringWithFormat:@"pipeline for %@ failed: %@", name, error]);
            }
            pipelines[name] = pipeline;
            printf("pipeline.%s.max_threads=%lu\n", name.UTF8String,
                   (unsigned long)pipeline.maxTotalThreadsPerThreadgroup);
            printf("pipeline.%s.static_threadgroup_memory=%lu\n", name.UTF8String,
                   (unsigned long)pipeline.staticThreadgroupMemoryLength);
        }
        printf("environment.prepared_pipeline_count=%lu\n", (unsigned long)pipelines.count);

        id<MTLCommandQueue> queue = [device newCommandQueue];
        NSMutableDictionary<NSString *, OperandSet *> *operandCache = [NSMutableDictionary dictionary];
        NSMutableDictionary<NSString *, id<MTLBuffer>> *outputCache = [NSMutableDictionary dictionary];

        for (NSString *line in read_lines(manifestPath)) {
            NSArray<NSString *> *fields = [line componentsSeparatedByString:@"\t"];
            if (fields.count != 11 || ![fields[0] isEqualToString:@"case"]) {
                die([NSString stringWithFormat:@"malformed manifest line: %@", line]);
            }
            NSString *caseId = fields[1];
            NSString *kernelName = fields[2];
            const uint32_t m_extent = (uint32_t)[fields[3] intValue];
            const uint32_t n_extent = (uint32_t)[fields[4] intValue];
            const uint32_t k_extent = (uint32_t)[fields[5] intValue];
            const uint32_t tile_m = (uint32_t)[fields[6] intValue];
            const uint32_t tile_w = (uint32_t)[fields[7] intValue];
            NSString *operandSource = fields[8];
            const int reps = [fields[9] intValue];
            NSString *emit = fields[10];

            const uint64_t leftCount = (uint64_t)m_extent * k_extent;
            const uint64_t rightCount = (uint64_t)n_extent * k_extent;
            const uint64_t outCount = (uint64_t)m_extent * n_extent;

            id<MTLComputePipelineState> pipeline = pipelines[kernelName];
            if (pipeline == nil) {
                printf("case.%s.status=refused:no-such-kernel\n", caseId.UTF8String);
                continue;
            }

            // Structural preconditions, refused rather than approximated. A
            // refusal is a recorded row, not an abort: refusals are part of the
            // result.
            NSString *refusal = nil;
            const BOOL isDirect = [kernelName isEqualToString:@"contract_direct"];
            if (k_extent == 0 || m_extent == 0 || n_extent == 0) {
                refusal = @"empty-extent";
            } else if (!isDirect && (tile_m == 0 || tile_w == 0)) {
                refusal = @"tiled-requires-positive-tile";
            } else if (!isDirect && (tile_w % tile_m) != 0) {
                refusal = @"tiled-requires-tile-m-dividing-tile-w";
            } else if (!isDirect && (k_extent % tile_w) != 0) {
                refusal = @"tiled-requires-k-multiple-of-tile-w";
            } else if (!isDirect &&
                       (NSUInteger)(tile_m * tile_w) > pipeline.maxTotalThreadsPerThreadgroup) {
                refusal = @"tiled-exceeds-pipeline-max-threads";
            }
            if (refusal != nil) {
                printf("case.%s.status=refused:%s\n", caseId.UTF8String, refusal.UTF8String);
                continue;
            }

            NSString *operandKey = [NSString stringWithFormat:@"%@|%u|%u|%u", operandSource, m_extent, n_extent, k_extent];
            OperandSet *operands = operandCache[operandKey];
            if (operands == nil) {
                operands = [[OperandSet alloc] init];
                operands.left = [device newBufferWithLength:leftCount * sizeof(float)
                                                    options:MTLResourceStorageModeShared];
                operands.right = [device newBufferWithLength:rightCount * sizeof(float)
                                                     options:MTLResourceStorageModeShared];
                if (operands.left == nil || operands.right == nil) {
                    die([NSString stringWithFormat:@"operand allocation failed for %@", caseId]);
                }
                if ([operandSource hasPrefix:@"prng:"]) {
                    const uint64_t seed = strtoull([operandSource substringFromIndex:5].UTF8String, NULL, 10);
                    fill_prng((float *)operands.left.contents, leftCount, seed);
                    fill_prng((float *)operands.right.contents, rightCount, seed ^ 0xA5A5A5A5A5A5A5A5ULL);
                } else if ([operandSource hasPrefix:@"const:"]) {
                    // `const:<a_bits>,<b_bits>` fills each operand with one
                    // binary32 bit pattern. It exists for the signed-zero case:
                    // with `const:80000000,00000000` every product is -0.0, so a
                    // strict fold seeded from the first product returns
                    // 0x80000000 while one seeded at +0.0 returns 0x00000000.
                    // Under PRNG operands those two are indistinguishable,
                    // because fl(+0.0 + x) == x for every x but -0.0 -- so
                    // without this source the deliberately wrong twin would pass
                    // the oracle and the oracle would be demonstrating nothing.
                    NSArray<NSString *> *parts =
                        [[operandSource substringFromIndex:6] componentsSeparatedByString:@","];
                    if (parts.count != 2) {
                        die([NSString stringWithFormat:@"malformed const operand source: %@", operandSource]);
                    }
                    const uint32_t aBits = (uint32_t)strtoul(parts[0].UTF8String, NULL, 16);
                    const uint32_t bBits = (uint32_t)strtoul(parts[1].UTF8String, NULL, 16);
                    float aValue;
                    float bValue;
                    memcpy(&aValue, &aBits, sizeof(aValue));
                    memcpy(&bValue, &bBits, sizeof(bValue));
                    float *leftValues = (float *)operands.left.contents;
                    float *rightValues = (float *)operands.right.contents;
                    for (uint64_t index = 0; index < leftCount; ++index) {
                        leftValues[index] = aValue;
                    }
                    for (uint64_t index = 0; index < rightCount; ++index) {
                        rightValues[index] = bValue;
                    }
                } else {
                    die([NSString stringWithFormat:@"unknown operand source: %@", operandSource]);
                }
                operands.leftDigest = hex_digest(operands.left.contents, (size_t)(leftCount * sizeof(float)));
                operands.rightDigest = hex_digest(operands.right.contents, (size_t)(rightCount * sizeof(float)));
                operandCache[operandKey] = operands;
            }
            printf("case.%s.operand_a_sha256=%s\n", caseId.UTF8String, operands.leftDigest.UTF8String);
            printf("case.%s.operand_b_sha256=%s\n", caseId.UTF8String, operands.rightDigest.UTF8String);

            NSString *outputKey = [NSString stringWithFormat:@"%u|%u", m_extent, n_extent];
            id<MTLBuffer> output = outputCache[outputKey];
            if (output == nil) {
                output = [device newBufferWithLength:outCount * sizeof(float)
                                             options:MTLResourceStorageModeShared];
                if (output == nil) {
                    die([NSString stringWithFormat:@"output allocation failed for %@", caseId]);
                }
                outputCache[outputKey] = output;
            }

            ContractionDims dims = {m_extent, n_extent, k_extent, 0};
            const int totalDispatches = (reps > 0) ? (reps + 1) : 1;  // one warm-up when timing

            for (int dispatchIndex = 0; dispatchIndex < totalDispatches; ++dispatchIndex) {
                float *outputValues = (float *)output.contents;
                for (uint64_t index = 0; index < outCount; ++index) {
                    outputValues[index] = kOutputSeed;
                }

                id<MTLCommandBuffer> commands = [queue commandBuffer];
                id<MTLComputeCommandEncoder> encoder = [commands computeCommandEncoder];
                [encoder setComputePipelineState:pipeline];
                [encoder setBuffer:operands.left offset:0 atIndex:0];
                [encoder setBuffer:operands.right offset:0 atIndex:1];
                [encoder setBuffer:output offset:0 atIndex:2];
                [encoder setBytes:&dims length:sizeof(dims) atIndex:3];

                if (isDirect) {
                    NSUInteger width = MIN((NSUInteger)32, (NSUInteger)n_extent);
                    NSUInteger height = MIN(pipeline.maxTotalThreadsPerThreadgroup / width, (NSUInteger)m_extent);
                    [encoder dispatchThreads:MTLSizeMake(n_extent, m_extent, 1)
                       threadsPerThreadgroup:MTLSizeMake(width, height, 1)];
                } else {
                    [encoder dispatchThreadgroups:MTLSizeMake((n_extent + tile_w - 1) / tile_w,
                                                              (m_extent + tile_m - 1) / tile_m, 1)
                            threadsPerThreadgroup:MTLSizeMake(tile_w, tile_m, 1)];
                }
                [encoder endEncoding];
                [commands commit];
                [commands waitUntilCompleted];

                // Terminal success is required before any readback.
                if (commands.status != MTLCommandBufferStatusCompleted) {
                    printf("case.%s.status=refused:command-buffer-status-%ld\n", caseId.UTF8String,
                           (long)commands.status);
                    goto next_case;
                }
                if (commands.error != nil) {
                    printf("case.%s.status=refused:command-buffer-error\n", caseId.UTF8String);
                    goto next_case;
                }
                if (dispatchIndex > 0 || reps == 0) {
                    const double seconds = commands.GPUEndTime - commands.GPUStartTime;
                    printf("case.%s.gpu_seconds.%d=%.9f\n", caseId.UTF8String,
                           (reps > 0) ? dispatchIndex : 0, seconds);
                }
            }

            {
                const float *values = (const float *)output.contents;
                uint64_t unwritten = 0;
                uint32_t seedBits;
                memcpy(&seedBits, &kOutputSeed, sizeof(seedBits));
                for (uint64_t index = 0; index < outCount; ++index) {
                    uint32_t bits;
                    memcpy(&bits, &values[index], sizeof(bits));
                    if (bits == seedBits) {
                        unwritten += 1;
                    }
                }
                printf("case.%s.unwritten_count=%llu\n", caseId.UTF8String, (unsigned long long)unwritten);
                printf("case.%s.result_sha256=%s\n", caseId.UTF8String,
                       hex_digest(values, (size_t)(outCount * sizeof(float))).UTF8String);

                if ([emit isEqualToString:@"file"]) {
                    NSString *path = [workDir stringByAppendingPathComponent:
                                                  [NSString stringWithFormat:@"%@.bin", caseId]];
                    NSData *payload = [NSData dataWithBytes:values length:(NSUInteger)(outCount * sizeof(float))];
                    if (![payload writeToFile:path atomically:YES]) {
                        die([NSString stringWithFormat:@"cannot write %@", path]);
                    }
                    printf("case.%s.result_file=%s\n", caseId.UTF8String, path.UTF8String);
                }
                printf("case.%s.status=ok\n", caseId.UTF8String);
            }
        next_case:;
        }
        printf("host.status=complete\n");
        return 0;
    }
}
