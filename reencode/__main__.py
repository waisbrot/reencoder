import multiprocessing
import argparse
import os.path
import sys
from fnmatch import fnmatch
from . import reencode
import logging
import logging.config
import logging.handlers

log = None


def configure_logging(args):
    if args.verbose == 0:
        level = 'WARNING'
    elif args.verbose == 1:
        level = 'INFO'
    elif args.verbose >= 2:
        level = 'DEBUG'

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
    parser.add_argument('--server', action='store_true', required=False, help="Run the server (as opposed to the client)")
    parser.add_argument('--wait', action='store_true', required=False, help="As the client, wait for the job to complete?")
    parser.add_argument('--threads', type=int, required=False, default=1, help="Number of worker-threads")
    parser.add_argument('--width', type=int, required=False, default=1280, help="Max width of the video (height will be calculated automatically)")
    parser.add_argument('--bitrate', type=int, required=False, default=2000000, help="Video bit-rate")
    parser.add_argument('--verbose', '-v', required=False, default=0, action='count', help="Log level (start at warning)")
    parser.add_argument('--file', type=str, required=False, default=None, help="File to re-encode")
    parser.add_argument('--ignored-patterns', type=split_list, required=False,
                        default=['*.nfo', '*.sub', '*.idx', '*.txt', '.*', '*.url', '*.jpg', '*.zip', '*.sfv', '*.srr', '*.nzb', '*.diz'],
                        help='Comma-separated list of Unix file-glob patterns to ignore')
    parser.add_argument('--port', type=int, required=False, default=3260, help="Port to listen/connect")
    parser.add_argument('--host', type=str, required=False, default='localhost', help="Host to listent/connect")
    parser.add_argument('--debug', type=str, required=False, default='False', help="Set to 'True' for debug-mode")
    parser.add_argument('--debug-delay', type=int, required=False, default=500, help="Time in ms to delay when in debug-mode")
    return parser.parse_args()


def check_args(args):
    if args.server:
        if args.file:
            log.error("Can't supply the --file arg to a server")
            sys.exit(1)
        if args.wait:
            log.error("Can't supply the --wait arg to a server")
            sys.exit(1)


def filter_ignored(files, patterns):
    for f in files:
        ok = True
        for p in patterns:
            if fnmatch(f.lower(), p.lower()):
                ok = False
                break
        if ok:
            yield f


print("Hello world")

# args = parse_args()
# check_args(args)
# configure_logging(args)
# log.info("Inital arguments: {}".format(args))
# status = 0
# if args.server:
#     status = reencode.server(args)
# else:
#     message_dict = {k: v for k, v in vars(args).items() if k in ['width', 'bitrate', 'file', 'debug', 'debug_delay']}
#     status = reencode.client(args.host, args.port, message_dict, args.wait)
# sys.exit(status)
