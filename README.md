# Firebase Admin SDK for Rust

_This is a WIP! 🚧_

An implementation of the Firebase Admin SDK for Rust, primarily for use with my
backend projects that use [Firebase](https://firebase.google.com) services.

Currently, only some of the methods from [Firebase Authentication](https://firebase.google.com/docs/auth) are implemented (see [src/auth.rs](./src/auth.rs)).

Contributions are welcome!

## Install

Via [`cargo-edit`](https://github.com/killercup/cargo-edit):

```bash
cargo add \
  --git https://github.com/stevenxie/firebase-admin-rs \
  --branch master \
  --package firebase-admin \
  firebase_admin
```

Via `Cargo.toml`:

```toml
[dependencies.firebase_admin]
package = "firebase-admin"
git = "https://github.com/stevenxie/firebase-admin-rs"
branch = "master"
```

## Try the Example

Make sure you have the corresponding [Google Application Credentials](https://cloud.google.com/docs/authentication/production) downloaded somewhere (i.e. `.config/google-application-credentials.json`).

Then, write the path to those credentials (as well as your project ID) to a
`.env` file, for example:

```bash
GOOGLE_APPLICATION_CREDENTIALS=.config/google-application-credentials.json
FIREBASE_PROJECT_ID=abcd-1234
```

Now you can run the example!

```bash
cargo run --example auth
```
