# Smart Git Commit Command

This command helps you create well-organized commits by analyzing currently modified files and grouping them into logical, atomic commits with clear descriptions.

## Instructions

1. **Check current git status** to see all modified files
   ```bash
   git status
   ```

2. **Analyze the changes** in each file to understand what was modified
   ```bash
   git diff <file>
   ```

3. **Group files by logical changes**
   - Group files that work together to implement a single feature/fix
   - Separate infrastructure changes from feature implementation
   - Keep refactoring separate from functional changes
   - Group test files with the code they test

4. **Create atomic commits** for each logical group:
   - Stage only the files for the current logical change
   - Write descriptive commit messages following this format:

   ```
   <type>: <short description>

   <detailed explanation of what and why>

   Changes:
   - Specific change 1
   - Specific change 2
   - ...

   <any additional context or notes>
   ```

5. **Commit types to use**:
   - `feat`: New feature or functionality
   - `fix`: Bug fix
   - `refactor`: Code restructuring without changing functionality
   - `docs`: Documentation changes
   - `test`: Adding or modifying tests
   - `chore`: Maintenance tasks, dependency updates
   - `style`: Code formatting, missing semicolons, etc.

## Example Workflow

```bash
# 1. Check status
git status

# 2. Review changes
git diff src/system/settings/settings_manager.rs
git diff src/system/runtime.rs

# 3. Group 1: Settings infrastructure
git add src/system/settings/settings_manager.rs src/system/settings/system_config.rs
git commit -m "feat: add SettingsManager with thread-safe access

Implement centralized settings management with separate RwLocks per category
for optimal concurrency. Supports JSON persistence and async access patterns.

Changes:
- Add SettingsManager with Application, Automatic1111, and ComfyUI categories
- Implement thread-safe getters/setters with RwLock protection
- Add JSON serialization/deserialization with settings.json
- Include convenience methods for common settings access"

# 4. Group 2: Runtime integration
git add src/system/runtime.rs
git commit -m "feat: integrate SettingsManager into Runtime

Add SettingsManager as Arc-wrapped field in Runtime struct following the
established manager pattern used by PipeManager.

Changes:
- Add settings_manager field to Runtime struct
- Initialize SettingsManager in Runtime::new()
- Add settings_manager() accessor method for external access"

# 5. Group 3: Migration of existing code
git add src/system/mailboxes/internal/handlers/settings.rs src/system/runtime_tasks.rs
git commit -m "refactor: migrate hardcoded delays to SettingsManager

Replace hardcoded timing constants with configurable values from SettingsManager
to allow users to tune the application for their environment.

Changes:
- Migrate generation debounce (1500ms) to user settings
- Migrate state save delay (3000ms) to user settings
- Migrate keepalive interval (500ms) to user settings
- Update macros to fetch delays from settings automatically"
```

## Best Practices

1. **Keep commits atomic** - Each commit should represent one logical change
2. **Write in imperative mood** - "Add feature" not "Added feature"
3. **Explain why, not just what** - Context is valuable for future developers
4. **Reference issues** - Include issue numbers if applicable (#123)
5. **Don't commit commented code** - Remove or explain why it's kept
6. **Separate formatting changes** - Don't mix formatting with functional changes

## Common Groupings

- **Infrastructure + Usage**: When adding a new system, commit infrastructure first, then migrations
- **API + Implementation**: Commit interface definitions before implementations
- **Feature + Tests**: Commit feature and its tests together
- **Refactor by scope**: Group refactoring by module or functionality
- **Config changes**: Group all configuration-related changes together

## Validation Checklist

Before committing, ensure:
- [ ] Changes are grouped logically
- [ ] Commit message clearly explains the change
- [ ] File permissions haven't changed unnecessarily
- [ ] No accidental file additions (check .gitignore)

Remember: Good commits tell a story. Someone should be able to understand the evolution of your codebase by reading commit messages alone.