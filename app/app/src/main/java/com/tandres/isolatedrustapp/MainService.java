package com.tandres.isolatedrustapp;

import androidx.annotation.Nullable;
import android.app.Service;

import android.content.ComponentName;
import android.content.Intent;
import android.content.ServiceConnection;
import android.os.Handler;
import android.os.IBinder;
import android.util.Log;

public class MainService extends Service implements ServiceConnection {
    private static final String TAG = "IsolatedMain";
    private Handler mHandler = new Handler();

    static {
        System.loadLibrary("rust");
    }

    public MainService() {

    }

    public native void onServiceConnected(ComponentName className, IBinder service);
    public native void onServiceDisconnected(ComponentName className);
    public native void startParent(MainService activity);

    @Override
    public void onCreate() {

        Log.i(TAG, "Starting");

        mHandler.postDelayed(() -> {
            Log.i(TAG, "Binding service");
            startParent(MainService.this);
        }, 5000);
    }

    @Nullable
    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
}