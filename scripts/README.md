# SYN Recruitment Video Scripts

This directory contains the scripts and assets for the SYN recruitment video command (`nexus-network syn-recruit`).

## Files

### `scripts/syn/all-your-node.scenes.json`
The main dialogue script. Each scene has:
- `speaker`: Character hex tag (0xACCC = ACK villain, 0xCABB = SYN hero, etc.)
- `line`: The dialogue text
- `delay_ms`: Pause duration in milliseconds

### `scripts/syn/activity.log.json`
Activity log entries that appear during the video. Each entry has:
- `level`: Log level (INFO, WARN, ALERT, CRIT, ERROR, OK)
- `msg`: The log message

### `assets/ascii/outro-syn.txt`
ASCII art for the SYN logo outro.

### `assets/audio/`
Generated audio files:
- `syn_bg_music.wav` - Background music
- `console_beep.wav` - Beep for each message
- `alert.wav` - Alert sound
- `rocket.wav` - Rocket launch sound
- `victory.wav` - Victory fanfare

## Customization

### Adding New Characters
1. Add new hex tags to the script
2. Update `speaker_color()` function in `clients/cli/src/syn.rs`
3. Add corresponding sound effects

### Changing Dialogue
Edit `scripts/syn/all-your-node.scenes.json` to modify the script.

### Custom Sound Effects
Modify `clients/cli/src/audio.rs` to generate different sounds.

### Different ASCII Art
Replace `assets/ascii/outro-syn.txt` with your own ASCII art.

## Creating Your Own Version

1. Fork the repository
2. Modify the scripts and assets
3. Test with `nexus-network syn-recruit`
4. Record your video
5. Submit a PR with your creative interpretation!

## Character Guide

- **0xDEAD**: Narrator/system (gray)
- **0xCABB**: SYN team leader/hero (yellow)
- **0xF1X3**: SYN team member (green)
- **0xD00D**: SYN team member (cyan)
- **0xACCC**: ACK villain (magenta)
