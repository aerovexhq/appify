# Community Marketplace

The **Appify Marketplace** is a global catalog of verified user-created plugins and themes hosted on Cloudflare Workers CDN at **`https://cdn.appify.aerovex.net`**.

## Browsing the Marketplace

Open the Appify Manager:
```bash
appify ui
```
Click **Marketplace** in the sidebar. You can:
- Filter by **All**, **Plugins (JS)**, and **Themes (CSS)**.
- Search extensions by title, tags, or description.
- Preview source code before installing with Monaco Editor.
- Install directly to your local registry in 1 click.

## CLI Usage

You can also browse and install extensions directly from the terminal:
```bash
# List all marketplace items
appify marketplace list

# Search for extensions
appify marketplace search "dark"

# Install an extension by ID
appify marketplace install oled-true-black
```

## Contributing an Extension

We accept community contributions via **GitHub Issues** on [aerovexsim/appify](https://github.com/aerovexsim/appify):

1. Go to [New Issue](https://github.com/aerovexsim/appify/issues/new/choose).
2. Choose either **Plugin Submission** or **Theme Submission**.
3. Fill in the name, description, author handle, target domain, and paste your JavaScript or CSS code.
4. Once reviewed, the extension is tested and deployed directly to `cdn.appify.aerovex.net`!
