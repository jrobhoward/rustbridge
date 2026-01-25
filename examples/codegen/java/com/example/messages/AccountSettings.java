package com.example.messages;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * User account settings.
 * Demonstrates nested custom types.
 */
public class AccountSettings {

    /**
     * Enable email notifications.
     */
    @JsonProperty("email_notifications")
    public boolean emailNotifications;

    /**
     * Enable SMS notifications.
     */
    @JsonProperty("sms_notifications")
    public boolean smsNotifications;

    /**
     * Preferred language code.
     */
    public String language;

    /**
     * Preferred timezone (IANA timezone identifier).
     */
    public String timezone;

    public AccountSettings() {}

    public AccountSettings(boolean emailNotifications, boolean smsNotifications, String language, String timezone) {
        this.emailNotifications = emailNotifications;
        this.smsNotifications = smsNotifications;
        this.language = language;
        this.timezone = timezone;
    }

    public boolean getEmailNotifications() {
        return emailNotifications;
    }

    public void setEmailNotifications(boolean emailNotifications) {
        this.emailNotifications = emailNotifications;
    }

    public boolean getSmsNotifications() {
        return smsNotifications;
    }

    public void setSmsNotifications(boolean smsNotifications) {
        this.smsNotifications = smsNotifications;
    }

    public String getLanguage() {
        return language;
    }

    public void setLanguage(String language) {
        this.language = language;
    }

    public String getTimezone() {
        return timezone;
    }

    public void setTimezone(String timezone) {
        this.timezone = timezone;
    }
}
