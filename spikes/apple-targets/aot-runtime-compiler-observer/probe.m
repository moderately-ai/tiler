// Observe compiler-related process images around native metallib preparation.
//
// This program deliberately has no source-compilation mode. It accepts only a
// metallib produced before process launch, then follows the same data-library
// and compute-pipeline preparation sequence as the serial-sum runtime proof.

#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

#include <dlfcn.h>
#include <dispatch/dispatch.h>
#include <mach-o/dyld.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

static bool is_build_character(uint8_t byte) {
    return (byte >= '0' && byte <= '9') || byte == '.' || byte == '-';
}

static NSArray<NSString *> *compiler_builds(NSString *path) {
    NSData *data = [NSData dataWithContentsOfFile:path];
    if (data == nil) {
        return @[];
    }
    const uint8_t *bytes = data.bytes;
    const NSUInteger length = data.length;
    static const uint8_t prefix[] = "metalfe-";
    NSMutableOrderedSet<NSString *> *builds = [NSMutableOrderedSet orderedSet];
    for (NSUInteger offset = 0; offset + sizeof(prefix) - 1 < length; offset += 1) {
        if (memcmp(bytes + offset, prefix, sizeof(prefix) - 1) != 0) {
            continue;
        }
        NSUInteger end = offset + sizeof(prefix) - 1;
        while (end < length && is_build_character(bytes[end])) {
            end += 1;
        }
        if (end == offset + sizeof(prefix) - 1) {
            continue;
        }
        NSData *slice = [NSData dataWithBytes:bytes + offset length:end - offset];
        NSString *build = [[NSString alloc] initWithData:slice encoding:NSASCIIStringEncoding];
        if (build != nil) {
            [builds addObject:build];
        }
    }
    return builds.array;
}

static bool is_compiler_image(NSString *path) {
    return [path containsString:@"GPUCompiler"] || [path containsString:@"MTLCompiler"];
}

static void snapshot(NSString *stage) {
    uint32_t count = _dyld_image_count();
    NSUInteger compiler_count = 0;
    for (uint32_t index = 0; index < count; index += 1) {
        const char *name = _dyld_get_image_name(index);
        if (name != NULL && is_compiler_image(@(name))) {
            compiler_count += 1;
        }
    }
    printf("stage.%s.compiler_image_count=%lu\n", stage.UTF8String,
           (unsigned long)compiler_count);
    NSUInteger emitted = 0;
    for (uint32_t index = 0; index < count; index += 1) {
        const char *name = _dyld_get_image_name(index);
        if (name == NULL) {
            continue;
        }
        NSString *path = @(name);
        if (!is_compiler_image(path)) {
            continue;
        }
        printf("stage.%s.image.%lu.path=%s\n", stage.UTF8String, (unsigned long)emitted,
               path.UTF8String);
        NSArray<NSString *> *builds = compiler_builds(path);
        printf("stage.%s.image.%lu.build_count=%lu\n", stage.UTF8String,
               (unsigned long)emitted, (unsigned long)builds.count);
        for (NSUInteger build_index = 0; build_index < builds.count; build_index += 1) {
            printf("stage.%s.image.%lu.build.%lu=%s\n", stage.UTF8String,
                   (unsigned long)emitted, (unsigned long)build_index,
                   builds[build_index].UTF8String);
        }
        emitted += 1;
    }
}

static int fail(NSString *stage, NSError *error) {
    fprintf(stderr, "aot-runtime-compiler-observer: %s failed: %s\n", stage.UTF8String,
            error == nil ? "no error object" : error.localizedDescription.UTF8String);
    return 4;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 2 && argc != 4) {
            fprintf(stderr, "usage: probe <metallib> [--preload <image>]\n");
            return 2;
        }
        if (argc == 4) {
            if (strcmp(argv[2], "--preload") != 0) {
                fprintf(stderr, "usage: probe <metallib> [--preload <image>]\n");
                return 2;
            }
            void *handle = dlopen(argv[3], RTLD_NOW | RTLD_LOCAL);
            if (handle == NULL) {
                fprintf(stderr, "aot-runtime-compiler-observer: preload failed: %s\n", dlerror());
                return 4;
            }
        }

        printf("probe.route=native-metallib-library-and-compute-pipeline\n");
        printf("probe.observation_api=dyld-loaded-image-membership-and-image-byte-scan\n");
        snapshot(@"process_start");

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            fprintf(stderr, "aot-runtime-compiler-observer: no default Metal device\n");
            return 3;
        }
        printf("environment.device=%s\n", device.name.UTF8String);
        printf("environment.device_apple9_support=%s\n",
               [device supportsFamily:MTLGPUFamilyApple9] ? "supported" : "unsupported");
        snapshot(@"after_device");

        NSString *metallib_path = @(argv[1]);
        NSData *metallib = [NSData dataWithContentsOfFile:metallib_path];
        if (metallib == nil) {
            fprintf(stderr, "aot-runtime-compiler-observer: cannot read %s\n", argv[1]);
            return 2;
        }
        dispatch_data_t library_data = dispatch_data_create(
            metallib.bytes, metallib.length, dispatch_get_main_queue(),
            DISPATCH_DATA_DESTRUCTOR_DEFAULT);
        NSError *error = nil;
        id<MTLLibrary> library = [device newLibraryWithData:library_data error:&error];
        if (library == nil) {
            return fail(@"newLibraryWithData", error);
        }
        snapshot(@"after_library");

        id<MTLFunction> function = [library newFunctionWithName:@"tiler_aot_observer_probe"];
        if (function == nil) {
            fprintf(stderr, "aot-runtime-compiler-observer: function is absent\n");
            return 4;
        }
        snapshot(@"after_function");

        error = nil;
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (pipeline == nil) {
            return fail(@"newComputePipelineStateWithFunction", error);
        }
        snapshot(@"after_pipeline");
        printf("probe.pipeline_threads=%lu\n", (unsigned long)pipeline.maxTotalThreadsPerThreadgroup);
        printf("probe.status=ok\n");
        return 0;
    }
}
