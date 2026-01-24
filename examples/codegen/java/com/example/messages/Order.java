package com.example.messages;

import com.google.gson.annotations.SerializedName;
import java.util.List;

/**
 * Order information.
 * Demonstrates complex nested structures with multiple custom types.
 */
public class Order {

    /**
     * Unique order identifier.
     */
    @SerializedName("order_id")
    public long orderId;

    /**
     * Customer user ID.
     */
    @SerializedName("customer_id")
    public long customerId;

    /**
     * Order items.
     */
    public List<OrderItem> items;

    /**
     * Shipping address.
     */
    @SerializedName("shipping_address")
    public Address shippingAddress;

    /**
     * Billing address (optional, may be same as shipping).
     */
    @SerializedName("billing_address")
    public Address billingAddress;

    /**
     * Order total in cents.
     */
    @SerializedName("total_cents")
    public long totalCents;

    /**
     * Order status.
     */
    public String status;

    /**
     * Order creation timestamp (Unix epoch milliseconds).
     */
    @SerializedName("created_at")
    public long createdAt;

    public Order() {}

    public Order(long orderId, long customerId, List<OrderItem> items, Address shippingAddress, Address billingAddress, long totalCents, String status, long createdAt) {
        this.orderId = orderId;
        this.customerId = customerId;
        this.items = items;
        this.shippingAddress = shippingAddress;
        this.billingAddress = billingAddress;
        this.totalCents = totalCents;
        this.status = status;
        this.createdAt = createdAt;
    }

    public long getOrderId() {
        return orderId;
    }

    public void setOrderId(long orderId) {
        this.orderId = orderId;
    }

    public long getCustomerId() {
        return customerId;
    }

    public void setCustomerId(long customerId) {
        this.customerId = customerId;
    }

    public List<OrderItem> getItems() {
        return items;
    }

    public void setItems(List<OrderItem> items) {
        this.items = items;
    }

    public Address getShippingAddress() {
        return shippingAddress;
    }

    public void setShippingAddress(Address shippingAddress) {
        this.shippingAddress = shippingAddress;
    }

    public Address getBillingAddress() {
        return billingAddress;
    }

    public void setBillingAddress(Address billingAddress) {
        this.billingAddress = billingAddress;
    }

    public long getTotalCents() {
        return totalCents;
    }

    public void setTotalCents(long totalCents) {
        this.totalCents = totalCents;
    }

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public long getCreatedAt() {
        return createdAt;
    }

    public void setCreatedAt(long createdAt) {
        this.createdAt = createdAt;
    }
}
