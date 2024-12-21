# Code Historian Documentation

This directory contains the documentation for Code Historian, built using MkDocs with the Material theme.

## Setup

1. Install Python dependencies:
```bash
pip install -r requirements.txt
```

2. Run the documentation server locally:
```bash
# Default port 8000
mkdocs serve

# Custom port
mkdocs serve -a localhost:8080
```

3. Build the documentation:
```bash
mkdocs build
```

## Directory Structure

```
docs/
├── getting-started/     # Installation and setup guides
├── user-guide/         # User documentation
├── development/        # Development documentation
├── status/            # Project status and metrics
├── reference/         # API and configuration reference
└── requirements.txt   # Python dependencies
```

## Contributing to Documentation

1. **Adding New Pages**
   - Create a new .md file in the appropriate directory
   - Add the page to the nav section in mkdocs.yml

2. **Style Guide**
   - Use ATX-style headers (#)
   - Include a single H1 (#) at the top of each page
   - Use Mermaid for diagrams
   - Include code examples where appropriate

3. **Features**
   - Material theme components
   - Mermaid diagrams
   - Code syntax highlighting
   - Search functionality
   - Dark/light mode

4. **Best Practices**
   - Keep pages focused and concise
   - Use admonitions for important notes
   - Include examples and diagrams
   - Keep navigation hierarchy logical
   - Update the nav in mkdocs.yml

## Building and Deployment

The documentation is automatically built and deployed on push to the main branch.

### Local Development
```bash
# Start local server with hot-reload (default port 8000)
mkdocs serve

# Start with custom port
mkdocs serve -a localhost:8080

# Build static site
mkdocs build

# Deploy to GitHub Pages
mkdocs gh-deploy
```

### Writing Tips

1. **Admonitions**
   ```markdown
   !!! note
       Important information here
   ```

2. **Code Blocks**
   ````markdown
   ```python
   def example():
       return "Hello, World!"
   ```
   ````

3. **Mermaid Diagrams**
   ````markdown
   ```mermaid
   graph TD
       A[Start] --> B[End]
   ```
   ````

4. **Tables**
   ```markdown
   | Header 1 | Header 2 |
   |----------|----------|
   | Cell 1   | Cell 2   |
   ```

## Maintenance

- Regular review of documentation accuracy
- Update API documentation when changes occur
- Keep examples up to date
- Monitor and fix broken links
- Update dependencies regularly 