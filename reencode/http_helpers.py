'''
Helper functions for working with an HTTP call inside Gunicorn
'''
import json


def error_not_found(start_response, message='Not Found'):
    response_body = bytes(message, 'UTF-8')
    status = '404 Not Found'
    response_headers = [
        ('Content-Type', 'text/plain'),
    ]
    start_response(status, response_headers)
    return [response_body]


def error_bad_request(start_response, message='Bad Request'):
    response_body = bytes(message, 'UTF-8')
    status = '400 Bad Request'
    response_headers = [
        ('Content-Type', 'text/plain'),
    ]
    start_response(status, response_headers)
    return [response_body]


def json_ok(start_response, data):
    response_body = bytes(json.dumps(data, ensure_ascii=False), 'UTF-8')
    status = '200 OK'
    response_headers = [
        ('Content-Type', 'application/json'),
    ]
    start_response(status, response_headers)
    return [response_body]


def parse_query(query_string):
    if len(query_string) == 0:
        return {}
    q = {}
    for kv in query_string.split('&'):
        [k, v] = kv.split('=')
        q[k] = v
    return q
