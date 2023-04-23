//
//  ViewController.m
//  VulkanExample00
//
//  Created by HK416 on 2023/02/22.
//

#import <Metal/Metal.h>
#import "ViewController.h"
#import "SceneDelegate.h"
#import "AppDelegate.h"
#import "framework.h"


@implementation ViewController {
    CADisplayLink *_displayLink;
    BOOL _frameworkStandby;
    BOOL _viewHasAppeared;
    void *_framework;
}

- (void)viewDidLoad {
    [super viewDidLoad];
    
    self.view.contentScaleFactor = UIScreen.mainScreen.nativeScale;
    
    SceneDelegate *sceneDelegate = (SceneDelegate*)UIApplication.sharedApplication.connectedScenes.allObjects.firstObject.delegate;
    sceneDelegate.viewController = self;
    
    _displayLink = [CADisplayLink displayLinkWithTarget:self selector:@selector(frameAdvanced)];
    [_displayLink setPreferredFramesPerSecond:UIScreen.mainScreen.maximumFramesPerSecond];
    [_displayLink addToRunLoop:NSRunLoop.currentRunLoop forMode:NSDefaultRunLoopMode];
    
    _viewHasAppeared = NO;
}

- (void)handleErrorMessage {
    if (_framework == NULL) {
        _frameworkStandby = NO;
    
        char buf[MAX_MSG_BUF_SIZE];
        memset((void*)&buf[0], 0, sizeof(char) * MAX_MSG_BUF_SIZE);
        getLastErrorMessage(&buf[0], MAX_MSG_BUF_SIZE);
        
        UIAlertController *alert = [UIAlertController alertControllerWithTitle:@"Framework Error"
                                                         message:[NSString stringWithUTF8String:buf]
                                                  preferredStyle:UIAlertControllerStyleAlert];
        UIAlertAction* action = [UIAlertAction actionWithTitle:@"Okay"
                                                         style:UIAlertActionStyleDefault
                                                       handler:^void(UIAlertAction *action) { exit(1); }];
        
        [alert addAction:action];
        [self presentViewController:alert animated:YES completion:nil];
    }
    else {
        _frameworkStandby = YES;
    }
}

- (void)frameAdvanced {
    if (_viewHasAppeared && _frameworkStandby) {
        _framework = updateFramework(_framework);
        [self handleErrorMessage];
    }
}

- (void)viewDidAppear:(BOOL)animated {
    [super viewDidAppear:animated];
    
    CGFloat scale_factor = UIScreen.mainScreen.nativeScale;
    const char *assets_dir = [NSBundle.mainBundle.resourcePath stringByAppendingString: @"/Assets/"].UTF8String;
    
    CGRect screenSize = self.view.bounds;
    UIEdgeInsets safeArea = self.view.safeAreaInsets;
    unsigned int screenWidth = (unsigned int)screenSize.size.width;
    unsigned int screenHeight = (unsigned int)screenSize.size.height;
    int viewerTop = (int)safeArea.top;
    int viewerLeft = (int)safeArea.left;
    int viewerBottom = (int)safeArea.bottom;
    int viewerRight = (int)safeArea.right;
    _framework = createFramework((__bridge void*)self.view, assets_dir, (float)scale_factor, screenWidth, screenHeight, viewerTop, viewerLeft, viewerBottom, viewerRight);
    [self handleErrorMessage];
    
    _viewHasAppeared = YES;
}

- (BOOL)canBecomeFirstResponder {
    return _viewHasAppeared;
}

- (void)destroyFramework {
    if (_viewHasAppeared && _frameworkStandby) {
        [_displayLink invalidate];
        destroyFramework(_framework);
    }
}

- (void)pausedFramework {
    if (_viewHasAppeared && _frameworkStandby) {
        [_displayLink setPaused:YES];
        _framework = pauseFramework(_framework);
        [self handleErrorMessage];
    }
}

- (void)resumeFramework {
    if (_viewHasAppeared && _frameworkStandby) {
        [_displayLink setPaused:NO];
        _framework = resumeFramework(_framework);
        [self handleErrorMessage];
    }
}


@end

#pragma mark - AppView @implementation

@implementation AppView

+ (Class)layerClass {
    return [CAMetalLayer class];
}

@end
