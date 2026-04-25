# Go (Standard Comments with Tags)

## Example Module
```go
// @module FileServer
// @see Protocol definition in {@link protocol.go}

// UploadHandle handles file uploads.
// @param r - HTTP request containing multipart/form-data
// @modifies LocalFileSystem - writes to /tmp/uploads
// @pre request must contain a valid X-Auth-Token
func UploadHandle(w http.ResponseWriter, r *http.Request) { ... }
```
