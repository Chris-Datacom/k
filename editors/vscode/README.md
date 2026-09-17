# K language support for VS Code

This directory contains a small TextMate grammar and language configuration
for editing `.k` files. It provides syntax highlighting, comments, bracket
matching, and automatic quote/bracket pairing.

To use it locally during development:

1. Open `K/editors/vscode` as a folder in VS Code.
2. Press `F5` to launch an Extension Development Host.
3. Open any `.k` file in that new window and select **K** as the language if
   VS Code does not detect it automatically.

The extension has no runtime dependencies. To install it permanently, package
the folder with the VS Code extension tooling and install the resulting `.vsix`
file from **Extensions: Install from VSIX**.

The repository also includes `.gitattributes` with a Linguist override so
GitHub identifies `.k` files as K when the K language definition is available.
GitHub's web highlighter is maintained by Linguist; the TextMate grammar here
is for local editors and can be proposed upstream separately.
