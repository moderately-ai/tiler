#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#import <QuartzCore/QuartzCore.h>

#include <math.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    uint32_t heads;
    uint32_t live_extent;
    uint32_t width;
    uint32_t head_stride;
} LayoutDims;

static void die(NSString *message) {
    fprintf(stderr, "dynamic-kv-layout: %s\n", message.UTF8String);
    exit(2);
}

static float value_at(uint32_t head, uint32_t sequence, uint32_t component) {
    int32_t value = (int32_t)((head * 131u + sequence * 17u + component) % 1009u) - 504;
    return (float)value;
}

static uint64_t percentile(uint64_t *values, NSUInteger count, double fraction) {
    qsort_b(values, count, sizeof(uint64_t), ^int(const void *a, const void *b) {
        const uint64_t left = *(const uint64_t *)a;
        const uint64_t right = *(const uint64_t *)b;
        return left < right ? -1 : left > right ? 1 : 0;
    });
    NSUInteger index = (NSUInteger)floor((double)(count - 1) * fraction);
    return values[index];
}

static id<MTLBuffer> make_input(id<MTLDevice> device, NSString *layout,
                                uint32_t live, uint32_t capacity) {
    const uint32_t heads = 8, width = 128;
    const uint64_t stored = [layout isEqualToString:@"exact-head"]
        ? (uint64_t)heads * live * width
        : (uint64_t)heads * capacity * width;
    id<MTLBuffer> buffer = [device newBufferWithLength:stored * sizeof(float)
                                                 options:MTLResourceStorageModeShared];
    if (buffer == nil) die(@"input allocation failed");
    float *values = buffer.contents;
    for (uint64_t i = 0; i < stored; ++i) values[i] = -1048576.0f;
    for (uint32_t h = 0; h < heads; ++h) {
        for (uint32_t s = 0; s < live; ++s) {
            for (uint32_t d = 0; d < width; ++d) {
                uint64_t index;
                if ([layout isEqualToString:@"sequence-major"]) {
                    index = ((uint64_t)s * heads + h) * width + d;
                } else {
                    uint32_t stride = ([layout isEqualToString:@"exact-head"] ? live : capacity) * width;
                    index = (uint64_t)h * stride + (uint64_t)s * width + d;
                }
                values[index] = value_at(h, s, d);
            }
        }
    }
    return buffer;
}

static void verify_output(id<MTLBuffer> output, uint32_t live, NSString *label) {
    const float *values = output.contents;
    uint64_t mismatches = 0;
    uint64_t first = UINT64_MAX;
    const uint64_t count = (uint64_t)8 * live * 128;
    for (uint64_t gid = 0; gid < count; ++gid) {
        uint32_t d = (uint32_t)(gid % 128);
        uint32_t s = (uint32_t)((gid / 128) % live);
        uint32_t h = (uint32_t)(gid / ((uint64_t)128 * live));
        if (values[gid] != value_at(h, s, d)) {
            if (first == UINT64_MAX) first = gid;
            ++mismatches;
        }
    }
    if (mismatches != 0) {
        die([NSString stringWithFormat:@"oracle mismatch for %@: count=%llu first=%llu",
             label, mismatches, first]);
    }
}

static void run_access_case(id<MTLDevice> device, id<MTLCommandQueue> queue,
                            NSDictionary<NSString *, id<MTLComputePipelineState>> *pipelines,
                            NSString *cell, uint32_t live, uint32_t capacity,
                            NSString *layout, NSString *injected, int round,
                            int warmups, int reps) {
    NSString *kernel = [layout isEqualToString:@"exact-head"] ? @"copy_exact_head_major" :
                       [layout isEqualToString:@"capacity-head"] ? @"copy_capacity_head_major" :
                       @"copy_sequence_major";
    id<MTLBuffer> input = make_input(device, layout, live, capacity);
    const uint64_t count = (uint64_t)8 * live * 128;
    id<MTLBuffer> output = [device newBufferWithLength:count * sizeof(float)
                                                  options:MTLResourceStorageModeShared];
    if (output == nil) die(@"output allocation failed");
    LayoutDims dims = {8, live, 128,
        ([layout isEqualToString:@"exact-head"] ? live : capacity) * 128};
    if ([injected isEqualToString:layout]) {
        if ([layout isEqualToString:@"sequence-major"]) {
            // A head-major input presented to the sequence-major payload.
            input = make_input(device, @"capacity-head", live, capacity);
        } else {
            // Swap the two valid head strides; both remain in-bounds.
            dims.head_stride = ([layout isEqualToString:@"exact-head"] ? capacity : live) * 128;
        }
    }
    const int total = warmups + reps;
    uint64_t *gpu = calloc((size_t)reps, sizeof(uint64_t));
    uint64_t *wall = calloc((size_t)reps, sizeof(uint64_t));
    if (gpu == NULL || wall == NULL) die(@"sample allocation failed");
    for (int iteration = 0; iteration < total; ++iteration) {
        memset(output.contents, 0xff, (size_t)(count * sizeof(float)));
        CFTimeInterval start = CACurrentMediaTime();
        id<MTLCommandBuffer> command = [queue commandBuffer];
        id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
        id<MTLComputePipelineState> pipeline = pipelines[kernel];
        [encoder setComputePipelineState:pipeline];
        [encoder setBuffer:input offset:0 atIndex:0];
        [encoder setBuffer:output offset:0 atIndex:1];
        [encoder setBytes:&dims length:sizeof(dims) atIndex:2];
        NSUInteger width = MIN((NSUInteger)256, pipeline.maxTotalThreadsPerThreadgroup);
        [encoder dispatchThreads:MTLSizeMake((NSUInteger)count, 1, 1)
           threadsPerThreadgroup:MTLSizeMake(width, 1, 1)];
        [encoder endEncoding];
        [command commit];
        [command waitUntilCompleted];
        CFTimeInterval end = CACurrentMediaTime();
        if (command.status != MTLCommandBufferStatusCompleted || command.error != nil) {
            die([NSString stringWithFormat:@"command failed for %@/%@: status=%lu error=%@",
                 cell, layout, (unsigned long)command.status, command.error]);
        }
        if (iteration == 0 || iteration == total - 1) verify_output(output, live, layout);
        if (iteration >= warmups) {
            int sample = iteration - warmups;
            gpu[sample] = (uint64_t)llround((command.GPUEndTime - command.GPUStartTime) * 1.0e9);
            wall[sample] = (uint64_t)llround((end - start) * 1.0e9);
        }
    }
    uint64_t gpuMedian = percentile(gpu, reps, 0.5);
    uint64_t gpuP95 = percentile(gpu, reps, 0.95);
    uint64_t wallMedian = percentile(wall, reps, 0.5);
    printf("access\t%s\t%s\t%u\t%u\t%d\t%d\t%d\t%llu\t%llu\t%llu\n",
           cell.UTF8String, layout.UTF8String, live, capacity, round, warmups, reps,
           gpuMedian, gpuP95, wallMedian);
    free(gpu);
    free(wall);
}

static void run_allocation_case(id<MTLDevice> device, NSString *cell,
                                uint32_t live, uint32_t capacity,
                                NSString *layout, int warmups, int reps) {
    uint32_t stored = [layout isEqualToString:@"exact-head"] ? live : capacity;
    NSUInteger bytes = (NSUInteger)8 * stored * 128 * sizeof(float);
    uint64_t *samples = calloc((size_t)reps, sizeof(uint64_t));
    if (samples == NULL) die(@"allocation sample allocation failed");
    for (int iteration = 0; iteration < warmups + reps; ++iteration) {
        @autoreleasepool {
            CFTimeInterval start = CACurrentMediaTime();
            id<MTLBuffer> key = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
            id<MTLBuffer> value = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
            CFTimeInterval end = CACurrentMediaTime();
            if (key == nil || value == nil) die(@"allocation benchmark failed");
            if (iteration >= warmups) samples[iteration - warmups] =
                (uint64_t)llround((end - start) * 1.0e9);
        }
    }
    uint64_t median = percentile(samples, reps, 0.5);
    uint64_t p95 = percentile(samples, reps, 0.95);
    printf("allocation\t%s\t%s\t%u\t%u\t-1\t%d\t%d\t%zu\t%llu\t%llu\n",
           cell.UTF8String, layout.UTF8String, live, capacity, warmups, reps,
           bytes * 2, median, p95);
    free(samples);
}

static void run_lifecycle(id<MTLDevice> device, NSString *cell,
                          uint32_t first, uint32_t capacity,
                          NSString *layout, int warmups, int reps) {
    uint64_t *samples = calloc((size_t)reps, sizeof(uint64_t));
    if (samples == NULL) die(@"lifecycle sample allocation failed");
    const uint32_t steps = capacity - first + 1;
    const BOOL exact = [layout isEqualToString:@"exact-head-compact"];
    uint64_t requested = 0;
    if (exact) {
        for (uint32_t live = first; live <= capacity; ++live) {
            requested += (uint64_t)2 * 8 * live * 128 * sizeof(float);
        }
    } else {
        requested = (uint64_t)4 * 8 * capacity * 128 * sizeof(float);
    }
    for (int iteration = 0; iteration < warmups + reps; ++iteration) {
        @autoreleasepool {
            CFTimeInterval start = CACurrentMediaTime();
            id<MTLBuffer> oldKey = nil, oldValue = nil;
            if (exact) {
                for (uint32_t live = first; live <= capacity; ++live) {
                    NSUInteger bytes = (NSUInteger)8 * live * 128 * sizeof(float);
                    id<MTLBuffer> nextKey = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
                    id<MTLBuffer> nextValue = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
                    if (nextKey == nil || nextValue == nil) die(@"exact lifecycle allocation failed");
                    oldKey = nextKey;
                    oldValue = nextValue;
                }
            } else {
                NSUInteger bytes = (NSUInteger)8 * capacity * 128 * sizeof(float);
                oldKey = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
                oldValue = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
                id<MTLBuffer> nextKey = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
                id<MTLBuffer> nextValue = [device newBufferWithLength:bytes options:MTLResourceStorageModeShared];
                if (oldKey == nil || oldValue == nil || nextKey == nil || nextValue == nil) {
                    die(@"stable lifecycle allocation failed");
                }
            }
            CFTimeInterval end = CACurrentMediaTime();
            if (iteration >= warmups) samples[iteration - warmups] =
                (uint64_t)llround((end - start) * 1.0e9);
        }
    }
    uint64_t median = percentile(samples, reps, 0.5);
    uint64_t p95 = percentile(samples, reps, 0.95);
    printf("lifecycle\t%s\t%s\t%u\t%u\t-1\t%d\t%d\t%llu\t%llu\t%llu\n",
           cell.UTF8String, layout.UTF8String, first, capacity, warmups, reps,
           requested, median, p95);
    fprintf(stderr, "lifecycle %s/%s: steps=%u requested=%llu\n",
            cell.UTF8String, layout.UTF8String, steps, requested);
    free(samples);
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc < 2 || argc > 3) die(@"usage: host <metallib> [inject-layout]");
        NSString *injected = argc == 3 ? [NSString stringWithUTF8String:argv[2]] : @"none";
        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) die(@"no default Metal device");
        NSError *error = nil;
        id<MTLLibrary> library = [device newLibraryWithURL:
            [NSURL fileURLWithPath:[NSString stringWithUTF8String:argv[1]]] error:&error];
        if (library == nil) die([NSString stringWithFormat:@"cannot load metallib: %@", error]);
        NSMutableDictionary *pipelines = [NSMutableDictionary dictionary];
        for (NSString *name in @[@"copy_exact_head_major", @"copy_capacity_head_major",
                                 @"copy_sequence_major"]) {
            id<MTLFunction> function = [library newFunctionWithName:name];
            id<MTLComputePipelineState> pipeline =
                [device newComputePipelineStateWithFunction:function error:&error];
            if (pipeline == nil) die([NSString stringWithFormat:@"pipeline %@: %@", name, error]);
            pipelines[name] = pipeline;
        }
        printf("kind\tcell\tlayout\tlive\tcapacity\tround\twarmups\treps\tbytes_or_gpu_median_ns\tp95_ns\twall_median_ns\n");
        NSArray *cells = @[
            @[@"C1-first", @5, @18], @[@"C1-last", @15, @18],
            @[@"B1-first", @8192, @8320], @[@"B1-last", @8320, @8320]
        ];
        NSArray *layouts = @[@"exact-head", @"capacity-head", @"sequence-major"];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        for (NSArray *cell in cells) {
            if (![injected isEqualToString:@"none"]) {
                run_access_case(device, queue, pipelines, cell[0], [cell[1] unsignedIntValue],
                                [cell[2] unsignedIntValue], injected, injected, 0, 1, 1);
                return 0;
            }
            for (int round = 0; round < 5; ++round) {
                for (int position = 0; position < 3; ++position) {
                    int index = (round + position) % 3;
                    NSString *layout = layouts[index];
                    run_access_case(device, queue, pipelines, cell[0], [cell[1] unsignedIntValue],
                                    [cell[2] unsignedIntValue], layout, injected, round, 3, 7);
                }
            }
            for (NSString *layout in layouts) {
                run_allocation_case(device, cell[0], [cell[1] unsignedIntValue],
                                    [cell[2] unsignedIntValue], layout, 20, 101);
            }
        }
        for (NSString *layout in layouts) {
            NSString *lifecycleLayout = [layout isEqualToString:@"exact-head"]
                ? @"exact-head-compact" : layout;
            run_lifecycle(device, @"C1-decode", 5, 18, lifecycleLayout, 10, 51);
            run_lifecycle(device, @"B1-decode", 8192, 8320, lifecycleLayout, 3, 11);
        }
        run_lifecycle(device, @"C1-decode", 5, 18, @"exact-head-pooled", 10, 51);
        run_lifecycle(device, @"B1-decode", 8192, 8320, @"exact-head-pooled", 3, 11);
        printf("environment\tdevice\t%s\n", device.name.UTF8String);
    }
    return 0;
}
