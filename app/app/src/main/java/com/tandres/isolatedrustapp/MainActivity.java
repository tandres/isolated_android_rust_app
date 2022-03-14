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
    public native void startParent(MainActivity activity, Intent intent);

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Log.i(TAG, "Starting");

        mHandler.postDelayed(() -> {
            Log.i(TAG, "Binding service");
            //build up intent from java side because it is easier but we could do it from Rust
            Intent intent = new Intent(MainActivity.this, IsolatedRustService.class);
            startParent(MainActivity.this, intent);
        }, 5000);
    }
}