package com.example.messages;

import com.google.gson.annotations.SerializedName;
import java.util.List;

/**
 * A product in an e-commerce system.
 * Demonstrates numeric types and serde rename.
 */
public class Product {

    /**
     * Product SKU (Stock Keeping Unit).
     */
    @SerializedName("sku")
    public String stockKeepingUnit;

    /**
     * Product name.
     */
    public String name;

    /**
     * Product description (optional).
     */
    public String description;

    /**
     * Price in cents (to avoid floating point precision issues).
     */
    @SerializedName("price_cents")
    public long priceCents;

    /**
     * Available quantity.
     */
    public int quantity;

    /**
     * Product categories.
     */
    public List<String> categories;

    /**
     * Product ratings.
     */
    public List<ProductRating> ratings;

    public Product() {}

    public Product(String stockKeepingUnit, String name, String description, long priceCents, int quantity, List<String> categories, List<ProductRating> ratings) {
        this.stockKeepingUnit = stockKeepingUnit;
        this.name = name;
        this.description = description;
        this.priceCents = priceCents;
        this.quantity = quantity;
        this.categories = categories;
        this.ratings = ratings;
    }

    public String getStockKeepingUnit() {
        return stockKeepingUnit;
    }

    public void setStockKeepingUnit(String stockKeepingUnit) {
        this.stockKeepingUnit = stockKeepingUnit;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public long getPriceCents() {
        return priceCents;
    }

    public void setPriceCents(long priceCents) {
        this.priceCents = priceCents;
    }

    public int getQuantity() {
        return quantity;
    }

    public void setQuantity(int quantity) {
        this.quantity = quantity;
    }

    public List<String> getCategories() {
        return categories;
    }

    public void setCategories(List<String> categories) {
        this.categories = categories;
    }

    public List<ProductRating> getRatings() {
        return ratings;
    }

    public void setRatings(List<ProductRating> ratings) {
        this.ratings = ratings;
    }
}
