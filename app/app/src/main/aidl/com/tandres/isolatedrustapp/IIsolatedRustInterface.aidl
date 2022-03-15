// IIsolatedRustInterface.aidl
package com.tandres.isolatedrustapp;

import android.os.ParcelFileDescriptor;

interface IIsolatedRustInterface {
    void start(in ParcelFileDescriptor pfd, int id);
}