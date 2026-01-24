package com.example.messages;

import com.google.gson.annotations.SerializedName;

/**
 * An item in an order.
 */
public class OrderItem {

    /**
     * Product SKU.
     */
    public String sku;

    /**
     * Quantity ordered.
     */
    public int quantity;

    /**
     * Price per unit in cents (at time of order).
     */
    @SerializedName("unit_price_cents")
    public long unitPriceCents;

    public OrderItem() {}

    public OrderItem(String sku, int quantity, long unitPriceCents) {
        this.sku = sku;
        this.quantity = quantity;
        this.unitPriceCents = unitPriceCents;
    }

    public String getSku() {
        return sku;
    }

    public void setSku(String sku) {
        this.sku = sku;
    }

    public int getQuantity() {
        return quantity;
    }

    public void setQuantity(int quantity) {
        this.quantity = quantity;
    }

    public long getUnitPriceCents() {
        return unitPriceCents;
    }

    public void setUnitPriceCents(long unitPriceCents) {
        this.unitPriceCents = unitPriceCents;
    }
}
