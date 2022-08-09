# actix-web JSON-formatted Error Middleware

## Description

This library is an [actix-web][actix-web-site] middleware that:
- Changes the `content-type` header from html to `application/json`
- changes the repsonses body from blank to a JSON blob that looks like:
  ```json
    {"error": 404, "message": "error"}
  ```

# Development Environment Setup

Currently this project only requires you to run:
```bash
cargo check
```

# Testing The Library

All tests are unit written in `src/lib.rs` currently to run the tests:

```bash
cargo test
```

# To Contribute

Please fork, edit/create a patch and submit a PR against [the project repo][project-repo]

# File a Bug

Use the [issues-page] of the github repo

[actix-web-site]: https://actix.rs/
[project-repo]: https://github.com/howaboutudance/actix-json-error-middleware
[issues-page]: https://github.com/howaboutudance/actix-json-error-middleware/issues