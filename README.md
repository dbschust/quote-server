Author: Daniel Schuster, dschust@pdx.edu (with large contributions by Professor Bart Massey)

This is a full-stack web server written in Rust
that displays famous quotes.

- When initially running the code, you can run the following to initialize a database with some quotes:
```
cargo run --release -- --init-from assets/static/quotes.json
```

- The server uses Axum/Askama/Tokio/Sqlx/Sqlite, and the code is located in the server directory.  To run the server, use the following command from the root directory of the repository:
```
cargo run -p quote-server
```

- The client uses Leptos, and the code is located in the quote-client directory.  To run the client, use the following command from the quote-client directory:
```
trunk serve --open
```

- Adding a new quote to the database can be done by modifying the bash script `server/src/add-quote.sh` to contain the information for the desired quote, then running the script (while the server is running).