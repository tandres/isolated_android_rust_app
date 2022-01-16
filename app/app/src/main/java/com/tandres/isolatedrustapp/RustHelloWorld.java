package com.tandres.isolatedrustapp;

import android.util.Log;

public class RustHelloWorld {
    private static final String TAG = "RustHelloWorld";
    private static native String hello(String input);

    static {
        System.loadLibrary("rust");
    }

    public static void main(String name) {
        Log.i(TAG, "Invoking hello from native");
        String output = RustHelloWorld.hello(name);
        Log.i(TAG, "Res: " + output);
    }
}
