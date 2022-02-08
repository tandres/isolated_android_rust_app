package com.tandres.isolatedrustapp;

import android.app.Service;
import android.content.Intent;
import android.os.IBinder;
import android.os.ParcelFileDescriptor;
import android.os.RemoteException;
import android.util.Log;

public class IsolatedRustService extends Service {
    private static final String TAG = "IsolatedService";
    private RustHelloWorld mRust = null;

    public IsolatedRustService() {
    }

    @Override
    public void onCreate() {
        Log.i(TAG, "Created!");
        this.mRust = new RustHelloWorld("IsolatedRustInner");
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

    private final IIsolatedRustInterface.Stub mBinder = new IIsolatedRustInterface.Stub() {
        @Override
        public void say_hello() throws RemoteException {
            RustHelloWorld.main("From Isolated!");
        }

        @Override
        public int getPid() {
            return android.os.Process.myPid();
        }

        @Override
        public void readFile(ParcelFileDescriptor pfd) {
            RustHelloWorld.read_file(pfd.detachFd());
        }

        @Override
        public void start(ParcelFileDescriptor pfd) {
            int fd = pfd.detachFd();
            Log.i(TAG, "Starting rust process with fd: " + fd);
            mRust.start(fd);
        }
    };
}