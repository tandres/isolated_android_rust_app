// IIsolatedRustInterface.aidl
package com.tandres.isolatedrustapp;

import android.os.ParcelFileDescriptor;

interface IIsolatedRustInterface {
    void say_hello();
    int getPid();
    ParcelFileDescriptor getStdoutFd();
    void readFile(in ParcelFileDescriptor pfd);
    void start(in ParcelFileDescriptor pfd);
}