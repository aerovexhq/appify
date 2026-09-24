# Extension Studio & Registry

Appify features a powerful modular extension system split into two categories:
- **Plugins (JavaScript)**: Injected on DOM load to alter behavior, automate workflows, or manipulate pages.
- **Themes (CSS)**: Injected to apply dark modes, adjust layouts, or conceal elements.

## Global Registry vs. Per-App Studio

- **Global Registry** (`~/.local/share/appify/registry.json`): A shared library of extensions available across all installed apps.
- **In-App Studio**: Configure which plugins and themes execute for a specific app, customize their parameters, and reorder their execution order.

## Features

- **Embedded Monaco Editor**: Self-contained Monaco code editor with Monarch JavaScript and CSS tokenization, quick snippets, and dark theme.
- **Drag-and-Drop Sequencing**: Drag cards by their grip handles (`⋮⋮`) to change which extension runs first (`#1`, `#2`, etc.).
- **Safe Isolation**: Custom user scripts are executed inside isolated WebKitGTK user content script pools without exposing node or file-system primitives.
