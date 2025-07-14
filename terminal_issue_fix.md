# Terminal State Issue Fix for Moose Dev

## Problem Description

When interrupting the `moose dev` command with Ctrl+C, the terminal would not return to a clean state, leaving the cursor in a problematic state showing escape sequences like `^[OA`. This resulted in:

- Terminal cursor getting stuck or behaving erratically
- Terminal not accepting input properly
- Visual artifacts remaining on screen
- Need to manually reset terminal or restart shell session

## Root Cause Analysis

The issue was caused by insufficient terminal state restoration during the shutdown process. The original shutdown function only performed a basic terminal clear:

```rust
crossterm::execute!(
    std::io::stdout(),
    crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
)
```

This approach had several problems:

1. **Incomplete Terminal Reset**: Only clearing until the end of line was insufficient to restore the terminal to its normal state
2. **Missing Raw Mode Handling**: If any processes had enabled raw mode, it wasn't being properly disabled
3. **Missing Alternate Screen Mode Handling**: Terminal might be left in alternate screen mode
4. **No Cursor State Management**: Cursor visibility and positioning weren't being properly restored
5. **No Mouse Reporting Cleanup**: Mouse event reporting modes weren't being disabled
6. **Insufficient Escape Sequence Handling**: Not all terminal modes were being properly reset

## Solution Implemented

### 1. Enhanced Terminal Restoration Function

Created a comprehensive `restore_terminal_state()` function that:

- Disables raw mode if it was enabled
- Clears the entire screen instead of just the current line
- Moves cursor to home position (0,0)
- Shows the cursor if it was hidden
- Sends comprehensive escape sequences to reset terminal state
- Includes proper timing to allow terminal to process all commands

### 2. Comprehensive Escape Sequence Reset

The new function sends multiple escape sequences to handle various terminal states:

```rust
print!("\x1b[?1049l");  // Exit alternate screen mode (if active)
print!("\x1b[?25h");    // Show cursor
print!("\x1b[0m");      // Reset all attributes (colors, styles, etc.)
print!("\x1b[2J");      // Clear entire screen
print!("\x1b[H");       // Move cursor to home position
print!("\x1b[?12l");    // Disable cursor blinking (if enabled)
print!("\x1b[?1000l");  // Disable mouse reporting (if enabled)
print!("\x1b[?1002l");  // Disable button event mouse reporting (if enabled)
print!("\x1b[?1006l");  // Disable SGR extended mouse reporting (if enabled)
print!("\x1b[!p");      // Soft reset (RIS - Reset to Initial State)
print!("\x1b[?47l");    // Exit alternate screen buffer (alternative to ?1049l)
```

### 3. Global Panic Handler

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

### 4. Integration with Existing Shutdown Process

The new terminal restoration function is called during the normal shutdown process, ensuring clean terminal state regardless of how the application terminates.

## Files Modified

1. **apps/framework-cli/src/cli/local_webserver.rs**
   - Added `restore_terminal_state()` function
   - Modified `shutdown()` function to call terminal restoration
   - Added panic handler for terminal restoration

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

### Automated Testing

The fix can be tested programmatically by:

1. Creating a test moose project
2. Starting `moose dev` in a subprocess
3. Sending SIGINT signal after startup
4. Verifying terminal state restoration

### Edge Case Testing

Test the following scenarios:

1. **Quick interruption**: Ctrl+C immediately after starting
2. **During spinners**: Ctrl+C while "Starting local infrastructure" spinner is active
3. **During workflow shutdown**: Ctrl+C while workflows are being terminated
4. **Multiple rapid interruptions**: Multiple Ctrl+C presses in quick succession

## Additional Improvements

The solution also includes:

1. **Timing Safety**: Added a small delay to ensure all escape sequences are processed
2. **Error Handling**: All terminal operations use `let _ = ` to prevent panic on failure
3. **Comprehensive Coverage**: Handles multiple terminal modes and states
4. **Backward Compatibility**: Maintains existing functionality while adding robust cleanup

## Benefits

- **Improved User Experience**: Terminal remains usable after interrupting moose dev
- **Reduced Support Issues**: Users won't need to restart their terminal sessions
- **Better Development Workflow**: Developers can quickly restart moose dev without terminal issues
- **Robust Error Handling**: Terminal state is restored even during unexpected errors

## Future Considerations

- Monitor for any remaining edge cases in different terminal emulators
- Consider adding terminal type detection for more specific handling
- Evaluate if similar fixes are needed for other moose commands
- Add automated testing for terminal state management

This fix ensures that moose dev provides a professional, robust development experience with proper terminal state management.