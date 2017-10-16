import multiprocessing
import argparse
import os.path
import os
import fnmatch
from . import progress
from . import reencode


queue = multiprocessing.Queue()


def parse_args():
    def split_list(l):
        l.split(',')
    parser = argparse.ArgumentParser(description="Re-encode video files")
    parser.add_argument('--width', type=int, required=False, default=1280, help="Max width of the video (height will be calculated automatically)")
    parser.add_argument('--encoding', type=str, required=False, default='h265', choices=['h264', 'h265'], help="Video encoding. Either h264 or h265")
    parser.add_argument('--bitrate', type=int, required=False, default=2000000, help="Video bit-rate")
    parser.add_argument('--file', type=str, required=True, help="File to re-encode")
    parser.add_argument('--ignored-patterns', type=split_list, required=False, default=['*.nfo', '*.sub', '*.idx', '*.txt', '.*'], help='Comma-separated list of Unix file-glob patterns to ignore')
    return parser.parse_args()


try:
    args = parse_args()
    progress.start_pbar()
    if os.path.isdir(args.file):
        for root, dirs, files in os.walk(args.file):
            for f in fnmatch.filter(files, args.ignored_patterns):
                reencode.process(os.path.join(root, f), args.bitrate, args.width, queue)
            dirs = fnmatch.filter(dirs, args.ignored_patterns)
    else:
        reencode.process(args.file, args.bitrate, args.width, queue)
finally:
    pass
