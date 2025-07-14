# Terminal State Issue Fix for Moose Dev

## Problem Description

When interrupting the `moose dev` command with Ctrl+C, the terminal would not return to a clean state, leaving the cursor in a problematic state showing escape sequences like `^[OA`. This resulted in:

- Terminal cursor getting stuck or behaving erratically
- Terminal not accepting input properly
- Visual artifacts remaining on screen
- Arrow keys showing raw escape sequences (`^[OA`) instead of working properly
- History tools like atuin.sh not functioning correctly
- Need to manually reset terminal or restart shell session

**Special Note**: This issue is particularly problematic for users of terminal history management tools like atuin.sh, which rely on proper terminal cursor key mode settings.

## Root Cause Analysis - The Real Issue

The issue was caused by **interrupted terminal spinners** during the shutdown process. During `moose dev`, the `spinners` crate is used to display animated loading messages like:
- "Starting local infrastructure" 
- "Stopping all workflows"
- "Stopping containers"

**The Real Culprit**: When the process is interrupted with Ctrl+C, these spinners don't get a chance to properly clean up their terminal state. The spinners use methods like `sp.stop_with_newline()` to restore terminal state, but these cleanup methods are not called during signal interruption.

### What Spinners Do to Terminal State

The `spinners` crate manipulates terminal state to create animated spinners:
- May hide/show cursor during animation
- Clears and rewrites terminal lines
- Potentially modifies cursor key behavior
- When interrupted, leaves terminal in modified state

### Original Insufficient Fix

The original shutdown function only performed a basic terminal clear:

```rust
crossterm::execute!(
    std::io::stdout(),
    crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
)
```

This approach had several problems:

1. **No Spinner Cleanup**: Active spinners weren't being properly stopped during interruption
2. **Application Cursor Key Mode**: Terminal was left in a mode where arrow keys send raw escape sequences (`^[OA`) instead of being interpreted by atuin
3. **Incomplete Terminal Reset**: Only clearing until the end of line was insufficient
4. **Missing Raw Mode Handling**: If any processes had enabled raw mode, it wasn't being properly disabled
5. **Missing Alternate Screen Mode Handling**: Terminal might be left in alternate screen mode
6. **No Cursor State Management**: Cursor visibility and positioning weren't being properly restored

## Solution Implemented

### 1. Enhanced Terminal Restoration Function

Created a comprehensive `restore_terminal_state()` function that:

- **Cleans up spinner state first** using `force_cleanup_spinners()`
- Disables raw mode if it was enabled
- Exits application cursor key mode (critical for arrow keys)
- Clears the entire screen and resets cursor position
- Shows the cursor if it was hidden
- Disables mouse reporting and special modes
- Includes proper timing to allow terminal to process all commands

### 2. Spinner-Specific Cleanup

Added `force_cleanup_spinners()` function that:

```rust
pub fn force_cleanup_spinners() {
    // Force a newline to ensure we're on a clean line
    print!("\n");
    
    // Send escape sequences to ensure any spinner-related terminal state is cleared
    print!("\x1b[?25h");    // Show cursor (in case spinner hid it)
    print!("\x1b[K");       // Clear current line (in case spinner left artifacts)
    print!("\x1b[0m");      // Reset terminal attributes
    
    let _ = std::io::Write::flush(&mut std::io::stdout());
}
```

### 3. Targeted Escape Sequence Reset

The new function uses a simplified, targeted approach to reset the most critical terminal states:

```rust
print!("\x1b[?1l");     // Exit application cursor key mode (CRITICAL for arrow keys)
print!("\x1b[?1049l");  // Exit alternate screen mode
print!("\x1b[?25h");    // Show cursor
print!("\x1b[0m");      // Reset all attributes
print!("\x1b[2J");      // Clear screen
print!("\x1b[H");       // Move cursor to home
print!("\x1b[?1000l");  // Disable mouse reporting
print!("\x1b[?1002l");  // Disable button event mouse reporting
print!("\x1b[?1006l");  // Disable SGR extended mouse reporting
print!("\x1b[?2004l");  // Disable bracketed paste mode
print!("\x1b>");        // Exit alternate keypad mode
print!("\x1b[?1l");     // Exit application cursor key mode (repeated for emphasis)
```

**Key improvement**: The function prioritizes spinner cleanup and exiting application cursor key mode (`\x1b[?1l`), which is essential for tools like atuin.sh to properly interpret arrow key sequences.

### 4. Global Panic Handler

Added a panic handler that ensures terminal state is restored even if there's an unexpected panic:

```rust
let original_panic_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic_info| {
    // Restore terminal state before panicking
    restore_terminal_state();
    // Call the original panic hook
    original_panic_hook(panic_info);
}));
```

### 5. Integration with Existing Shutdown Process

The new terminal restoration function is called during the normal shutdown process, ensuring clean terminal state regardless of how the application terminates.

## Files Modified

1. **apps/framework-cli/src/cli/local_webserver.rs**
   - Added `restore_terminal_state()` function
   - Modified `shutdown()` function to call terminal restoration
   - Added panic handler for terminal restoration

2. **apps/framework-cli/src/cli/display.rs**
   - Added `force_cleanup_spinners()` function
   - Enhanced spinner cleanup handling

## Testing Instructions

### Manual Testing

1. **Start a moose development server**:
   ```bash
   cd /path/to/moose/project
   moose dev
   ```

2. **Wait for the server to fully start** (you should see the success messages and "Next Steps" prompt)

3. **Interrupt with Ctrl+C**

4. **Verify terminal state**:
   - Terminal should return to normal prompt immediately
   - Cursor should be visible and responsive
   - No escape sequences should be visible
   - Terminal should accept input normally

### Atuin-Specific Testing

For users with atuin.sh installed:

1. **Arrow key functionality**: After interrupting `moose dev`, test that:
   - Up arrow shows command history (not `^[OA`)
   - Down arrow navigates history (not `^[OB`)
   - Left/right arrows move cursor in command line (not `^[OD`/`^[OC`)

2. **History search**: Test that Ctrl+R still works for atuin's fuzzy history search

3. **Shell integration**: Verify that atuin's shell integration remains functional after interruption

### Edge Case Testing

Test the following scenarios:

1. **Quick interruption**: Ctrl+C immediately after starting (during "Starting local infrastructure" spinner)
2. **During spinners**: Ctrl+C while various spinners are active
3. **During workflow shutdown**: Ctrl+C while workflows are being terminated
4. **Multiple rapid interruptions**: Multiple Ctrl+C presses in quick succession

## Troubleshooting

If arrow keys still show escape sequences after the fix:

1. **Manual terminal reset**: Run `reset` command to fully reinitialize terminal
2. **Restart shell**: Exit and restart your shell session
3. **Atuin reinitialization**: Run `source ~/.bashrc` or `source ~/.zshrc` to reinitialize atuin
4. **Check TERM variable**: Ensure `echo $TERM` shows a valid terminal type (e.g., `xterm-256color`)

## Benefits

- **Improved User Experience**: Terminal remains usable after interrupting moose dev
- **Better Atuin Compatibility**: Specifically addresses arrow key issues with atuin.sh
- **Reduced Support Issues**: Users won't need to restart their terminal sessions
- **Better Development Workflow**: Developers can quickly restart moose dev without terminal issues
- **Robust Error Handling**: Terminal state is restored even during unexpected errors

## Future Considerations

- Monitor for any remaining edge cases in different terminal emulators
- Consider adding signal handlers to spinner functions for cleaner interruption
- Evaluate if similar fixes are needed for other moose commands that use spinners
- Add automated testing for terminal state management

This fix ensures that moose dev provides a professional, robust development experience with proper terminal state management, especially for users of advanced terminal tools like atuin.sh.