//
//  SceneDelegate.m
//  VulkanExample00
//
//  Created by HK416 on 2023/02/22.
//

#import "SceneDelegate.h"
#import "ViewController.h"

@interface SceneDelegate ()

@end

@implementation SceneDelegate


- (void)scene:(UIScene *)scene willConnectToSession:(UISceneSession *)session options:(UISceneConnectionOptions *)connectionOptions {
    // Use this method to optionally configure and attach the UIWindow `window` to the provided UIWindowScene `scene`.
    // If using a storyboard, the `window` property will automatically be initialized and attached to the scene.
    // This delegate does not imply the connecting scene or session are new (see `application:configurationForConnectingSceneSession` instead).
}


- (void)sceneDidDisconnect:(UIScene *)scene {
    [(ViewController*)_viewController destroyFramework];
}


- (void)sceneWillResignActive:(UIScene *)scene {
    [(ViewController*)_viewController pausedFramework];
}


- (void)sceneDidBecomeActive:(UIScene *)scene {
    [(ViewController*)_viewController resumeFramework];
}



@end
