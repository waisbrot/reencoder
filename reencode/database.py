'''
WSGI handler to manage a work queue.
Assumes that it's a single thread with no concurrency allowed.

Routes:
- GET /status?job=<job_id>  -- show a job's status. Returns a JSON object. Important keys:
  - "status" - text string describing the status
  - "done" - true if the job is complete (failed or succeeded)
  - "success" - true if the job completed successfully
- POST /status?job=<job_id> -- expects a status string. Used by workers to update a job's status
- POST /queue/push -- expects JSON object input with a key "file" containing the full path to a file to process. Returns the job record.
- GET /queue/pop -- used by the workers to retrieve the next job
- POST /gc[?delta=<seconds>]  -- remove finished jobs from the status-map. Defaults to jobs older than 30 days, but "delta" can be set to some lower number
'''

from reencode.http_helpers import error_not_found, error_bad_request, json_ok, parse_query
import json
import time
import hashlib
import logging


log = logging.getLogger(__name__)
work_queue = []
job_status = {}


def handler(environ, start_response):
    if environ['PATH_INFO'] == '/status':
        return handle_status(environ, start_response)
    elif environ['PATH_INFO'] == '/queue/push':
        return handle_queue_push(environ, start_response)
    elif environ['PATH_INFO'] == '/queue/pop':
        return handle_queue_pop(environ, start_response)
    elif environ['PATH_INFO'] == '/gc':
        return handle_gc(environ, start_response)
    else:
        return error_not_found(start_response, 'Bad path')


def handle_status(environ, start_response):
    query = parse_query(environ['QUERY_STRING'])
    if 'job' not in query:
        return error_bad_request(start_response, 'Must send a "job" query')
    job_id = query['job']
    if job_id not in job_status:
        return error_not_found(start_response, 'Job ID not found')
    elif environ['REQUEST_METHOD'] == 'GET':
        return handle_status_query(environ, start_response, job_id)
    elif environ['REQUEST_METHOD'] == 'POST':
        return handle_status_change(environ, start_response, job_id)


def handle_status_query(environ, start_response, job_id):
    return json_ok(start_response, job_status[job_id])


def handle_status_change(environ, start_response, job_id):
    status = environ['wsgi.input'].read().decode('UTF-8')
    log.debug("status change to %s", status)
    job_status[job_id]['status'] = status
    if status.startswith('Done'):
        job_status[job_id]['done'] = True
        job_status[job_id]['success'] = True
    elif status.startswith('Error'):
        job_status[job_id]['done'] = True
        job_status[job_id]['success'] = False
    return handle_status_query(environ, start_response, job_id)


def handle_queue_push(environ, start_response):
    try:
        job = json.loads(environ['wsgi.input'].read().decode('UTF-8'))
        job['status'] = 'queued'
        job['done'] = False
        job['success'] = None
        job['submit_time'] = int(time.time())
        job_id = hashlib.md5(bytes(job['file'], 'UTF-8')).hexdigest()
        job['id'] = job_id
        work_queue.append(job)
        job_status[job_id] = job
        return json_ok(start_response, job)
    except ValueError:
        return error_bad_request(start_response, 'Must send JSON data')
    except KeyError:
        return error_bad_request(start_response, 'Must include the "file" key in request')


def handle_queue_pop(environ, start_response):
    try:
        job = work_queue.pop(0)
        job['status'] = 'sent to worker'
        job_status[job['id']] = job
        return json_ok(start_response, job)
    except IndexError:
        return error_not_found(start_response, 'Queue empty')


def handle_gc(environ, start_response):
    delta = 60 * 60 * 24 * 30
    query = parse_query(environ['QUERY_STRING'])
    if 'delta' in query:
        delta = int(query['delta'])
    garbage = []
    cutoff = int(time.time()) - delta
    for job_id, job in job_status.items():
        if job['submit_time'] < cutoff and job['done'] and job_id not in work_queue:
            garbage.append(job_id)
    for job_id in garbage:
        del job_status[job_id]
    return json_ok(start_response, garbage)
