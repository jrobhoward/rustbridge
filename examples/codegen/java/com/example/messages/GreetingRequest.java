package com.example.messages;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * A simple greeting request.
 * This demonstrates basic string fields and documentation preservation.
 */
public class GreetingRequest {

    /**
     * The name of the person to greet.
     */
    public String name;

    /**
     * The greeting language code (e.g., "en", "es", "fr").
     * If not provided, defaults to English.
     */
    public String language;

    public GreetingRequest() {}

    public GreetingRequest(String name, String language) {
        this.name = name;
        this.language = language;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getLanguage() {
        return language;
    }

    public void setLanguage(String language) {
        this.language = language;
    }
}
