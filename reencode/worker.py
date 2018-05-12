'''
A worker-function to pull things from the queue definied by the 'database' module and
re-encode using the 'reencode' module.
Concurrency is safe here, because all shared state is kept in the 'database'.
'''

import time
import urllib.request
import urllib.parse
import urllib.error
import json
from reencode.reencode import scan_file, reencode, cleanup, IgnoreFileException
from subprocess import CalledProcessError
import logging


log = None


video_bitrate = 2000000  # used for detecting files that need no processing
width = 1280
database_url = 'http://127.0.0.1:8081'


def run(index):
    global log
    log = logging.getLogger('worker {}'.format(index))
    log.info("Starting worker")
    time.sleep(5)
    while True:
        task = fetch_from_queue()
        log.info("Got task off queue: %s", task)
        job_id = task['id']
        try:
            scan = scan_file(task['file'], video_bitrate, width)
            update_status(job_id, 'reencoding')
            reencode(scan)
            update_status(job_id, 'cleaning up')
            cleanup_result = cleanup(scan)
            update_status(job_id, 'Done - reduced from {}MiB to {}MiB'.format(cleanup_result['original_size_mb'], cleanup_result['new_size_mb']))
        except IgnoreFileException:
            update_status(job_id, 'Done - already processed')
        except CalledProcessError as e:
            log.error("CalledProcessError: %s \n-> %s", e.cmd, e.output)
            update_status(job_id, 'Error - error from subprocess (maybe we couldn\'t read the encoding?)')
        except Exception:
            log.exception("Error attempting to process file")
            update_status(job_id, 'Error - unknown error processing file')


def fetch_from_queue():
    while True:
        try:
            with urllib.request.urlopen('{}/queue/pop'.format(database_url)) as resp:
                return json.loads(resp.read().decode('UTF-8'))
        except urllib.error.HTTPError:
            log.debug("Error fetching from queue. Sleep and retry")
            time.sleep(10)


def update_status(job_id, status):
    req = urllib.request.Request(url='{}/status?job={}'.format(database_url, job_id), data=bytes(status, 'UTF-8'), method='POST')
    log.info("update status of job %s: %s", job_id, status)
    with urllib.request.urlopen(req):
        pass
