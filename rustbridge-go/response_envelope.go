package rustbridge

import "encoding/json"

// ResponseEnvelope wraps a JSON response from the FFI transport layer.
type ResponseEnvelope struct {
	Status       string          `json:"status"`
	Payload      json.RawMessage `json:"payload,omitempty"`
	ErrorCode    *uint32         `json:"error_code,omitempty"`
	ErrorMessage *string         `json:"error_message,omitempty"`
}

// IsSuccess returns true if the response indicates success.
func (r *ResponseEnvelope) IsSuccess() bool {
	return r.Status == "success"
}

// ToError converts an error response to a PluginError.
// Returns nil if the response is successful.
func (r *ResponseEnvelope) ToError() *PluginError {
	if r.IsSuccess() {
		return nil
	}

	code := ErrorCodeInternal
	if r.ErrorCode != nil {
		code = ErrorCode(*r.ErrorCode)
	}

	msg := "unknown error"
	if r.ErrorMessage != nil {
		msg = *r.ErrorMessage
	}

	return &PluginError{Code: code, Message: msg}
}

// PayloadJSON returns the payload as a JSON string.
func (r *ResponseEnvelope) PayloadJSON() string {
	if r.Payload == nil {
		return "null"
	}
	return string(r.Payload)
}

// parseResponseEnvelope parses an FfiBuffer's JSON data into a ResponseEnvelope.
func parseResponseEnvelope(data []byte) (*ResponseEnvelope, error) {
	var env ResponseEnvelope
	if err := json.Unmarshal(data, &env); err != nil {
		return nil, &PluginError{
			Code:    ErrorCodeSerializationError,
			Message: "failed to parse response JSON: " + err.Error(),
		}
	}
	return &env, nil
}
