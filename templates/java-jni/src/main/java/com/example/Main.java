package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.jni.JniPluginLoader;
import com.google.gson.Gson;

public class Main {
    // Define your request/response types to match your plugin's API
    static class EchoRequest {
        String message;
        EchoRequest(String message) { this.message = message; }
    }

    static class EchoResponse {
        String message;
        int length;
    }

    private static final Gson gson = new Gson();

    public static void main(String[] args) throws Exception {
        // TODO: Update this path to your .rbp bundle file
        String bundlePath = "my-plugin-1.0.0.rbp";

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)  // Set true for production
            .build();

        String libraryPath = bundleLoader.extractLibrary().toString();

        try (Plugin plugin = JniPluginLoader.load(libraryPath)) {
            // Example: Call the "echo" message type
            EchoRequest request = new EchoRequest("Hello from Java JNI!");
            String requestJson = gson.toJson(request);

            String responseJson = plugin.call("echo", requestJson);
            EchoResponse response = gson.fromJson(responseJson, EchoResponse.class);

            System.out.println("Response: " + response.message);
            System.out.println("Length: " + response.length);
        }

        bundleLoader.close();
    }
}
