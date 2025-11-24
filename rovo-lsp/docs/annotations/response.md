# @response

Define an API response for different status codes.

## Syntax
```rust
/// @response STATUS TYPE DESCRIPTION
```

## Parameters
- `STATUS`: HTTP status code (100-599)
- `TYPE`: Response type (e.g., `Json<User>`, `Vec<Post>`)
- `DESCRIPTION`: Human-readable description

## Examples
```rust
/// @response 200 Json<User> Successfully retrieved user
/// @response 404 Json<Error> User not found
/// @response 500 Json<Error> Internal server error
```

## Common Status Codes
- `200`: OK - Request succeeded
- `201`: Created - Resource created
- `400`: Bad Request - Invalid input
- `401`: Unauthorized - Authentication required
- `403`: Forbidden - Insufficient permissions
- `404`: Not Found - Resource doesn't exist
- `500`: Internal Server Error - Server error
