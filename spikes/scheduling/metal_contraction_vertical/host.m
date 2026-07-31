// Dispatch host for the first Metal contraction vertical.
//
// It reads a manifest the Python driver writes, runs one line at a time in
// manifest order, and prints `key=value` rows on stdout. Manifest order is what
// makes the timing legs A/B interleaved: the driver emits round-robin lines and
// this host never reorders them.
//
// Two disciplines are load-bearing and are stated here because a reader of the
// retained record has to be able to check them:
//
//   1. The output allocation is seeded with a finite pattern no admitted case
//      can produce (-3.0e38f; every case's exact result is bounded by 768 in
//      magnitude), so "never written" is distinguishable from "wrote zero". The
//      pattern is finite rather than NaN because the opaque MPS path computes
//      `alpha * A*B + beta * C` and a NaN seed would poison the result through
//      `0 * NaN` even where beta is zero.
//   2. `MTLCommandBufferStatusCompleted` and a nil `commandBuffer.error` are
//      required before any readback, per the repository's rule that exact
//      command-buffer terminal success precedes host validation readback.
//
// Operands are generated in-process from a SplitMix64 stream so that a
// 622 MB weight matrix never touches the filesystem. The host prints the
// SHA-256 of the exact operand bytes it generated; the driver reconstructs the
// same stream independently and stops if the digests disagree, which is what
// makes the CPU oracle a comparison against these operands rather than against
// operands the driver merely believes were used.

#import <CommonCrypto/CommonDigest.h>
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#import <MetalPerformanceShaders/MetalPerformanceShaders.h>

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
// Operand generation. Mirrored exactly by `contraction_probe.py`.
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

// --------------------------------------------------------------------------
// Operand cache. Keyed by the manifest's own operand-source string plus the
// extents, so the 622 MB vocabulary weight is built once per process.
// --------------------------------------------------------------------------

@interface OperandSet : NSObject
@property(nonatomic, strong) id<MTLBuffer> left;
@property(nonatomic, strong) id<MTLBuffer> right;
@property(nonatomic, copy) NSString *leftDigest;
@property(nonatomic, copy) NSString *rightDigest;
@end

@implementation OperandSet
@end

static void die(NSString *message) {
    fprintf(stderr, "contraction host: %s\n", message.UTF8String);
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
        printf("environment.device=%s\n", device.name.UTF8String);
        printf("environment.device_apple9=%s\n",
               [device supportsFamily:MTLGPUFamilyApple9] ? "supported" : "unsupported");
        printf("environment.device_max_threads_per_threadgroup=%lu\n",
               (unsigned long)device.maxThreadsPerThreadgroup.width);
        printf("environment.device_max_buffer_length=%llu\n",
               (unsigned long long)device.maxBufferLength);
        printf("environment.mps_supported=%s\n", MPSSupportsMTLDevice(device) ? "yes" : "no");

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
        }

        id<MTLCommandQueue> queue = [device newCommandQueue];
        NSMutableDictionary<NSString *, OperandSet *> *operandCache = [NSMutableDictionary dictionary];
        NSMutableDictionary<NSString *, id<MTLBuffer>> *outputCache = [NSMutableDictionary dictionary];

        for (NSString *line in read_lines(manifestPath)) {
            NSArray<NSString *> *fields = [line componentsSeparatedByString:@"\t"];
            if (fields.count != 10 || ![fields[0] isEqualToString:@"case"]) {
                die([NSString stringWithFormat:@"malformed manifest line: %@", line]);
            }
            NSString *caseId = fields[1];
            NSString *kernelName = fields[2];
            const uint32_t m_extent = (uint32_t)[fields[3] intValue];
            const uint32_t n_extent = (uint32_t)[fields[4] intValue];
            const uint32_t k_extent = (uint32_t)[fields[5] intValue];
            const uint32_t split = (uint32_t)[fields[6] intValue];
            NSString *operandSource = fields[7];
            const int reps = [fields[8] intValue];
            NSString *emit = fields[9];

            const uint64_t leftCount = (uint64_t)m_extent * k_extent;
            const uint64_t rightCount = (uint64_t)n_extent * k_extent;
            const uint64_t outCount = (uint64_t)m_extent * n_extent;

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
                } else if ([operandSource hasPrefix:@"file:"]) {
                    NSArray<NSString *> *paths = [[operandSource substringFromIndex:5] componentsSeparatedByString:@","];
                    if (paths.count != 2) {
                        die([NSString stringWithFormat:@"malformed file operand source: %@", operandSource]);
                    }
                    NSData *leftData = [NSData dataWithContentsOfFile:paths[0]];
                    NSData *rightData = [NSData dataWithContentsOfFile:paths[1]];
                    if (leftData.length != leftCount * sizeof(float) || rightData.length != rightCount * sizeof(float)) {
                        die([NSString stringWithFormat:@"operand file size mismatch for %@", caseId]);
                    }
                    memcpy(operands.left.contents, leftData.bytes, leftData.length);
                    memcpy(operands.right.contents, rightData.bytes, rightData.length);
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

            const BOOL isOpaque = [kernelName isEqualToString:@"mps"];
            id<MTLComputePipelineState> pipeline = nil;
            if (!isOpaque) {
                pipeline = pipelines[kernelName];
                if (pipeline == nil) {
                    printf("case.%s.status=refused:no-such-kernel\n", caseId.UTF8String);
                    continue;
                }
            }

            // Structural preconditions, refused rather than approximated.
            NSString *refusal = nil;
            if (k_extent == 0 || m_extent == 0 || n_extent == 0) {
                refusal = @"empty-extent";
            } else if ([kernelName isEqualToString:@"contract_tiled"] && (k_extent % 16u) != 0) {
                refusal = @"tiled-requires-k-multiple-of-16";
            } else if (([kernelName isEqualToString:@"contract_ksplit_contiguous"] ||
                        [kernelName isEqualToString:@"contract_ksplit_strided"]) &&
                       (split == 0 || split > 32 || (k_extent % split) != 0)) {
                refusal = @"ksplit-requires-k-multiple-of-split";
            } else if ([kernelName isEqualToString:@"contract_simdgroup"] &&
                       ((m_extent % 8u) != 0 || (n_extent % 8u) != 0 || (k_extent % 8u) != 0)) {
                refusal = @"simdgroup-requires-extents-multiple-of-8";
            }
            if (refusal != nil) {
                printf("case.%s.status=refused:%s\n", caseId.UTF8String, refusal.UTF8String);
                continue;
            }

            MPSMatrixMultiplication *opaque = nil;
            MPSMatrix *opaqueLeft = nil;
            MPSMatrix *opaqueRight = nil;
            MPSMatrix *opaqueOut = nil;
            if (isOpaque) {
                if (!MPSSupportsMTLDevice(device)) {
                    printf("case.%s.status=refused:mps-unsupported-device\n", caseId.UTF8String);
                    continue;
                }
                opaque = [[MPSMatrixMultiplication alloc] initWithDevice:device
                                                           transposeLeft:NO
                                                          transposeRight:YES
                                                              resultRows:m_extent
                                                           resultColumns:n_extent
                                                         interiorColumns:k_extent
                                                                   alpha:1.0
                                                                    beta:0.0];
                if (opaque == nil) {
                    printf("case.%s.status=refused:mps-kernel-unavailable\n", caseId.UTF8String);
                    continue;
                }
                opaqueLeft = [[MPSMatrix alloc]
                    initWithBuffer:operands.left
                        descriptor:[MPSMatrixDescriptor matrixDescriptorWithRows:m_extent
                                                                         columns:k_extent
                                                                        rowBytes:(NSUInteger)k_extent * sizeof(float)
                                                                        dataType:MPSDataTypeFloat32]];
                opaqueRight = [[MPSMatrix alloc]
                    initWithBuffer:operands.right
                        descriptor:[MPSMatrixDescriptor matrixDescriptorWithRows:n_extent
                                                                         columns:k_extent
                                                                        rowBytes:(NSUInteger)k_extent * sizeof(float)
                                                                        dataType:MPSDataTypeFloat32]];
                opaqueOut = [[MPSMatrix alloc]
                    initWithBuffer:output
                        descriptor:[MPSMatrixDescriptor matrixDescriptorWithRows:m_extent
                                                                         columns:n_extent
                                                                        rowBytes:(NSUInteger)n_extent * sizeof(float)
                                                                        dataType:MPSDataTypeFloat32]];
            }

            ContractionDims dims = {m_extent, n_extent, k_extent, split};
            const int totalDispatches = (reps > 0) ? (reps + 1) : 1;  // one warm-up when timing

            for (int dispatchIndex = 0; dispatchIndex < totalDispatches; ++dispatchIndex) {
                float *outputValues = (float *)output.contents;
                for (uint64_t index = 0; index < outCount; ++index) {
                    outputValues[index] = kOutputSeed;
                }

                id<MTLCommandBuffer> commands = [queue commandBuffer];
                if (isOpaque) {
                    [opaque encodeToCommandBuffer:commands
                                       leftMatrix:opaqueLeft
                                      rightMatrix:opaqueRight
                                     resultMatrix:opaqueOut];
                } else {
                    id<MTLComputeCommandEncoder> encoder = [commands computeCommandEncoder];
                    [encoder setComputePipelineState:pipeline];
                    [encoder setBuffer:operands.left offset:0 atIndex:0];
                    [encoder setBuffer:operands.right offset:0 atIndex:1];
                    [encoder setBuffer:output offset:0 atIndex:2];
                    [encoder setBytes:&dims length:sizeof(dims) atIndex:3];

                    if ([kernelName isEqualToString:@"contract_direct"] ||
                        [kernelName isEqualToString:@"contract_direct_zero_seed"]) {
                        NSUInteger width = MIN((NSUInteger)32, (NSUInteger)n_extent);
                        NSUInteger height = MIN(pipeline.maxTotalThreadsPerThreadgroup / width, (NSUInteger)m_extent);
                        [encoder dispatchThreads:MTLSizeMake(n_extent, m_extent, 1)
                           threadsPerThreadgroup:MTLSizeMake(width, height, 1)];
                    } else if ([kernelName isEqualToString:@"contract_tiled"]) {
                        [encoder dispatchThreadgroups:MTLSizeMake((n_extent + 15) / 16, (m_extent + 15) / 16, 1)
                                threadsPerThreadgroup:MTLSizeMake(16, 16, 1)];
                    } else if ([kernelName isEqualToString:@"contract_ksplit_contiguous"] ||
                               [kernelName isEqualToString:@"contract_ksplit_strided"]) {
                        [encoder dispatchThreadgroups:MTLSizeMake((n_extent + 7) / 8, m_extent, 1)
                                threadsPerThreadgroup:MTLSizeMake(32, 8, 1)];
                    } else if ([kernelName isEqualToString:@"contract_simdgroup"]) {
                        [encoder dispatchThreadgroups:MTLSizeMake(n_extent / 8, m_extent / 8, 1)
                                threadsPerThreadgroup:MTLSizeMake(32, 1, 1)];
                    } else {
                        die([NSString stringWithFormat:@"no dispatch shape for kernel %@", kernelName]);
                    }
                    [encoder endEncoding];
                }
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
                for (uint64_t index = 0; index < outCount; ++index) {
                    uint32_t bits;
                    memcpy(&bits, &values[index], sizeof(bits));
                    uint32_t seedBits;
                    memcpy(&seedBits, &kOutputSeed, sizeof(seedBits));
                    if (bits == seedBits) {
                        unwritten += 1;
                    }
                }
                printf("case.%s.unwritten_count=%llu\n", caseId.UTF8String, (unsigned long long)unwritten);
                printf("case.%s.result_sha256=%s\n", caseId.UTF8String,
                       hex_digest(values, (size_t)(outCount * sizeof(float))).UTF8String);

                if ([emit isEqualToString:@"full"]) {
                    NSMutableString *hex = [NSMutableString stringWithCapacity:(NSUInteger)outCount * 9];
                    for (uint64_t index = 0; index < outCount; ++index) {
                        uint32_t bits;
                        memcpy(&bits, &values[index], sizeof(bits));
                        if (index > 0) {
                            [hex appendString:@","];
                        }
                        [hex appendFormat:@"%08x", bits];
                    }
                    printf("case.%s.results=%s\n", caseId.UTF8String, hex.UTF8String);
                } else if ([emit isEqualToString:@"file"]) {
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
