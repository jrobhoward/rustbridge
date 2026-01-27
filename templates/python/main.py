#!/usr/bin/env python3
"""rustbridge Python consumer template."""

import json
from dataclasses import dataclass
from rustbridge.core import BundleLoader


# Define your request/response types to match your plugin's API
@dataclass
class EchoRequest:
    message: str


@dataclass
class EchoResponse:
    message: str
    length: int


def main():
    # TODO: Update this path to your .rbp bundle file
    bundle_path = "my-plugin-1.0.0.rbp"

    # BundleLoader.load() extracts the library and returns the plugin directly
    loader = BundleLoader(verify_signatures=False)
    with loader.load(bundle_path) as plugin:
        # Example: Call the "echo" message type
        request = EchoRequest(message="Hello from Python!")
        request_json = json.dumps({"message": request.message})

        response_json = plugin.call("echo", request_json)
        response_dict = json.loads(response_json)
        response = EchoResponse(**response_dict)

        print(f"Response: {response.message}")
        print(f"Length: {response.length}")


if __name__ == "__main__":
    main()
