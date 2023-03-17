//
//  framework.h
//  VulkanExample00
//
//  Created by HK416 on 2023/02/22.
//

#ifndef framework_h
#define framework_h

#define MAX_MSG_BUF_SIZE 4096
 
void *createFramework(void *view, const char* assets_dir, float scale_factor, unsigned int screenWidth, unsigned int screenHeight, int viewerTop, int viewerLeft, int viewerBottom, int viewerRight);

void destroyFramework(void *framework);

void *updateFramework(void *framework);

void *pauseFramework(void *framework);

void *resumeFramework(void *framework);

bool getLastFrameworkErrMsg(char *buf, unsigned int bufSize);

bool getLastFrameworkErrMsgDbg(char *buf, unsigned int bufSize);

bool getLastErrorMessage(char *buf, unsigned int bufSize) {
#ifdef DEBUG
    return getLastFrameworkErrMsgDbg(buf, bufSize);
#else
    return getLastFrameworkErrMsg(buf, bufSize);
#endif
}

#endif /* framework_h */
