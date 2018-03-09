import multiprocessing
import argparse
import os.path
import os
from fnmatch import fnmatch
from . import progress
from . import reencode
import logging
import logging.config
import logging.handlers

log = None
def configure_logging(args):
    if args.verbose == 1:
        level = 'INFO'
    elif args.verbose >= 2:
        level = 'DEBUG'

    if args.live_progress:
        handler = 'file'
    else:
        handler = 'stream'

    logging.config.dictConfig({
        'version': 1,
        'formatters': {
            'file_formatter': {
                'format': '%(asctime)s %(levelname)-8s %(name)-15s %(message)s',
                'datefmt': '%Y-%m-%dT%H:%M:%S',
            }
        },
        'filters': {},
        'handlers': {
            'file': {
                'class': 'logging.handlers.RotatingFileHandler',
                'level': 'DEBUG',
                'formatter': 'file_formatter',
                'filename': 'reencode.debug.log',
                'maxBytes': 1024*1024,
                'backupCount': 1,
                'mode': 'w',
            },
            'stream': {
                'class': 'logging.StreamHandler',
                'level': 'DEBUG',
                'formatter': 'file_formatter',
            },
        },
        'loggers': {},
        'root': {
            'level': level,
            'handlers': [handler],
        },
        'incremental': False,
        'disable_existing_loggers': False,
    })
    global log
    log = logging.getLogger('main')


def parse_args():
    def split_list(l):
        l.split(',')
    parser = argparse.ArgumentParser(description="Re-encode video files")
    parser.add_argument('--width', type=int, required=False, default=1280, help="Max width of the video (height will be calculated automatically)")
    parser.add_argument('--encoding', type=str, required=False, default='h265', choices=['h264', 'h265'], help="Video encoding. Either h264 or h265")
    parser.add_argument('--bitrate', type=int, required=False, default=2000000, help="Video bit-rate")
    parser.add_argument('--live-progress', required=False, action='store_false', help="Avoid showing the live progress bar and instead just print a line?")
    parser.add_argument('--verbose', '-v', required=False, action='count', help="Log level (start at warning)")
    parser.add_argument('--file', type=str, required=True, help="File to re-encode")
    parser.add_argument('--ignored-patterns', type=split_list, required=False,
                        default=['*.nfo', '*.sub', '*.idx', '*.txt', '.*', '*.url', '*.jpg', '*.zip', '*.sfv', '*.srr'],
                        help='Comma-separated list of Unix file-glob patterns to ignore')
    return parser.parse_args()


def filter_ignored(files, patterns):
    for f in files:
        ok = True
        for p in patterns:
            if fnmatch(f.lower(), p.lower()):
                ok = False
                break
        if ok:
            yield f


try:
    args = parse_args()
    configure_logging(args)
    log.info("Inital arguments: {}".format(args))
    queue = multiprocessing.Queue()
    progress.start_pbar(queue, args.live_progress)
    if os.path.isdir(args.file):
        for root, dirs, files in os.walk(args.file):
            for f in filter_ignored(files, args.ignored_patterns):
                reencode.process(os.path.join(root, f), args.bitrate, args.width, queue)
                queue.put('next')
            dirs = list(filter_ignored(dirs, args.ignored_patterns))
    else:
        reencode.process(args.file, args.bitrate, args.width, queue)
finally:
    pass
