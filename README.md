# Re-Encoder
A little script for re-encoding video files with ffmpeg, since the options are a pain to remember and its output is gross.

# Usage
find /path/to/video -type f -exec reencode.py --file '{}' \;
