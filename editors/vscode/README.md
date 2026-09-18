# K language support for VS Code

This directory contains TextMate grammars and language configurations for
editing `.k` and x86 GNU assembly files. K highlighting covers the documented
keywords, primitive and qualified pointer types, operators, declarations,
comments, literals, and punctuation. Assembly highlighting covers the `.s`,
`.S`, and `.asm` files used by KrumpyOS, including directives, labels,
registers, instructions, immediates, strings, and comments.

To use it locally during development:

1. Open `K/editors/vscode` as a folder in VS Code.
2. Press `F5` to launch an Extension Development Host.
3. Open a `.k` or assembly file in that new window. Select **K** or
   **K Assembly** as the language if VS Code does not detect it automatically.

The extension has no runtime dependencies. To install it permanently, package
the folder with the VS Code extension tooling and install the resulting `.vsix`
file from **Extensions: Install from VSIX**.

The repository also includes `.gitattributes` with a Linguist override so
GitHub identifies `.k` files as K when the K language definition is available.
GitHub's web highlighter is maintained by Linguist; the TextMate grammar here
is for local editors and can be proposed upstream separately.

This extension is a host-development tool, not the planned KrumpyOS editor.
KrumpyOS will eventually bundle a separate Vim-inspired native editor for
source, configuration, package manifests, and manual pages.
