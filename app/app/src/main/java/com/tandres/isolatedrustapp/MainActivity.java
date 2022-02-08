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

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileReader;

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

        mRustObject =  new RustHelloWorld("IsolatedRustOuter");

        mHandler.postDelayed(() -> {
            Log.i(TAG, "Binding service");
            Intent intent = new Intent(MainActivity.this, IsolatedRustService.class);
            bindService(intent, mConnection, Context.BIND_AUTO_CREATE);
        }, 5000);
    }

    private void startOuter(ParcelFileDescriptor pfd) {
        int fd = pfd.detachFd();
        Log.i(TAG, "Starting outer process with fd: " + fd);
        mRustObject.start(fd);
    }

    private void readChildStat(int pid) {
        try {
            String filepath = "/proc/" + pid + "/stat";
            BufferedReader stat = new BufferedReader(new FileReader(filepath));
            String line = stat.readLine();
            Log.d(TAG, "Stat: " + line);
        }
        catch (Exception e) {
            Log.e(TAG, "Couldn't read stat for " + pid + ": " + e);
        }
    }

    private final ServiceConnection mConnection = new ServiceConnection() {
        public void onServiceConnected(ComponentName className, IBinder service) {
            Log.i(TAG, "Service connected");
            iIsolatedRustInterface = IIsolatedRustInterface.Stub.asInterface(service);
            mHandler.post(() -> {
                Log.i(TAG, "Starting Processes");
                try {
                    ParcelFileDescriptor[] pairs = ParcelFileDescriptor.createSocketPair();
                    MainActivity.this.startOuter(pairs[1]);
                    int pid = iIsolatedRustInterface.getPid();
                    Log.i(TAG, "PID: " + pid);
                    File statdir = new File("/proc");
                    try {
                        for (File file : statdir.listFiles()) {
                            Log.i(TAG, file.getName());
                        }
                    } catch(Exception e) {
                        Log.e(TAG, e.toString());
                    }
                    readChildStat(android.os.Process.myPid());
                    iIsolatedRustInterface.start(pairs[0]);
                    mHandler.postDelayed(() -> {
                        readChildStat(pid);
                    }, 5000);
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