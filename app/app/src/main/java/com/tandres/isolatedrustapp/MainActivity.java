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
    private RustHelloWorld mRustObject = null;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Log.i(TAG, "Starting");

        mRustObject =  new RustHelloWorld();
        mRustObject.start();

        mHandler.postDelayed(() -> {
            Log.i(TAG, "Binding service");
            Intent intent = new Intent(MainActivity.this, IsolatedRustService.class);
            bindService(intent, mConnection, Context.BIND_AUTO_CREATE);
        }, 10000);
    }

    private final ServiceConnection mConnection = new ServiceConnection() {
        public void onServiceConnected(ComponentName className, IBinder service) {
            Log.i(TAG, "Service connected");
            iIsolatedRustInterface = IIsolatedRustInterface.Stub.asInterface(service);
            mHandler.post(() -> {
                Log.i(TAG, "Reading file across service boundary");
                try {
                    File file = new File(getFilesDir(), "test_file");
                    ParcelFileDescriptor pfd = ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY);
                    iIsolatedRustInterface.readFile(pfd);
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