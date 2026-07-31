#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

int main(void) {
    @autoreleasepool {
        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        NSError *error = nil;
        (void)[device newLibraryWithSource:@"kernel void sentinel() {}"
                                  options:nil
                                    error:&error];
        return error == nil ? 0 : 1;
    }
}
