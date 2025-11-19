[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/xGnTrW1S)
[![Open in Codespaces](https://classroom.github.com/assets/launch-codespace-2972f46106e565e64193e422d61a12cf1da4916b45550586e14ef0a7c637dd04.svg)](https://classroom.github.com/open-in-codespaces?assignment_repo_id=20873211)

# LogSmart - COMP2003-2025-2026

A security log management system for tracking and analyzing security events.

## 🔗 Links

- **Production:** [https://logsmart.app/](https://logsmart.app/)
- **API:** [https://logsmart.app/api/](https://logsmart.app/api/)
- **API Docs:** [https://logsmart.app/api/swagger-ui](https://logsmart.app/api/swagger-ui)
- **Direct API:** [https://api.logsmart.app/](https://api.logsmart.app/)
- **Repository:** [https://github.com/Plymouth-University/comp2003-2025-2026-group-2](https://github.com/Plymouth-University/comp2003-2025-2026-group-2)

## 🛠️ Tech Stack

### Front-end
- **Framework:** SvelteKit
- **Styling:** Tailwind CSS
- **Runtime:** Bun
- **Hosting:** Cloudflare Pages

### Back-end
- **Language:** Rust
- **Framework:** Axum
- **Databases:**
  - SQLite (authentication, security logs, users)
  - MongoDB (templates, customer logs)
- **Hosting:** Oracle VPS (ARM64)

## 🚀 Quick Start

### Prerequisites
- **Front-end:** [Bun](https://bun.sh/)
- **Back-end:** [Nix](https://nixos.org/) (for development environment)

### Local Development

**Front-end:**
```bash
cd SourceCode/front-end/logsmart
bun install
bun run dev
```

**Back-end:**
```bash
cd SourceCode/back-end
nix develop
cargo run
```

## 📋 Development Workflow

### Git Branching Strategy
- `main` - Development branch for integration testing
- `prod` - Production branch (triggers automatic deployment)
- `feature/*` - Feature branches

### Commit Message Convention
Format: `Topic: Message`

Examples:
- `Front-End: Add dashboard component`
- `Back-End: Implement user authentication`
- `CI/CD: Update deployment workflow`

### Branch Management Rules
1. Always work on a feature branch
2. Merge feature branch to `main` for integration testing
3. Only merge `main` to `prod` after complete testing
4. Production deployments are triggered automatically on push to `prod`

### Common Git Commands

**Setup git graph alias:**
```bash
git config --global alias.graph "log --all --decorate --oneline --graph"
```

**Update feature branch with latest main:**
```bash
git stash                      # Stash uncommitted changes
git fetch                      # Fetch from origin
git switch main                # Switch to main
git pull                       # Pull latest changes
git switch my-feature-branch   # Switch back to feature branch
git rebase main                # Rebase onto updated main
git push --force-with-lease    # Force push rebased branch
git stash pop                  # Restore uncommitted changes
```

**Merge feature branch into main:**
```bash
git fetch                      # Fetch from origin
git switch main                # Switch to main
git pull                       # Update main
git merge my-feature-branch    # Merge feature branch
git push                       # Push to origin
```

**Deploy to production:**
```bash
git fetch                      # Fetch from origin
git switch prod                # Switch to prod
git pull                       # Update prod
git merge main                 # Merge main into prod
git push                       # Push to origin (triggers deployment)
```

## 🔄 CI/CD & Deployment

### Automatic Deployment
Push to the `prod` branch triggers automatic deployment:
- **Front-end:** Deploys to Cloudflare Pages
- **Back-end:** Builds and deploys to Oracle VPS via SSH

### Manual Deployment

**Front-end:**
```bash
cd SourceCode/front-end/logsmart
bun run deploy
```

**Back-end:**
```bash
# Build locally
cd SourceCode/back-end
nix build .#aarch64-linux

# Deploy to server
ssh user@server
sudo systemctl stop logsmart
# Upload binary via SFTP
sudo systemctl start logsmart
```

### Required GitHub Secrets
**Front-end:**
- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`

**Back-end:**
- `DEPLOY_HOST` - Server IP/hostname
- `DEPLOY_USER` - SSH username
- `DEPLOY_SSH_KEY` - Private SSH key for authentication
- `DEPLOY_PORT` - SSH port (usually 22)
- `DEPLOY_PATH` - Target directory on server

### Monitoring Logs
View real-time logs from Cloudflare Pages:
```bash
cd SourceCode/front-end/logsmart
bun run logs
```

Or using Wrangler directly:
```bash
wrangler pages deployment tail --project-name=logsmart
```

## 📁 Project Structure
```
SourceCode/
├── front-end/
│   └── logsmart/          # SvelteKit application
│       ├── src/
│       │   ├── routes/    # SvelteKit routes
│       │   └── lib/       # Shared components and utilities
│       └── package.json
└── back-end/              # Rust API server
    ├── src/
    │   ├── main.rs        # Application entry point
    │   ├── handlers.rs    # API route handlers
    │   ├── auth.rs        # Authentication logic
    │   └── db.rs          # Database operations
    ├── Cargo.toml
    └── flake.nix          # Nix development environment
```

## 🐛 Troubleshooting

### Swagger UI blank on custom domain
If Swagger UI loads on `*.pages.dev` but not on your custom domain:
1. Go to Cloudflare Dashboard → Speed → Optimization
2. Disable **Rocket Loader**
3. Purge cache (Caching → Purge Everything)

### Build fails on GitHub Actions
- Check that all required secrets are set correctly
- Verify Nix flake builds locally: `nix build .#aarch64-linux`
- Review workflow logs in GitHub Actions tab