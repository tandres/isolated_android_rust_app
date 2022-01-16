package com.tandres.isolatedrustapp;

import androidx.appcompat.app.AppCompatActivity;

import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.ServiceConnection;
import android.os.Binder;
import android.os.Bundle;
import android.os.Handler;
import android.os.IBinder;
import android.os.ParcelFileDescriptor;
import android.util.Log;

import java.io.File;
import java.io.FileInputStream;

public class MainActivity extends AppCompatActivity {
    private static final String TAG = "IsolatedMain";
    private Handler mHandler = new Handler();
    private IIsolatedRustInterface iIsolatedRustInterface = null;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Log.i(TAG, "Starting");

        mHandler.postDelayed(() -> {
            Log.i(TAG, "Binding service");
            Intent intent = new Intent(MainActivity.this, IsolatedRustService.class);
            bindService(intent, mConnection, Context.BIND_AUTO_CREATE);
            /*RustHelloWorld.main("TJ");
            try {
                File file = new File(getFilesDir(), "test_file");
                // We don't actually need pfd here (yet) but there's not a way to get
                // a raw fd to give to rust otherwise? I probably just can't find it.
                ParcelFileDescriptor pfd = ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY);
                RustHelloWorld.read_file(pfd.detachFd());
            } catch (Exception e) {
                Log.e(TAG, "Got exception " + e);
            } */

        }, 10000);
    }

    private final ServiceConnection mConnection = new ServiceConnection() {
        public void onServiceConnected(ComponentName className, IBinder service) {
            Log.i(TAG, "Service connected");
            iIsolatedRustInterface = IIsolatedRustInterface.Stub.asInterface(service);
            mHandler.post(() -> {
                Log.i(TAG, "Saying hello!");
                try {
                    iIsolatedRustInterface.say_hello();
                } catch(Exception e) {
                    Log.e(TAG, e.toString());
                }
            });
        }

        public void onServiceDisconnected(ComponentName className) {
            Log.e(TAG, "service disconnected");
            iIsolatedRustInterface = null;
        }
    };
}