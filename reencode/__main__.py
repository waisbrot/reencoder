'''
Entry-point. Starts a Gunicorn-based REST app to handle the "database"/queue and a separate worker-process to operate
'''

import argparse
import reencode.database
import reencode.worker
import logging
import logging.config
import logging.handlers
from multiprocessing import Process
import gunicorn.app.wsgiapp


log = None


def configure_logging(args):
    '''
    Given some command-line args, configure logging.
    Mostly, the dynamic config is to allow setting verbosity by command-line switch
    '''
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
    '''
    Handle argparse work
    '''
    def split_list(l):
        l.split(',')
    parser = argparse.ArgumentParser(description="Re-encode video files")
    parser.add_argument('--verbose', '-v', required=False, default=0, action='count', help="Log level (start at warning)")
    return parser.parse_args()


class StandaloneApplication(gunicorn.app.base.BaseApplication):
    '''
    Skeletal Gunicorn application
    '''
    def __init__(self, app, options=None):
        self.options = options or {}
        self.application = app
        super(StandaloneApplication, self).__init__()

    def load_config(self):
        config = {key: value for key, value in self.options.items()
                  if key in self.cfg.settings and value is not None}
        for key, value in config.items():
            self.cfg.set(key.lower(), value)

    def load(self):
        return self.application


args = parse_args()
configure_logging(args)
processes = []

db_options = {
    'bind': '0.0.0.0:8081',
    'workers': 1,
    'worker_class': 'sync',
}
db_app = StandaloneApplication(reencode.database.handler, db_options)
db_process = Process(target=db_app.run, name='database', args=[])
processes.append(db_process)

for wid in range(2):
    worker = Process(target=reencode.worker.run, name='worker{}'.format(wid), args=[wid], daemon=True)
    processes.append(worker)

for proc in processes:
    proc.start()

for proc in processes:
    try:
        proc.join()
    except KeyboardInterrupt:
        pass  # silence stack traces
