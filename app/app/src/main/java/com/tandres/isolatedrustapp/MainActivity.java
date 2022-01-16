package com.tandres.isolatedrustapp;

import androidx.appcompat.app.AppCompatActivity;

import android.os.Bundle;
import android.os.Handler;
import android.os.ParcelFileDescriptor;
import android.util.Log;

import java.io.File;
import java.io.FileInputStream;

public class MainActivity extends AppCompatActivity {
    private static final String TAG = "IsolatedMain";
    private Handler mHandler = new Handler();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Log.i(TAG, "Starting");

        mHandler.postDelayed(new Runnable() {
            @Override
            public void run() {

                RustHelloWorld.main("TJ");
                try {
                    File file = new File(getFilesDir(), "test_file");
                    // We don't actually need pfd here (yet) but there's not a way to get
                    // a raw fd to give to rust otherwise? I probably just can't find it.
                    ParcelFileDescriptor pfd = ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY);
                    RustHelloWorld.read_file(pfd.detachFd());
                } catch (Exception e) {
                    Log.e(TAG, "Got exception " + e);
                }

            }
        }, 10000);
    }
}