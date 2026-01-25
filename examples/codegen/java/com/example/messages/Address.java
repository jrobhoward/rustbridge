package com.example.messages;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * A mailing address.
 */
public class Address {

    /**
     * Street address line 1.
     */
    public String street1;

    /**
     * Street address line 2 (optional).
     */
    public String street2;

    /**
     * City name.
     */
    public String city;

    /**
     * State/province code.
     */
    public String state;

    /**
     * Postal/ZIP code.
     */
    @JsonProperty("postal_code")
    public String postalCode;

    /**
     * Country code (ISO 3166-1 alpha-2).
     */
    public String country;

    public Address() {}

    public Address(String street1, String street2, String city, String state, String postalCode, String country) {
        this.street1 = street1;
        this.street2 = street2;
        this.city = city;
        this.state = state;
        this.postalCode = postalCode;
        this.country = country;
    }

    public String getStreet1() {
        return street1;
    }

    public void setStreet1(String street1) {
        this.street1 = street1;
    }

    public String getStreet2() {
        return street2;
    }

    public void setStreet2(String street2) {
        this.street2 = street2;
    }

    public String getCity() {
        return city;
    }

    public void setCity(String city) {
        this.city = city;
    }

    public String getState() {
        return state;
    }

    public void setState(String state) {
        this.state = state;
    }

    public String getPostalCode() {
        return postalCode;
    }

    public void setPostalCode(String postalCode) {
        this.postalCode = postalCode;
    }

    public String getCountry() {
        return country;
    }

    public void setCountry(String country) {
        this.country = country;
    }
}
