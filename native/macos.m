#import <AppKit/AppKit.h>
#import <AVKit/AVKit.h>
#import <AVFoundation/AVFoundation.h>

// These controls belong to the GPUI window. All bridge calls run on its main thread.
@interface ayoruControls : NSObject <NSSearchFieldDelegate>
@property(nonatomic, strong) NSSearchField *search;
@property(nonatomic, strong) AVPlayerView *video;
@property(nonatomic, copy) NSString *submitted;
@end

@implementation ayoruControls
- (void)submitSearch:(id)sender {
    self.submitted = self.search.stringValue;
}
@end

void ayoru_set_app_icon(const void *bytes, size_t length) {
    NSData *data = [NSData dataWithBytes:bytes length:length];
    NSImage *icon = [[NSImage alloc] initWithData:data];
    if (icon) NSApp.applicationIconImage = icon;
}

void *ayoru_controls_create(void) {
    ayoruControls *controls = [ayoruControls new];
    controls.search = [[NSSearchField alloc] initWithFrame:NSZeroRect];
    controls.search.placeholderString = @"Search anime…";
    controls.search.controlSize = NSControlSizeSmall;
    controls.search.font = [NSFont systemFontOfSize:14 weight:NSFontWeightMedium];
    controls.search.bordered = NO;
    controls.search.focusRingType = NSFocusRingTypeNone;
    controls.search.drawsBackground = NO;
    ((NSSearchFieldCell *)controls.search.cell).searchButtonCell = nil;
    controls.search.textColor = [NSColor colorWithSRGBRed:0.922 green:0.922 blue:0.922 alpha:1];
    controls.search.sendsSearchStringImmediately = NO;
    controls.search.sendsWholeSearchString = YES;
    controls.search.target = controls;
    controls.search.action = @selector(submitSearch:);
    controls.search.appearance = [NSAppearance appearanceNamed:NSAppearanceNameDarkAqua];
    controls.video = [[AVPlayerView alloc] initWithFrame:NSZeroRect];
    controls.video.controlsStyle = AVPlayerViewControlsStyleInline;
    controls.video.showsFullScreenToggleButton = YES;
    controls.video.videoGravity = AVLayerVideoGravityResizeAspect;
    return (__bridge_retained void *)controls;
}

static void place(NSView *view, NSView *parent, double x, double y, double w, double h) {
    if (view.superview != parent) [parent addSubview:view];
    CGFloat top = parent.isFlipped ? y : parent.bounds.size.height - y - h;
    view.frame = NSMakeRect(x, top, w, h);
    view.hidden = NO;
}

void ayoru_controls_place(void *handle, void *parent, int video,
                          double x, double y, double w, double h) {
    ayoruControls *controls = (__bridge ayoruControls *)handle;
    place(video ? controls.video : controls.search, (__bridge NSView *)parent, x, y, w, h);
}

void ayoru_search_visible(void *handle, bool visible) {
    ayoruControls *controls = (__bridge ayoruControls *)handle;
    if (!visible && controls.search.currentEditor) {
        [controls.search.window makeFirstResponder:controls.search.superview];
    }
    controls.search.hidden = !visible;
}

void ayoru_search_focus(void *handle) {
    ayoruControls *controls = (__bridge ayoruControls *)handle;
    [controls.search.window makeFirstResponder:controls.search];
}

const char *ayoru_search_take(void *handle) {
    ayoruControls *controls = (__bridge ayoruControls *)handle;
    return controls.submitted ? controls.submitted.UTF8String : NULL;
}

void ayoru_search_clear_submission(void *handle) {
    ((__bridge ayoruControls *)handle).submitted = nil;
}

void ayoru_video_load(void *handle, const char *url, const char *referrer) {
    ayoruControls *controls = (__bridge ayoruControls *)handle;
    [controls.video.player pause];
    NSURL *mediaURL = [NSURL URLWithString:[NSString stringWithUTF8String:url]];
    NSMutableDictionary *options = [@{
        @"AVURLAssetHTTPHeaderFieldsKey": @{@"Referer": [NSString stringWithUTF8String:referrer], @"User-Agent": @"Mozilla/5.0"}
    } mutableCopy];
    // This provider serves MP4 bytes as application/octet-stream at an
    // extensionless URL. Give AVFoundation the media type it cannot infer.
    if (@available(macOS 13.0, *)) {
        options[AVURLAssetHTTPUserAgentKey] = @"Mozilla/5.0";
    }
    if (@available(macOS 14.0, *)) {
        if ([mediaURL.host isEqualToString:@"tools.fast4speed.rsvp"]) {
            options[AVURLAssetOverrideMIMETypeKey] = @"video/mp4";
        }
    }
    AVURLAsset *asset = [AVURLAsset URLAssetWithURL:mediaURL options:options];
    controls.video.player = [AVPlayer playerWithPlayerItem:[AVPlayerItem playerItemWithAsset:asset]];
    controls.video.player.automaticallyWaitsToMinimizeStalling = NO;
    [controls.video.player play];
}

// 0 = buffering, 1 = actually playing, -1 = failure, 2 = finished.
int ayoru_video_status(void *handle) {
    AVPlayer *player = ((__bridge ayoruControls *)handle).video.player;
    if (!player) return 0;
    if (player.status == AVPlayerStatusFailed || player.currentItem.status == AVPlayerItemStatusFailed) return -1;
    double time = CMTimeGetSeconds(player.currentTime);
    double duration = CMTimeGetSeconds(player.currentItem.duration);
    if (isfinite(duration) && duration > 0 && time >= duration - 0.2) return 2;
    if (player.timeControlStatus == AVPlayerTimeControlStatusPlaying && time > 0) return 1;
    return 0;
}

const char *ayoru_video_error(void *handle) {
    AVPlayer *player = ((__bridge ayoruControls *)handle).video.player;
    return (player.currentItem.error ?: player.error).localizedDescription.UTF8String;
}

void ayoru_video_stop(void *handle) {
    ayoruControls *controls = (__bridge ayoruControls *)handle;
    [controls.video.player pause];
    controls.video.player = nil;
    controls.video.hidden = YES;
}

void ayoru_controls_destroy(void *handle) {
    ayoruControls *controls = (__bridge_transfer ayoruControls *)handle;
    [controls.video.player pause];
    [controls.video removeFromSuperview];
    [controls.search removeFromSuperview];
}
