package com.tandres.isolatedrustapp;

import android.app.Service;
import android.content.Intent;
import android.os.Handler;
import android.os.IBinder;
import android.os.ParcelFileDescriptor;
import android.util.Log;

public class IsolatedRustService extends Service {
    private static final String TAG = "IsolatedService";
    private Handler mHandler = new Handler();

    static {
        System.loadLibrary("rust");
    }

    public IsolatedRustService() {
    }

    @Override
    public void onCreate() {
        Log.i(TAG, "Created!");
    }

    @Override
    public void onDestroy() {
        Log.i(TAG, "Destroyed!");
    }

    @Override
    public IBinder onBind(Intent intent) {
        Log.i(TAG, "Bound!");
        return mBinder;
    }

    public native void startChild(ParcelFileDescriptor pfd);

    private final IIsolatedRustInterface.Stub mBinder = new IIsolatedRustInterface.Stub() {
        @Override
        public void start(ParcelFileDescriptor pfd) {
            startChild(pfd);
        }
    };
}