package com.example.messages;

import com.google.gson.annotations.SerializedName;
import java.util.List;

/**
 * User profile information.
 * Demonstrates various field types including optional fields,
 * custom types, and collections.
 */
public class UserProfile {

    /**
     * Unique user identifier.
     */
    public long id;

    /**
     * User's display name.
     * This field uses snake_case in Rust but will be converted
     * to camelCase in Java: displayName
     */
    @SerializedName("display_name")
    public String displayName;

    /**
     * User's email address (optional).
     */
    public String email;

    /**
     * User's age in years (optional).
     */
    public Integer age;

    /**
     * List of tags associated with the user.
     */
    public List<String> tags;

    /**
     * User's account settings.
     */
    public AccountSettings settings;

    public UserProfile() {}

    public UserProfile(long id, String displayName, String email, Integer age, List<String> tags, AccountSettings settings) {
        this.id = id;
        this.displayName = displayName;
        this.email = email;
        this.age = age;
        this.tags = tags;
        this.settings = settings;
    }

    public long getId() {
        return id;
    }

    public void setId(long id) {
        this.id = id;
    }

    public String getDisplayName() {
        return displayName;
    }

    public void setDisplayName(String displayName) {
        this.displayName = displayName;
    }

    public String getEmail() {
        return email;
    }

    public void setEmail(String email) {
        this.email = email;
    }

    public Integer getAge() {
        return age;
    }

    public void setAge(Integer age) {
        this.age = age;
    }

    public List<String> getTags() {
        return tags;
    }

    public void setTags(List<String> tags) {
        this.tags = tags;
    }

    public AccountSettings getSettings() {
        return settings;
    }

    public void setSettings(AccountSettings settings) {
        this.settings = settings;
    }
}
