# Vidra VSCode Extension

This extension provides syntax highlighting and basic language support for the `VidraScript` (`.vidra`) programmable video language.

## Installation Methods

There are two primary ways to install the Vidra VSCode extension depending on your needs.

### Option 1: Symlink (Best for Active Core Development)
If you frequently modify the language grammar and want changes to reflect automatically without having to repackage the extension every time, you should use a symbolic link.

Run the following commands in your terminal:

```bash
# 1. Navigate to the extension's directory
cd path/to/vidra/editors/vscode

# 2. Install the necessary NPM dependencies
npm install

# 3. Create a symbolic link pointing from your global VSCode extensions folder to this directory
ln -s $(pwd) ~/.vscode/extensions/vidra-vscode
```

After doing this, you must **restart VSCode** or run the `Developer: Reload Window` command from the Command Palette (`Cmd+Shift+P` on Mac). Any `.vidra` file you open will now instantly pick up the official syntax highlighting.

---

### Option 2: Build a Packaged `.vsix` (Best for Standard Installation)
If you want to create the final installer file (`.vsix`) so you can distribute it to end-users or simply install it cleanly without linking to your source folder:

```bash
# 1. Navigate to the extension's directory
cd path/to/vidra/editors/vscode

# 2. Install the necessary NPM dependencies
npm install

# 3. Install the VSCode Extension Manager tool globally (if you don't already have it)
npm install -g @vscode/vsce

# 4. Package the extension (this creates a .vsix file in the current directory)
vsce package --no-dependencies

# 5. Install the generated .vsix file directly into your VSCode
code --install-extension vidra-vscode-0.1.0.vsix
```

*(Note: after either method, it is always recommended to perform a full window reload in VSCode by hitting `Cmd+Shift+P` and running `Developer: Reload Window` so it explicitly registers the new language grammar!)*
