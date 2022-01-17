package com.tandres.isolatedrustapp;

import android.util.Log;

public class RustHelloWorld {
    private static final String TAG = "IsolatedRustHelloWorld";
    private static native String hello(String input);
    private static native String readFileNative(int input);
    private static native void spawnThread(String tag, int fd) throws Exception;
    private String loggingTag = TAG;
    static {
        System.loadLibrary("rust");
    }

    RustHelloWorld(String tag) {
        this.loggingTag = tag;
    }

    public void start(int fd) {
        try {
            Log.i(this.loggingTag, "Invoking spawnThread");
            RustHelloWorld.spawnThread(this.loggingTag, fd);
            Log.i(this.loggingTag, "Completed spawnThread");
        } catch(Exception e) {
            Log.e(TAG, e.toString());
        }
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
}
