# Docker Manager for Zed

A Zed IDE extension that lets you create and manage Docker containers directly from the Assistant panel using slash commands.

## Features

| Command | Description | Argument |
|---|---|---|
| `/docker-ps` | List containers (`all` to include stopped) | Optional |
| `/docker-images` | List available images | -- |
| `/docker-start` | Start a stopped container | Container name/ID |
| `/docker-stop` | Stop a running container | Container name/ID |
| `/docker-rm` | Remove a container (`--force` supported) | Container name/ID |
| `/docker-rmi` | Remove an image | Image name/ID |
| `/docker-logs` | View container logs (`--tail N` supported) | Container name/ID |
| `/docker-run` | Run a new container from an image | Image + flags |
| `/docker-stats` | Show container resource usage | -- |
| `/docker-inspect` | Inspect container configuration | Container name/ID |
| `/docker-compose-up` | Start compose services (`-d`, `--build`) | Optional flags |
| `/docker-compose-down` | Stop and remove compose services | Optional flags |
| `/docker-exec` | Execute command in running container | Container + command |
| `/docker-build` | Build image from Dockerfile | Build args |
| `/docker-pull` | Pull image from registry | Image name |

## Smart Completions

The extension provides dynamic argument completions. When typing a command that expects a container name, it automatically queries Docker and suggests:

- **Running containers** for `/docker-stop`, `/docker-logs`, `/docker-exec`, `/docker-inspect`
- **All containers** (including stopped) for `/docker-start`, `/docker-rm`
- **Available images** for `/docker-rmi`, `/docker-run`, `/docker-pull`

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) installed and available in your PATH
- [Zed IDE](https://zed.dev/) installed
- [Rust](https://www.rust-lang.org/tools/install) installed via `rustup` (for development)

## Installation

### As a Dev Extension (for development/testing)

1. Clone this repository:
   ```bash
   git clone https://github.com/huzaifa/zed-docker-manager.git
   ```

2. Open Zed and use the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`)

3. Run `zed: install dev extension`

4. Select the `zed-docker-manager` directory

### From the Extensions Marketplace (coming soon)

Once published, you can install it directly from Zed's Extensions page.

## Usage

1. Open the Zed **Command Palette** (`Ctrl+Shift+P` / `Cmd+Shift+P`).
2. Search for and execute **`agent: new thread`** to open the Agent Panel.
3. Make sure you have authorized the `process:exec` command in Zed if prompted.
4. Test your commands by typing any of the available slash commands, for example:

```text
/docker-ps
/docker-ps all
/docker-images
/docker-run -d --name my-nginx nginx:latest
/docker-stop my-nginx
/docker-logs my-nginx --tail 50
/docker-exec my-nginx ls -la /etc/nginx
/docker-rm --force my-nginx
/docker-compose-up -d
/docker-compose-down
/docker-build -t my-app .
/docker-pull python:3.12
/docker-stats
/docker-inspect my-container
/docker-rmi nginx:latest
```

## Development

### Building

```bash
# Add the WASM target (first time only)
rustup target add wasm32-wasip1

# Build the extension
cargo build --target wasm32-wasip1 --release
```

### Testing

All command handlers, completions, and helpers are unit tested with a mock
Docker executor. No Docker installation is needed to run the tests.

```bash
cargo test
```

### Manual Testing

1. Build the extension
2. In Zed, run `zed: install dev extension` and select this directory
3. Open the Assistant and test the slash commands
4. Check logs with `zed: open log` or run Zed with `zed --foreground`

### Rebuilding

After making changes, run `zed: rebuild extension` in Zed's Command Palette to reload without restarting.

## License

This project is licensed under the [MIT License](LICENSE).
