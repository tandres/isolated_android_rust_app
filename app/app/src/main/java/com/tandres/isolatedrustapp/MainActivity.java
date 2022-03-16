package com.tandres.isolatedrustapp;

import androidx.appcompat.app.AppCompatActivity;

import android.content.ComponentName;
import android.content.Intent;
import android.content.ServiceConnection;
import android.os.Bundle;
import android.os.Handler;
import android.os.IBinder;
import android.util.Log;

public class MainActivity extends AppCompatActivity implements ServiceConnection {
    private static final String TAG = "IsolatedMain";
    private Handler mHandler = new Handler();

    static {
        System.loadLibrary("rust");
    }

    public native void onServiceConnected(ComponentName className, IBinder service);
    public native void onServiceDisconnected(ComponentName className);
    public native void startParent(MainActivity activity);

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Log.i(TAG, "Starting");

        mHandler.postDelayed(() -> {
            Log.i(TAG, "Binding service");
            startParent(MainActivity.this);
        }, 5000);
    }
}