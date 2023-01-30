# UserClouds rust sample app
This is a very simple starter app for [UserClouds](https://userclouds.com) written in rust.

In order to make getting up and running even faster, we've included some test credentials in the code itself. These credentials won't actually do anything besides allow you to create an account and log into localhost, but
they should make things just a little bit easier.

We've also tried to simplify the code to the bare minimum to see what's going on, so please apply actual software
engineering practices to any code you write that derives from this!

See [Getting Started With Plex](https://documentation.userclouds.com/home/authentication/implement-plex/set-up-plex) for a more detailed guide.

## Getting started
1. Clone the repo 
```bash
git clone git@github.com:tbakerx/userclouds-rust-sample.git && cd userclouds-rust-sample
```
2. Build the app
```bash
cargo build
```
3. Run the server
```bash
cargo run --bin userclouds
```
4. With the server running, navigate to `http://localhost:3000/login` to see the UserClouds sample app login screen
5. Create a test account here and login. On success, you will be redirected to the `/callback` route and will display your `id_token`, `access_token`, `scopes` etc.

