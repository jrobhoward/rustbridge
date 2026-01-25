"""Response envelope for parsing JSON responses from FFI."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any

from rustbridge.core.plugin_exception import PluginException


@dataclass
class ResponseEnvelope:
    """
    Response envelope wrapping a response from FFI transport.

    Attributes:
        status: Response status ("success" or "error").
        payload: Response payload (on success) or None.
        error_code: Error code (on failure) or None.
        error_message: Error message (on failure) or None.
        request_id: Original request ID for correlation (optional).
    """

    status: str
    payload: Any | None = None
    error_code: int | None = None
    error_message: str | None = None
    request_id: int | None = None

    @property
    def is_success(self) -> bool:
        """Check if this is a success response."""
        return self.status == "success"

    @classmethod
    def from_json(cls, json_str: str) -> ResponseEnvelope:
        """
        Parse a ResponseEnvelope from JSON string.

        Args:
            json_str: The JSON string.

        Returns:
            The parsed ResponseEnvelope.

        Raises:
            PluginException: If parsing fails.
        """
        try:
            data = json.loads(json_str)
        except json.JSONDecodeError as e:
            raise PluginException(f"Failed to parse response JSON: {e}") from e

        return cls(
            status=data.get("status", "error"),
            payload=data.get("payload"),
            error_code=data.get("error_code"),
            error_message=data.get("error_message"),
            request_id=data.get("request_id"),
        )

    @classmethod
    def from_bytes(cls, data: bytes) -> ResponseEnvelope:
        """
        Parse a ResponseEnvelope from bytes.

        Args:
            data: The JSON bytes.

        Returns:
            The parsed ResponseEnvelope.
        """
        return cls.from_json(data.decode("utf-8"))

    def get_payload_json(self) -> str:
        """
        Get the payload as a JSON string.

        Returns:
            The payload serialized as JSON, or "null" if no payload.
        """
        if self.payload is None:
            return "null"
        return json.dumps(self.payload)

    def to_exception(self) -> PluginException:
        """
        Convert this error response to a PluginException.

        Returns:
            A PluginException with the error details.
        """
        return PluginException(
            message=self.error_message or "Unknown error",
            error_code=self.error_code or 1,
        )

    def unwrap(self) -> Any:
        """
        Unwrap the payload, raising an exception if this is an error response.

        Returns:
            The payload value.

        Raises:
            PluginException: If this is an error response.
        """
        if not self.is_success:
            raise self.to_exception()
        return self.payload
