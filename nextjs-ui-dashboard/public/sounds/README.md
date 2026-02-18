# Notification Sounds

This directory contains audio files used for notification sound effects.

## Required Files

The following audio files should be placed in this directory:

1. **notification.mp3** - Default notification sound
   - Used for general notifications
   - Recommended: Short, pleasant chime sound
   - Duration: 0.5-1 second

2. **trade-executed.mp3** (optional) - Trade execution sound
   - Used when a trade is executed
   - Recommended: Success/positive sound
   - Duration: 0.5-1 second

3. **alert.mp3** (optional) - Alert sound
   - Used for important alerts and warnings
   - Recommended: Attention-grabbing sound
   - Duration: 0.5-1 second

## Sound Sources

You can obtain free notification sounds from:
- https://freesound.org/
- https://mixkit.co/free-sound-effects/
- https://www.zapsplat.com/

## Format Requirements

- Format: MP3 or WAV
- Sample Rate: 44.1kHz
- Bit Depth: 16-bit
- Channels: Stereo or Mono
- File Size: < 100KB recommended

## Usage

The notification system will automatically play sounds based on user preferences.
If a sound file is missing, the system will silently skip playing sounds.
