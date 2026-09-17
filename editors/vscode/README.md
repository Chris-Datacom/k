# K language support for VS Code

This directory contains a small TextMate grammar and language configuration
for editing `.k` files. It provides syntax highlighting, comments, bracket
matching, and automatic quote/bracket pairing.

To install it locally during development:

1. Open `editors/vscode` as a VS Code extension folder.
2. Run `npm install` only if packaging tools are added later; this grammar has
   no runtime dependencies.
3. Press `F5` from the extension folder, or package the folder with the VS
   Code extension tooling.

The repository also includes `.gitattributes` with a Linguist override so
GitHub identifies `.k` files as K when the K language definition is available.
GitHub's web highlighter is maintained by Linguist; the TextMate grammar here
is for local editors and can be proposed upstream separately.
