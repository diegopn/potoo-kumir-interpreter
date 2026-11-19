<p align="center">
  <img src="https://diegopn.github.io/potoo-site/src/assets/images/language-interpreter-icon.svg?v=1c651ca7" alt="Potoo IDE language interpreter icon" width="120" />
</p>

<h1 align="center">Kumir Interpreter</h1>

<p align="center">A Rust interpreter for the language and executor interfaces documented by Kumir 2.1.0-rc11.</p>

<p align="center">
  <a href="https://github.com/diegopn/potoo-kumir-interpreter/releases">Download</a> ·
  <a href="https://diegopn.github.io/potoo-site/manual/kumir/">Website</a> ·
  <a href="https://diegopn.github.io/potoo-site/manual/kumir/">User manual</a>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white"></a>
  <a href="https://doc.rust-lang.org/cargo/"><img alt="Cargo" src="https://img.shields.io/badge/Cargo-DEA584?style=for-the-badge&logo=rust&logoColor=white"></a>
  <img alt="Rust library" src="https://img.shields.io/badge/Rust_library-4A86CF?style=for-the-badge">
  <img alt="CLI" src="https://img.shields.io/badge/CLI-6A4C93?style=for-the-badge">
</p>

This Rust package is part of the Potoo IDE ecosystem and is distributed independently from the desktop interface. Integration with the IDE has not been implemented or verified.

The crate exposes explicit classic and high-grade profiles.

## Language profiles and core

`parse(source)` selects the classic profile. Use `parse_with_profile(source, Profile::HighGrade)` or the CLI option `--profile high-grade` for the high-grade profile. Both profiles use the 2.1.0-rc11 core syntax. Classic includes the standard libraries for lexical conversion, mathematics, simple string operations, files, and system functions. High-grade also includes the `Строки` library; `использовать Строки` is rejected in classic.

The parser and runtime cover:

- Program preambles, global and local declarations, literal constants, the first algorithm as the main algorithm, value-returning algorithms, recursion, and `арг`, `рез`, `аргрез`, and `арг рез` parameters.
- Integer, real, logical, character, text, complex, color, scan-code, and file values; one-, two-, and three-dimensional tables with declared bounds.
- Arithmetic, comparison, logical, table-index, and string-slice expressions; assignment; typed input with space-separated values and `нс` line breaks; output with `нс`; and `утв`, `дано`, and `надо` assertions and algorithm contracts.
- Conditional and `выбор` branches, `нц пока`, counted and repeat loops, repeat-until (`кц при` / `кц_при`), `выход`, `стоп`, `пауза`, and statement separators.
- The documented compact layouts: semicolon separators; implicit breaks around `нач`, `выбор`, loop and branch delimiters; branch labels split across lines; and the `кц при` repeat-until terminator.
- The reference standard-library groups: `цел_в_лит`, `вещ_в_лит`, `лит_в_цел`, `лит_в_вещ`; the documented math functions and constants; `длин`, `код`, `юникод`, `символ`, and `юнисимвол`; file operations; and the high-grade string algorithms.

The classic and high-grade library inventories follow their respective source indexes. The high-grade profile adds only the documented `Строки` library to the common groups.

## Executor modules

The nine documented modules are recognized: Robot, Draftsman, Grasshopper, Isometric Robot, Turtle, Keyboard, Painter, Vodoley, and Complex Numbers. Their documented calls are represented by module name, normalized command name, arguments, and input/result modes. This preserves the command interface while leaving environment behavior to the embedding application.

The module command families include:

- **Robot:** movement, coloring, wall and cell queries, and the teacher API.
- **Draftsman:** pen state, ink, point/vector movement, text, and the teacher geometry query.
- **Grasshopper:** forward/backward jumps and repainting.
- **Isometric Robot:** movement, turns, coloring, wall/cell queries, and the teacher API.
- **Turtle:** tail state and pixel-distance/degree movement.
- **Keyboard:** key state/code/reset and the documented scan-code constants.
- **Painter:** color construction/decomposition, canvas setup and inspection, pen/brush/font settings, and drawing operations.
- **Vodoley:** filling, emptying, pouring, vessel queries, and the solved query.
- **Complex Numbers:** the `компл` type, documented arithmetic, and `Re` / `Im`.

Implement `ModuleHost` to handle calls for actor modules. The interpreter does not draw canvases, simulate actor fields or vessels, or capture keyboard events. Teacher inspection routines are rejected by default; select the hidden teacher-code context when executing that section:

```rust
use potoo_kumir_interpreter::{ExecutionContext, Interpreter};

let interpreter = Interpreter::new(host)
    .with_execution_context(ExecutionContext::HiddenTeacherCode);
```

## Host services and command line

Implement `Host` for input/output, clock, waiting, and random-number services. Implement `FileHost` for file operations and return it from `Host::file_host()`; the host owns file handles, paths, encodings, and access policy. The interpreter itself does not open user-selected files. `ModuleHost` and `FileHost` are optional; a program that needs an unavailable service receives a source-located runtime error.

The command-line host connects `ввод` and `вывод` to standard input/output. It has no configured file or actor-module host, so programs that use those services fail with a clear runtime error.

Save a minimal program in `example.kum`:

```text
алг
нач
вывод 42
кон
```

Choose the intended language profile:

```sh
cargo run --locked -- --profile classic example.kum
cargo run --locked -- --profile high-grade example.kum
potoo-kumir --profile classic example.kum
potoo-kumir --profile high-grade example.kum
```

The CLI defaults to the classic profile if `--profile` is omitted. Install Rust 1.93 or later to build and install the CLI from this repository:

```sh
cargo build --release --locked --bin potoo-kumir
cargo install --path . --locked --bin potoo-kumir
```

Cargo installs the command in its configured binary directory, usually `~/.cargo/bin`. Ensure that directory is on `PATH` before running the installed command.

### Install a prebuilt release

After a release is published, download `SHA256SUMS` and the archive for your platform from the [GitHub Releases page](https://github.com/diegopn/potoo-kumir-interpreter/releases). The archive names are:

- Linux x86_64 GNU: `potoo-kumir-interpreter-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- Windows x86_64 MSVC: `potoo-kumir-interpreter-<tag>-x86_64-pc-windows-msvc.zip`
- Intel macOS: `potoo-kumir-interpreter-<tag>-x86_64-apple-darwin.tar.gz`
- Apple silicon macOS: `potoo-kumir-interpreter-<tag>-aarch64-apple-darwin.tar.gz`

Replace `<tag>` with the downloaded release tag and use the filename for your platform. The Linux build is made on Ubuntu 22.04 and may require a sufficiently recent glibc. On Linux or macOS, set `archive` to the exact downloaded filename and verify its matching checksum entry before extracting:

```sh
set -euo pipefail
archive="potoo-kumir-interpreter-<tag>-x86_64-unknown-linux-gnu.tar.gz"
matches="$(awk -v archive="$archive" '$2 == archive { count++; line = $0 } END { if (count != 1) exit 1; print line }' SHA256SUMS)"
if command -v sha256sum >/dev/null 2>&1; then
  printf '%s\n' "$matches" | sha256sum --check -
else
  printf '%s\n' "$matches" | shasum -a 256 --check -
fi
mkdir -p "$HOME/.local/bin"
tar -xzf "$archive"
install -m 0755 "${archive%.tar.gz}/potoo-kumir" "$HOME/.local/bin/potoo-kumir"
export PATH="$HOME/.local/bin:$PATH"
```

For Intel or Apple silicon macOS, use the corresponding archive name above. Add the `PATH` line to `~/.profile` or your shell configuration to keep it for future sessions. On Windows PowerShell, use the Windows archive and verify the exact entry before extraction:

```powershell
$archive = "potoo-kumir-interpreter-<tag>-x86_64-pc-windows-msvc.zip"
$checksumEntries = @(Get-Content .\SHA256SUMS | Where-Object { $_ -match "^\w+\s+$([regex]::Escape($archive))$" })
if ($checksumEntries.Count -ne 1) { throw "Expected one SHA256SUMS entry for $archive" }
$expected = ($checksumEntries[0] -split '\s+')[0].ToLowerInvariant()
$actual = (Get-FileHash -Algorithm SHA256 $archive).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "SHA-256 mismatch for $archive" }
Expand-Archive -Path $archive -DestinationPath .
$bundleDir = [IO.Path]::GetFileNameWithoutExtension($archive)
$installDir = Join-Path $env:LOCALAPPDATA "Programs\potoo-kumir"
New-Item -ItemType Directory -Force $installDir | Out-Null
Copy-Item (Join-Path $bundleDir "potoo-kumir.exe") $installDir
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ';') -notcontains $installDir) {
  $updatedPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $installDir } else { "$userPath;$installDir" }
  [Environment]::SetEnvironmentVariable("Path", $updatedPath, "User")
}
$env:Path = "$installDir;$env:Path"
```

The Windows commands add the user-writable install directory to your user `PATH`; open a new terminal to use the command in future sessions.

## Coverage boundaries

This is reference-targeted coverage, not a claim of byte-for-byte or behavioral compatibility with every Kumir distribution. The [reference coverage matrix](REFERENCE-COVERAGE.md) maps the selected manuals to the implementation and its remaining limits:

- Actor environment state, drawing, event delivery, and teacher-program visibility must be supplied by an embedding host. The source manuals mark some Robot, Draftsman, and Isometric Robot inspection routines as teacher-only. The interpreter enforces this at runtime using `ExecutionContext`; the embedding application remains responsible for assigning the correct context to each executed source section.
- Actor calls use a shared dynamic `Value` host interface. Documented argument, output-parameter, and result types are checked before and after host calls. The Robot manual declares `@@верхняя буква`, `@@нижняя буква`, `@@температура`, and `@@радиация` as logical while their prose describes character or numeric values; their result type is therefore left unspecified.
- When multiple enabled modules expose an identical command spelling, dispatch follows the order of `использовать` declarations. The source manuals do not define a conflict rule for that case.
- Ordinary variables and table elements begin undefined. Reading one before assignment raises a runtime error; the parser does not perform flow-sensitive definite-assignment analysis. Declarations in an algorithm body become available at their source position.
- File path safety, encoding details, handle cleanup, and filesystem permissions belong to `FileHost`. The file service is not a sandbox. The interpreter checks returned file-service values against the documented result types.
- Tables are limited to one million elements per declaration by this runtime. Execution also has a one-million-statement step limit and a recursion-depth limit of 1,024.
- Keyboard constants use host key-code integers. The manual names the keys and permits comparisons with integers, but does not define a platform-independent numeric mapping.
- The manuals describe random ranges as closed intervals. `Host::random_number` uses a half-open range so integer functions can include the source upper bound; a real-valued call's exact endpoint behavior depends on the generator.
- The file manual requires handles to be closed before normal program end. `FileHost::finish()` runs after successful execution; a host remains responsible for cleaning up after runtime errors.
- Named `конст` declarations are accepted as an interpreter extension; the selected language reference describes literal constants rather than this declaration form.

The selected reference is Kumir 2.1.0-rc11. Other Kumir releases and dialects are outside the verified profile. Standalone Robot Karel and Robot Karol are separate languages and are not included.

## Reference sources

All source links below are pinned to the `2.1.0-rc11` tag of the Kumir 2 source repository.

- Language profiles: [classic index](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/index-classic.xml), [high-grade index](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/index-highgrade.xml).
- Core: [program structure](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/structure.xml), [identifiers and types](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/indentifiers.xml), [simple commands](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/simple_commands.xml), [compound commands](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/compound_commands.xml).
- Standard libraries: [classic](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/standard_library-classic.xml), [high-grade](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/standard_library-highgrade.xml), [lexical conversion](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/lexical_cast.xml), [mathematics](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/math.xml), [simple strings](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/simple_string.xml), [files](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/files.xml), [system functions](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/sys_func.xml), [high-grade string algorithms](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/language/stringutils.xml).
- Executor modules: [Robot](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorRobot.xml), [Draftsman](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorDraw.xml), [Grasshopper](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorGrasshopper.xml), [Isometric Robot](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorIsometricRobot.xml), [Turtle](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorTurtle.xml), [Keyboard](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorKeyboard.xml), [Painter](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorPainter.xml), [Vodoley](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorVodoley.xml), [Complex Numbers](https://raw.githubusercontent.com/a-a-maly/kumir2/2.1.0-rc11/userdocs/ActorComplexnumbers.xml).

## Support

<p align="left">
  <a href="https://github.com/sponsors/diegopn"><img alt="Become a sponsor" src="https://img.shields.io/badge/Become_a_sponsor-EA4AAA?style=for-the-badge&logo=githubsponsors&logoColor=white"></a>
</p>

## Project policies

- [Code of conduct](CODE_OF_CONDUCT.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [GPL-3.0-or-later license](LICENSE)
