# Zed XML Language Server

[Zed](https://zed.dev) extension for XML language support powered by [Eclipse LemMinX](https://github.com/eclipse-lemminx/lemminx).

## Features

- **XML Language Support**: Syntax highlighting, code completion, diagnostics, and more via [Eclipse LemMinX](https://github.com/eclipse-lemminx/lemminx)
- **Language Server Protocol**: Full LSP support including hover, go-to-definition, references, and formatting
- **Automatic Binary Download**: Downloads LemMinX native binary from Red Hat's [vscode-xml](https://github.com/redhat-developer/vscode-xml) releases on first use (no manual installation required)
- **Cross-platform**: Linux, macOS (Intel & Apple Silicon), Windows
- **Configurable**: Custom binary path, arguments, and environment variables via Zed settings

## Dependency

This extension depends on Zed's official XML extension for the XML language definition and syntax grammar. Install and enable the official XML extension alongside this extension.

This extension provides only the LemMinX language server integration. It does not bundle its own XML language definition or Tree-sitter grammar.

## Installation

### From Zed Extensions (when published)

1. Install Zed's official [XML extension](https://zed.dev/extensions/xml)
1. Open Zed
1. Press `Cmd+Shift+X` (Extensions panel)
1. Search for "XML Language Server"
1. Click Install

### Development Installation

```bash
# Clone the repository
git clone https://github.com/impure0xntk/zed-xml-language-server
cd zed-xml-language-server

# Enter development environment (requires Nix with flakes)
nix develop

# Or manually install Rust, wasi-sdk, JDK 21, GraalVM

# Build and test
make check

# Install as dev extension in Zed:
# 1. Open Zed
# 2. Press Cmd+Shift+P -> "zed: install dev extension"
# 3. Select this directory
```

## Configuration

Configure via Zed settings (`settings.json`):

```json
{
  "lsp": {
    "lemminx": {
      "binary": {
        "path": "/custom/path/to/lemminx",      // Optional: custom binary path
        "arguments": ["-Xmx2g"],       // Optional: JVM args or native args
        "environment": {                         // Optional: environment variables
          "JAVA_HOME": "/path/to/jdk"
        }
      },
      "settings": {                              // LemMinX settings (workspace/configuration)
        "xml": {
          "server": {
            "workDir": ".lemminx"
          },
          "validation": {
            "enabled": true,
            "downloadSchemas": true
          },
          "completion": {
            "enabled": true
          },
          "format": {
            "enabled": true,
            "splitAttributes": true
          }
        }
      },
      "initializationOptions": {                 // Initialize params only
        "format": { "splitAttributes": true }
      }
    }
  }
}
```

## Binary Resolution

The extension resolves the LemMinX binary in this order:

1. **User-configured path** (`lsp.lemminx.binary.path`)
2. **PATH lookup** (`lemminx` command)
3. **Auto-download** from GitHub Releases / Maven Central

If native binary is not available in releases, it falls back to downloading the uber JAR and running with `java -jar`.

## Supported File Types

- `.xml`, `.xsd`, `.xslt`, `.xsl`
- `.svg`, `.xhtml`, `.plist`
- `.config`, `.props`, `.pom`
- `.nuspec`, `.vcxproj`, `.csproj`, `.fsproj`, `.vbproj`
- `.sqlproj`, `.dbproj`, `.targets`
- `.xml.dist`, `.xml.orig`

## Development

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled (recommended)
- Or manually: Rust (stable), wasi-sdk, JDK 21, GraalVM CE 21

### Commands

```bash
make dev        # Instructions for dev extension install
make test       # Run tests
make build      # Build WASM for Zed
make build-native # Build native binary for testing
make native     # Build LemMinX native binary (requires GraalVM)
make check      # Format + Clippy + Tests
make fmt        # Check formatting
make clippy     # Run clippy
make clean      # Clean artifacts
```

### Nix Flake

```bash
# Enter dev shell
nix develop

# Build package
nix build
```

## Architecture

```text
zed-xml-language-server/
├── extension.toml          # Extension manifest
├── Cargo.toml              # Rust package
├── flake.nix               # Nix development environment
├── src/
│   └── lib.rs              # Extension implementation
└── .github/workflows/
    └── ci.yml              # CI pipeline
```

## How It Works

1. Zed loads this extension through its extension host.
1. The extension resolves the LemMinX executable through user configuration, PATH, or automatic download.
1. Zed starts LemMinX and establishes an LSP connection over stdio.
1. The official XML extension provides the XML language definition and syntax grammar.
1. LemMinX provides language-server features such as completion, diagnostics, hover, navigation, and formatting.

## License

Apache-2.0 License - see [LICENSE](LICENSE) for details.

LemMinX is licensed under EPL-2.0 - see [NOTICE](NOTICE) for third-party licenses.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `make check` to verify
5. Submit a pull request

## References

- [Zed Language Extensions](https://zed.dev/docs/extensions/languages)
- [Zed Extension API](https://docs.rs/zed_extension_api)
- [XML extension](https://zed.dev/extensions/xml)
- [Eclipse LemMinX](https://github.com/eclipse-lemminx/lemminx)
