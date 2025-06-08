# DevContainer Environment Setup

## Required Environment Variables

Set these environment variables on your host system before starting the devcontainer:

### OpenAI Integration
```bash
export OPENAI_API_KEY="your-openai-api-key"
export OPENAI_ORG_ID="your-org-id"  # Optional
```

### Git Hosting Integration
Set tokens for any providers you intend to push to. The setup script will create
a `~/.netrc` file from these values.
```bash
export GITHUB_TOKEN="your-github-token"    # Optional
export GITLAB_TOKEN="your-gitlab-token"    # Optional
export BITBUCKET_TOKEN="your-bitbucket-token"  # Optional
```

## Setting Environment Variables

### Linux/macOS
Add to your `~/.bashrc`, `~/.zshrc`, or equivalent:
```bash
export OPENAI_API_KEY="sk-..."
export GITHUB_TOKEN="ghp_..."
export GITLAB_TOKEN="glpat-..."
export BITBUCKET_TOKEN="bbt_..."
```

### Windows
Using PowerShell:
```powershell
$env:OPENAI_API_KEY = "sk-..."
$env:GITHUB_TOKEN = "ghp_..."
$env:GITLAB_TOKEN = "glpat-..."
$env:BITBUCKET_TOKEN = "bbt_..."
```

Or set permanently via System Properties > Environment Variables.

## Persistent Cache Benefits

The devcontainer uses Docker volumes for caching:
- **Nix Store**: Packages persist between container rebuilds
- **Cargo Cache**: Rust dependencies cached across sessions
- **First run**: May take longer to populate caches
- **Subsequent runs**: Significantly faster due to cached packages

## Security Notes

- API keys are never stored in the container image
- Environment variables are only available during runtime
- Restart the container if you update environment variables
