import os
import glob

# Paths
base_dir = "/Users/mohamedahmed/Downloads/Projects/dev/vidra/optionaldeps-release"

replacements = {
    "@YOUR_SCOPE/YOUR_TOOL": "@sansavision/vidra",
    "@YOUR_SCOPE/": "@sansavision/",
    "YOUR_TOOL": "vidra",
    "your-tool": "vidra",
}

for filepath in glob.glob(f"{base_dir}/**/*.json", recursive=True) + \
                glob.glob(f"{base_dir}/**/*.js", recursive=True) + \
                glob.glob(f"{base_dir}/**/*.sh", recursive=True) + \
                glob.glob(f"{base_dir}/**/*.md", recursive=True):
    with open(filepath, "r") as f:
        content = f.read()

    new_content = content
    for old, new in replacements.items():
        new_content = new_content.replace(old, new)
        
    if "your-tool-mcp" in new_content:
        new_content = new_content.replace("vidra-mcp", "")

    if content != new_content:
        with open(filepath, "w") as f:
            f.write(new_content)

# Rename bin files
bin_dir = os.path.join(base_dir, "npm", "main-wrapper", "bin")
if os.path.exists(os.path.join(bin_dir, "your-tool.js")):
    os.rename(os.path.join(bin_dir, "your-tool.js"), os.path.join(bin_dir, "vidra.js"))
if os.path.exists(os.path.join(bin_dir, "your-tool-mcp.js")):
    os.remove(os.path.join(bin_dir, "your-tool-mcp.js"))

print("Replacements complete.")
