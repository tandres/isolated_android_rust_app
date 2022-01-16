package com.tandres.isolatedrustapp;

import android.util.Log;

public class RustHelloWorld {
    private static final String TAG = "IsolatedRustHelloWorld";
    private static native String hello(String input, int num);
    private static native String hello_hello(String input, int num);
    private static native String readFileNative(int input);

    static {
        System.loadLibrary("rust");
    }

    public static void main(String name) {
        Log.i(TAG, "Invoking hello from native");
        String output = RustHelloWorld.hello(name, 0);
        Log.i(TAG, "Res: " + output);
    }

    public static void read_file(int fd) {
        Log.i(TAG, "Invoking file read");
        String output= RustHelloWorld.readFileNative(fd);
        Log.i(TAG, "Completed invocation: " + output);
    }
}
