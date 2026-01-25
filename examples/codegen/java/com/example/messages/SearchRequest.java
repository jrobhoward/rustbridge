package com.example.messages;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

/**
 * Search query and filters.
 * Demonstrates optional complex types.
 */
public class SearchRequest {

    /**
     * Search query string.
     */
    public String query;

    /**
     * Maximum number of results to return.
     */
    public int limit;

    /**
     * Offset for pagination.
     */
    public int offset;

    /**
     * Categories to filter by (optional).
     */
    public List<String> categories;

    /**
     * Minimum price in cents (optional).
     */
    @JsonProperty("min_price_cents")
    public Long minPriceCents;

    /**
     * Maximum price in cents (optional).
     */
    @JsonProperty("max_price_cents")
    public Long maxPriceCents;

    public SearchRequest() {}

    public SearchRequest(String query, int limit, int offset, List<String> categories, Long minPriceCents, Long maxPriceCents) {
        this.query = query;
        this.limit = limit;
        this.offset = offset;
        this.categories = categories;
        this.minPriceCents = minPriceCents;
        this.maxPriceCents = maxPriceCents;
    }

    public String getQuery() {
        return query;
    }

    public void setQuery(String query) {
        this.query = query;
    }

    public int getLimit() {
        return limit;
    }

    public void setLimit(int limit) {
        this.limit = limit;
    }

    public int getOffset() {
        return offset;
    }

    public void setOffset(int offset) {
        this.offset = offset;
    }

    public List<String> getCategories() {
        return categories;
    }

    public void setCategories(List<String> categories) {
        this.categories = categories;
    }

    public Long getMinPriceCents() {
        return minPriceCents;
    }

    public void setMinPriceCents(Long minPriceCents) {
        this.minPriceCents = minPriceCents;
    }

    public Long getMaxPriceCents() {
        return maxPriceCents;
    }

    public void setMaxPriceCents(Long maxPriceCents) {
        this.maxPriceCents = maxPriceCents;
    }
}
