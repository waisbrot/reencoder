"""
Gunicorn configuration
"""

bind = '0.0.0.0:8080'

worker = 2
worker_class = 'sync'
timeout = 30

daemon = False

logfile = '-'
loglevel = 'info'
