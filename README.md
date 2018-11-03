# Jug Lottery (v3)

Rust implementation of Montpellier Jug lottery (third version).  
This implementation is a project to demonstrate the utilisation of several crates :
 - [Actix-web](https://github.com/actix/actix-web)
 - [Actix](https://github.com/actix/actix)
 - [Failure](https://github.com/rust-lang-nursery/failure)
 - [Serde](https://github.com/serde-rs/serde)
 - [Reqwest](https://github.com/seanmonstar/reqwest)
 - [Diesel](https://github.com/diesel-rs/diesel)

## Configuration

The application should run with the following env vars :
 - `ORGANIZER_TOKEN` : Organization id in eventbrite
 - `EVENTBRITE_TOKEN` : Personal OAuth Token in eventbrite
 - `RUST_LOG` : Log level (example : `info`)
 - `DATABASE_URL` : Database SQLite url (example : `test.db`)

## Development

The databases migrations will be executed by the application on startup.  
If you __want__ to executed the migrations manually (to check your scripts) :
```bash
# DATABASE_URL env var is mandatory
cargo install diesel_cli # to install diesel command
diesel setup # initialize database if needed
diesel migration run
```

### Requirements
Needed packages for build:
 - `libssl`
 - `libsqlite3`

### Docker
The application can be build in Docker [Dockerfile](Dockerfile) : `docker build -t lottery-jug-actix .`

Run the application : `docker run -e ORGANIZER_TOKEN=91672475603 -e EVENTBRITE_TOKEN=E7N7QDHSXM2V2YB6AC2I -p 8088:8088 lottery-jug-actix`

The `DATABASE_URL` is by default on `/var/data/lottery-jug/lottery.db`.  
The `RUST_LOG` is be default set to `info`.


## API

### Draw winners 
`GET` -> `/winners?nb=X`

__Results__ : 
 - `200` : 
```json
[
  {
    "first_name": "Francois",
    "last_name": "Teychene"
  },
  {
    "first_name": "Jean-Luc",
    "last_name": "Racine"
  },
  {
    "first_name": "Renard",
    "last_name": "Chenapan"
  }
]
```
 - `400` : Invalid parameter
 - `503` : No live events
 - `500` : Unxepected error

### Record a winner
`POST` -> `/record`

_Body_ : 
```json
{
  "first_name": "Francois",
  "last_name": "Teychene"
}
```

__Results__ : 
 - `200` : 
 ```json
{
    "id": "b3f0182e-b2f4-47a2-9c6f-9ea3a67b588c",
    "first_name": "Francois",
    "last_name": "Teychene",
    "event_id": "52097259305"
}
```
 - `500` : Unexpected error