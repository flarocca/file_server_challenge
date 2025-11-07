# General challenge

The most challenging aspect of this challenge is to, while akcnowledging ChatGPT and IA exists (and I use it), prove that I know what it is
reflected in this challenge, it is code I wrote (either directly or indirectly) and I understand completely. So I really oriented the whole solution around
being explicit where I use it and why. In any case, I should be able to explain every single line of code in this repository as if IA did not exist.
I started off from 4 premises:

1. IA assistance is a fact, so pretending I didn't use it is pointless. What I find challenging here is how to prove I leveraged IA as opposed
   to just delegate the solution to it (like Stackoverflow used to be a few years ago).
2. Engineers reviewing this code already discounted IA is going to be used. The challenge consists in finding a balance that make reviewers feel
   comfortable with how I leveraged IA. I do practice regularly, so luckily for me I have many examples (either in Rust or C#) already in my Github. I'll try to
   reference to existing code as much as possible.
3. I love coding, so even if I use IA, I still enjoy coding myself, and most importantly, the learnings. I still need to keep a reasonable balance
   between how much I code and how much code does not make any sense to waste time writing.
4. I am scared of IA because of the cognitive damage it can cause delegating the problem solving to it. So I take coding challenges as an opportunity
   to train areas where I usually would not have the need.

# Library

The Custom Merkle Tree implementation here was complete written by me. Writting this kind of algorithms is something I love doing from time to time
to keep my brain in shape (see [this repo](https://github.com/flarocca/algorithm_challenges) and [this one](https://github.com/flarocca/distributed_system_challlenges)).
Once completed, I asked ChatGPT for a review. I took some suggestions but just the ones that made sense to me.

## Pending Task & Improvements

- Feature for tracing
- Support for different hash algorithms

# Server

For the server I used a template I built myself at my current job. Since we run microservices, there is a template repository we use to quickly set up everything.
The original template also includes a crafted bash script that modifies some common properties such as service name, github actions pipelines, helm charts, etc.
I didn't include that scrtipt here. So I leveraged the Server harness I already built in the past to focus on the logic specific needed for the client to work.

I have a strong bias towards how to structure projects so that layers are properly decoupled from each other due to my past on .Net and C# (and I find it very useful
in Rust as well). You can have a look at this old [C# API Template](https://github.com/flarocca/tmenos3.netcore.apidemo)

The project structure is as follows:
- `handlers`: this is the most external layer (presentation layer). It is responsible for validating input and formatting output.
  - `requests`: pure input DTOs
  - `responses`: pure output DTOs
- `services`: business logic layer. If there is a bug or new requirements (related to logic, not visualization), this should be the first place to look at
- `repositories`: persistance layer. No logic other than the strictly need for persistance should be here.
   If the project is big, I would include a `entities` module for all those persistance DTOs.
- `models`: pure domain models. Important, there must exist a separation between domain models and DTOs used for input/output as well as persistance. 
- `infrastructure`: cross cutting concerns such as logging, tracing, configuration, etc.

The expected flow should always be: `handler -> service -> repository`.

One more consideration is that as the challenge is described right now, only one client with one set of files could be added.
I decided to include the concept of Upload ID so that a client can create groups of files to upload.
I imagine this as it would be a sort of torrent system or similar. Additionally, by adding the Upload ID, it is possible to have multiple clients
without modifying (at least not too agressively) the current Server API.

The flow then becomes:
1. Client initiates an Upload to the server
2. Client uploads files for that Upload ID
3. Client completes the Upload ID

When the client wants to download and verify files, it must include the Upload ID.

## Authentication

When a client is added to the server, a new API-KEY and API-SECRET are generated. This feature is not implemented, but I designed the server in that direction.
The client uses the API-SECRET to sign every request with HMAC-SHA256.

The server authenticates every request by reading 3 headers:
- `X-AUTH-TS`: the client must send the actual timestamp in milliseconds
- `X-AUTH-SIGNATURE`: the HMAC-SHA256 signature of the request, signed with the API-SECRET
- `X-AUTH-KEY`: the API-KEY assigned to the client

Right now, the client must sign the timestamp only and not the whole request, in a real production scenario, the client should sign the timestamp along
with the rest request (path, query and body)
The timestamp is a clever feature that not only prevents replay attacks but also works as a TTL mechanism (statelessly, which is very convenient).

I took this approach from Binance Futures API ([API Docs](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info#signed-endpoint-examples-for-post-fapiv1order---hmac-keys)) and Talos API (their documentation is private)

## Database

The server can use a custom In-Memory repository I implemented (nothing too fancy nor performant) or a Clickhouse database.
I chose Clickhouse cause it is what we are mostly using at my current company, so I already had some boilerplate ready to use.
Additinally, Clickhouse allows me to showcase examples of transformations needed between domain models and persistance models, and how
the current architecture decouples layers and implementation details.

## Storage

The server ccan use a custom In-Memory storage I implemented (nothing too fancy nor performant) or AWS S3.
Again, S3 is something I already had boilerplate for at my current company.

## Running the Server

It can be run isolated using one of the following alternatives:

1. `cargo run --bin file_server_server`
2. `cd server && cargo run`

Or using `docker-compose`:

1. `docker-compose -p server up --build`

The server is exposed at `http://localhost:8080` if using the default configuration.
OpenAPI documentation is available at `http://localhost:8080/apidoc/openapi.json`.
Swagger UI is available at `http://localhost:8080/swagger-ui`.

When running via `docker-compose`, some initialization scripts will be executed to create a default S3 bucket and Clickhouse tables.
Additionally, there are two `docker-compose` perofiles:
- `infra`: runs tabix, clickhouse and localstack without running the server. Useful to have the server running for debugging purposes.
- `server`: runs everything including the server.

## Deployment

The server is stateless if feature `persistent` is enabled. This allows to deploy multiple instances behind a load balancer or ingress controller.

## Pending Task & Improvements

- The authentication mechanism is not reflected in Swagger.
- Add health/probe endpoints (`healthy`, `ready` and `started`) for readiness and liveness probes.
- Server signatures: so far the client can verify files have not been tampered once upload to the server, but there is no mechanism for the client to verify
  the server is legit. To support that feature I would include server signatures in the all responses along with a registry of server public keys.
  Finally, the client can validate server signatures using the server public key from the registry.
- ORM for Clickhouse

# Client

For the client I used a template I already built in the past that uses Clap.
It stores hashes locally using files, but it could be easily extended to use SQLite for example.

The client uses some retry and exponential backoff mechanism to be more resilient to transient errors.
I chose `tokio-retry` cause that is the one I used in the past for rust projects and kind manages the same
semmantics that what I used to use back in the .Net world with Polly (see example [here](https://github.com/flarocca/tmenos3.netcore.apidemo/blob/21a0ec99e7622dced0b3c9285b875dc4d1b6a8b5/src/TMenos3.NetCore.ApiDemo.Services/Infrastructure/Extensions.cs#L21)).

## Commands

For the command line interface I used `clap-rs` which I used in the past so it was easier for me to set up quickly.
Clap adds a lot of boilerplate for free such as `--help` and `--version` commands.

To get the list of available commands, just run:

```bash
cargo run -- --help
cargo run -- [command] -h
```

the result will be:

```bash
Usage: file_server_client <COMMAND>

Commands:
  upload-files, --upload-files  This command initiates, uploads and completes an upload flow.
  verify-file, --verify-file    This command verifies that a file for a given index is valid.
  list-upload-ids, --list-upload-ids  List all upload IDs
  help                          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

### Upload Files

This commands executes the flow to upload files to the server.
It uploads all files in the `files-directory` folder and writes the root hash to the `roots-store-directory` folder by creating one `.root` file per upload.
The name of each root file will be the upload id returned by the server, such as `cddc3f80-cb9b-4a1b-9d32-332c2f27abc1.root`.

```
cargo run -- upload-files  -k client-1 -s secret-1
```

Run `cargo run -- upload-files --help` to see all available options.

```bash
Usage: file_server_client {upload-files|--upload-files} [OPTIONS] --api-key <api-key> --api-secret <api-secret>

Options:
  -k, --api-key <api-key>
          API Key for authentication
  -s, --api-secret <api-secret>
          API Secret for authentication
  -u, --base-url <base-url>
          API Secret for authentication [default: http://localhost:8080]
  -f, --files-directory <files-directory>
          Local directory containing files to upload [default: ~/files]
  -r, --roots-store-directory <roots-store-directory>
          Local directory to persist upload roots [default: ~/roots]
  -h, --help
          Print help
```

### Verify Files

This command verifies that a file for a given index is valid. Since Uploads are identified by an ID, the client must provide it along with the index of the file to be validated.

```bash
cargo run -- verify-file -x 0 -i cddc3f80-cb9b-4a1b-9d32-332c2f27abc1 -k client-1 -s secret-1
```

Run `cargo run -- verify-file --help` to see all available options.

```bash
Usage: file_server_client {verify-file|--verify-file} [OPTIONS] --api-key <api-key> --api-secret <api-secret>

Options:
  -k, --api-key <api-key>
          API Key for authentication
  -s, --api-secret <api-secret>
          API Secret for authentication
  -u, --base-url <base-url>
          API Secret for authentication [default: http://localhost:8080]
  -f, --files-directory <files-directory>
          Local directory containing files to upload [default: ~/files]
  -r, --roots-store-directory <roots-store-directory>
          Local directory to persist upload roots [default: ~/roots]
  -i, --id <id>
          Upload ID to verify
  -x, --index <index>
          Index of the file to verify
  -h, --help
          Print help
```

### List Upload IDs

Since I added the concept of Upload IDs, there must be a way for the client to know which Upload IDs.
This command reads all Upload IDs from `.root` files extracting the ids from file names.

```bash
cargo run -- list-upload-ids
```

Run `cargo run -- list-upload-ids --help` to see all available options.

```bash
Usage: file_server_client {list-upload-ids|--list-upload-ids} [OPTIONS]

Options:
  -f, --files-directory <files-directory>
          Local directory containing files to upload [default: ~/files]
  -r, --roots-store-directory <roots-store-directory>
          Local directory to persist upload roots [default: ~/roots]
  -h, --help
          Print help
```


## Pending Task & Improvements

- Add unit tests
- Better display of errors and results
