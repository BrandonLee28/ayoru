#import <AppKit/AppKit.h>
#import <AVFoundation/AVFoundation.h>
#import <AVKit/AVKit.h>
#include <assert.h>
#include <string.h>

void *ayoru_controls_create(void);
void ayoru_controls_place(void *, void *, int, double, double, double, double);
void ayoru_controls_destroy(void *);
const char *ayoru_search_take(void *);
void ayoru_search_clear_submission(void *);
void ayoru_search_visible(void *, bool);
void ayoru_video_load(void *, const char *, const char *);
int ayoru_video_status(void *);
void ayoru_video_stop(void *);
const char *ayoru_video_error(void *);

int main(int argc, const char **argv) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        NSWindow *window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 640, 480)
            styleMask:NSWindowStyleMaskTitled backing:NSBackingStoreBuffered defer:NO];
        void *controls = ayoru_controls_create();
        ayoru_controls_place(controls, (__bridge void *)window.contentView, 0, 10, 10, 400, 30);
        ayoru_controls_place(controls, (__bridge void *)window.contentView, 1, 10, 60, 600, 340);
        assert(window.contentView.subviews.count == 2);
        NSSearchField *search = nil;
        for (NSView *view in window.contentView.subviews) {
            if ([view isKindOfClass:NSSearchField.class]) search = (NSSearchField *)view;
        }
        assert(search);
        assert(search.focusRingType == NSFocusRingTypeNone);
        ayoru_search_visible(controls, false);
        assert(search.hidden);
        ayoru_search_visible(controls, true);
        assert(!search.hidden);
        search.stringValue = @"葬送のフリーレン";
        [NSApp sendAction:search.action to:search.target from:search];
        assert(strcmp(ayoru_search_take(controls), "葬送のフリーレン") == 0);
        ayoru_search_clear_submission(controls);
        assert(ayoru_search_take(controls) == NULL);
        if (argc > 1) {
            NSString *file = [NSString stringWithUTF8String:argv[1]];
            NSURL *url = [file hasPrefix:@"https://"] ? [NSURL URLWithString:file] : [NSURL fileURLWithPath:file];
            ayoru_video_load(controls, url.absoluteString.UTF8String, argc > 2 ? argv[2] : "https://mkissa.to/");
            NSDate *began = [NSDate date];
            NSDate *deadline = [NSDate dateWithTimeIntervalSinceNow:8];
            BOOL started = NO;
            while ([deadline timeIntervalSinceNow] > 0) {
                [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.05]];
                int status = ayoru_video_status(controls);
                if (status == -1) fprintf(stderr, "Video error: %s\n", ayoru_video_error(controls));
                assert(status != -1);
                if (status == 1) { started = YES; break; }
            }
            if (!started) {
                for (NSView *view in window.contentView.subviews) {
                    if ([view isKindOfClass:AVPlayerView.class]) {
                        AVPlayer *player = ((AVPlayerView *)view).player;
                        NSLog(@"Playback did not start: item=%ld player=%ld timeControl=%ld reason=%@ loaded=%@ error=%@", (long)player.currentItem.status, (long)player.status, (long)player.timeControlStatus, player.reasonForWaitingToPlay, player.currentItem.loadedTimeRanges, player.currentItem.error);
                    }
                }
            }
            assert(started && "Native video never started");
            NSTimeInterval elapsed = -[began timeIntervalSinceNow];
            printf("Playback started in %.2f seconds.\n", elapsed);
            assert(elapsed < 6.0 && "Playback exceeded the app's startup timeout");
        }
        ayoru_video_stop(controls);
        assert(ayoru_video_status(controls) == 0);
        ayoru_controls_destroy(controls);
        assert(window.contentView.subviews.count == 0);
        puts("Native search, video attachment, playback and cleanup passed.");
    }
    return 0;
}
