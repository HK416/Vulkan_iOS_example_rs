//
//  framework.h
//  VulkanExample00
//
//  Created by HK416 on 2023/02/22.
//

#ifndef framework_h
#define framework_h

#define MAX_MSG_BUF_SIZE 4096
 
void *createFramework(void *view, float scale, unsigned int screenWidth, unsigned int screenHeight, int viewerTop, int viewerLeft, int viewerBottom, int viewerRight);

void destroyFramework(void *framework);

void *updateFramework(void *framework);

void *pauseFramework(void *framework);

void *resumeFramework(void *framework);

bool getLastFrameworkErrMsg(char *buf, unsigned int bufSize);

#endif /* framework_h */
