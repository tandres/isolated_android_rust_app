package com.tandres.isolatedrustapp;

import android.util.Log;

public class RustHelloWorld implements LoggingInterface {
    private static final String TAG = "IsolatedRustHelloWorld";
    private static native String hello(String input);
    private static native String readFileNative(int input);
    private static native void spawnThread(LoggingInterface callback);

    static {
        System.loadLibrary("rust");
    }

    RustHelloWorld() {

    }

    public void start() {
        Log.i(TAG, "Invoking spawnThread");
        RustHelloWorld.spawnThread(this);
        Log.i(TAG, "Completed spawnThread");
    }

    public static void main(String name) {
        Log.i(TAG, "Invoking hello from native");
        String output = RustHelloWorld.hello(name);
        Log.i(TAG, "Res: " + output);
    }

    public static void read_file(int fd) {
        Log.i(TAG, "Invoking file read");
        String output= RustHelloWorld.readFileNative(fd);
        Log.i(TAG, "Completed invocation: " + output);
    }

    public void loggingCallback(String msg) {
        Log.i(TAG, msg);
    }
}
