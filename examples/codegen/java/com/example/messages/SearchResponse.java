package com.example.messages;

import com.google.gson.annotations.SerializedName;
import java.util.List;

/**
 * Search results.
 */
public class SearchResponse {

    /**
     * Total number of results available.
     */
    public long total;

    /**
     * Results for this page.
     */
    public List<Product> results;

    /**
     * Query execution time in milliseconds.
     */
    @SerializedName("query_time_ms")
    public int queryTimeMs;

    public SearchResponse() {}

    public SearchResponse(long total, List<Product> results, int queryTimeMs) {
        this.total = total;
        this.results = results;
        this.queryTimeMs = queryTimeMs;
    }

    public long getTotal() {
        return total;
    }

    public void setTotal(long total) {
        this.total = total;
    }

    public List<Product> getResults() {
        return results;
    }

    public void setResults(List<Product> results) {
        this.results = results;
    }

    public int getQueryTimeMs() {
        return queryTimeMs;
    }

    public void setQueryTimeMs(int queryTimeMs) {
        this.queryTimeMs = queryTimeMs;
    }
}
