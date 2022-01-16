package com.tandres.isolatedrustapp;

import androidx.appcompat.app.AppCompatActivity;

import android.os.Bundle;
import android.os.Handler;
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
        RustHelloWorld.main("TJ");

        mHandler.postDelayed(new Runnable() {
            @Override
            public void run() {
                try {
                    try {
                        FileInputStream fis = new FileInputStream(new File( getFilesDir(),"test_file"));
                        byte[] buf = new byte[100];
                        fis.read(buf);
                        Log.i(TAG, "BUF IS: " + new String(buf));
                    }catch (Exception e) {
                        Log.e(TAG, e.toString());
                    }

                    //String res = RustHelloWorld.read_file(fis.getFD());
                    /*if (res != null) {
                        Log.i(TAG, "OUTPUT RESULT: " + res);
                    }*/
                } catch(Exception e) {
                    Log.e(TAG, "Got exception " + e);
                }

            }
        }, 10000);
    }
}