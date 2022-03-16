package com.ncornell.testapp;

import androidx.appcompat.app.AppCompatActivity;

import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;

public class MainActivity extends AppCompatActivity {
    private Handler mHandler = new Handler();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        mHandler.postDelayed(()-> {
            Intent intent = new Intent();
            intent.setAction("com.tandres.isolatedrustapp.MainActivity");
            startActivity(intent);
        }, 5000);
    }
}