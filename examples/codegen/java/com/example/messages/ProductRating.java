package com.example.messages;

import com.google.gson.annotations.SerializedName;

/**
 * A product rating.
 */
public class ProductRating {

    /**
     * User who submitted the rating.
     */
    @SerializedName("user_id")
    public long userId;

    /**
     * Rating value (1-5 stars).
     */
    public int stars;

    /**
     * Rating comment (optional).
     */
    public String comment;

    /**
     * Rating timestamp (Unix epoch milliseconds).
     */
    public long timestamp;

    public ProductRating() {}

    public ProductRating(long userId, int stars, String comment, long timestamp) {
        this.userId = userId;
        this.stars = stars;
        this.comment = comment;
        this.timestamp = timestamp;
    }

    public long getUserId() {
        return userId;
    }

    public void setUserId(long userId) {
        this.userId = userId;
    }

    public int getStars() {
        return stars;
    }

    public void setStars(int stars) {
        this.stars = stars;
    }

    public String getComment() {
        return comment;
    }

    public void setComment(String comment) {
        this.comment = comment;
    }

    public long getTimestamp() {
        return timestamp;
    }

    public void setTimestamp(long timestamp) {
        this.timestamp = timestamp;
    }
}
