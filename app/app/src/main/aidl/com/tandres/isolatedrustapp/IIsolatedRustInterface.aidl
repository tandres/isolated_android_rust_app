// IIsolatedRustInterface.aidl
package com.tandres.isolatedrustapp;

import android.os.ParcelFileDescriptor;

interface IIsolatedRustInterface {
    void say_hello();
    void readFile(in ParcelFileDescriptor pfd);
}