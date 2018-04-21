import os.path as path
import time
import tempfile
import json
import subprocess
import shutil
import os
from collections import namedtuple
import logging
#import socketserver
import multiprocessing
import copy
import socket


log = logging.getLogger('reencoder')


ScanResult = namedtuple('ScanResult',
                        ['source', 'source_dir', 'source_basename', 'scale_arg', 'audio_bitrate', 'temp_out', 'temp_dir'])


class IgnoreFileException(Exception):
    pass


class BadClientRequest(Exception):
    pass


MAX_REQ_SIZE = 2048


counter_lock = multiprocessing.Lock()
work_counter = 0
job_id = 0
finished_job_floor = -1
finished_jobs = set()


def job_finished_callback(result):
    global work_counter, finished_job_floor, finished_jobs
    with counter_lock:
        job_id = result['job_id']
        if job_id == finished_job_floor + 1:
            finished_job_floor = job_id
            job_id = job_id + 1
            while job_id in finished_jobs:
                finished_jobs.remove(job_id)
                finished_job_floor = job_id
                job_id = job_id + 1
        else:
            finished_jobs.add(job_id)
        work_counter = work_counter - 1


def job_error_callback(error):
    log.error("Error from job. Now we don't know what state we're in :-(")


def is_job_finished(job_id):
    with counter_lock:
        return (job_id <= finished_job_floor) or (job_id in finished_jobs)


def server(args):
    pool = multiprocessing.Pool(args.threads)
    basic_request = {
        'bitrate': args.bitrate,
        'width': args.width,
        'debug': 'False',
        'debug_delay': 500,
    }

    class RequestHandler(socketserver.BaseRequestHandler):
        def handle(self):
            global work_counter, job_id
            req_str = str(self.request.recv(MAX_REQ_SIZE), 'utf-8').strip()
            log.debug("Got req {}".format(req_str))
            if req_str == 'TERMINATE':
                self.request.sendall(bytes("Terminating", 'utf-8'))

                def shutdown_server(server):
                    server.shutdown()
                multiprocessing.Process(target=shutdown_server, args=(self.server,)).start()
                return
            if req_str == 'COUNTER':
                with counter_lock:
                    self.request.sendall(bytes(json.dumps({'queue_size': work_counter, 'job_count': job_id}), 'utf-8'))
                return
            req_dict = json.loads(req_str)
            if 'file' not in req_dict:
                self.request.sendall(bytes("Bad Request", 'utf-8'))
                raise BadClientRequest(req_str)
            with counter_lock:
                work_counter = work_counter + 1
                job_id = job_id + 1
            full_req = copy.copy(basic_request)
            full_req.update(req_dict)
            full_req['job_id'] = job_id
            pool.apply_async(process, [], full_req, job_finished_callback, job_error_callback)
            self.request.sendall(bytes(json.dumps({'job_id': job_id}), 'utf-8'))

    server = socketserver.TCPServer((args.host, args.port), RequestHandler)
    log.info("Server listening, {} workers".format(args.threads))
    server.serve_forever(poll_interval=1)
    log.info("Server terminating")
    pool.terminate()
    pool.join()
    log.debug("Pool terminated")
    return 0


def client(host, port, message_dict, wait=False):
    message = json.dumps(message_dict)
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect((host, port))
        sock.sendall(bytes(message, 'utf-8'))
        response = str(sock.recv(1024), 'utf-8')
        print(response)
        if response.startswith('{'):
            response = json.loads(response)
            if wait:
                while not is_job_finished(response['job_id']):
                    time.sleep(60.0)
                    print('.')
            return 0
        elif response == 'Terminating':
            return 0
        else:
            return 1


def process(file=None, bitrate=None, width=None, debug='False', debug_delay=500, job_id=None):
    file_ = file
    try:
        if file_ is None or bitrate is None or width is None or job_id is None:
            log.warn("Request was missing a required value")
            return
        if debug == 'True':
            log.debug("Worker got request for file {}. Sleeping {}ms".format(file_, debug_delay))
            time.sleep(debug_delay / 1000.0)
            log.debug("Worker finished")
            return
        scan_result = scan_file(file_, bitrate, width)
        reencode(scan_result)
        return cleanup(scan_result, job_id)
    except IgnoreFileException:
        pass
    except subprocess.CalledProcessError:
        log.exception('CalledProcessError')
    except Exception:
        log.exception('Error processing file')


def scan_file(file_, bitrate, width):
    source = file_
    (_, ext) = path.splitext(source)

    (source_basename, _) = path.splitext(path.basename(source))
    source_dir = path.dirname(source)
    temp_dir = tempfile.mkdtemp()
    temp_out = '{}/{}.mp4'.format(temp_dir, source_basename)
    source_info = json.loads(subprocess.check_output(['exiftool', '-j', source]).decode('utf-8'))[0]
    video_bitrate = bitrate
    audio_bitrate = "128k"

    if not source_info["MIMEType"].startswith("video/"):
        raise IgnoreFileException()

    if "DisplayWidth" not in source_info:
        [w, h] = source_info["ImageSize"].split('x')
        source_info["DisplayWidth"] = w
        source_info["DisplayHeight"] = h

    probe_data = json.loads(subprocess.check_output(['ffprobe', '-show_format', '-of', 'json', source], stderr=subprocess.DEVNULL).decode('utf-8'))
    is_low_bitrate = int(probe_data["format"]["bit_rate"]) < (video_bitrate + 500000)
    is_hevc = "CompressorID" in source_info and source_info["CompressorID"] == "hev1"
    if is_hevc and is_low_bitrate:
        raise IgnoreFileException()

    scale_arg = "scale=0:0"
    origin_width = int(source_info["DisplayWidth"])
    # origin_height = int(source_info["DisplayHeight"])
    if origin_width > width:
        # factor = width / origin_width
        target_height = -1  # int(origin_height * factor)
        # if target_height != 0:
        #    target_height = target_height + 1
        scale_arg = "scale={}:{}".format(width, target_height)
    return ScanResult(
        source=source,
        source_dir=source_dir,
        source_basename=source_basename,
        scale_arg=scale_arg,
        audio_bitrate=audio_bitrate,
        temp_out=temp_out,
        temp_dir=temp_dir,
    )


def reencode(scan):
    command = [
        'ffmpeg',
        '-y',
        '-i', scan.source,
        '-c:v', 'libx265',
        '-preset', 'medium',
        '-crf', '28',
        '-vf', scan.scale_arg,
        '-c:a', 'aac',
        '-b:a', scan.audio_bitrate,
        '-f', 'mp4',
        '-hide_banner',
        '-nostats',
        '-v', 'warning',
        scan.temp_out,
    ]
    log.debug('--> %s', ' '.join(command))
    subprocess.check_call(command, cwd=scan.temp_dir, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def cleanup(scan, job_id):
    original_file_size = int(os.stat(scan.source).st_size / 1048576.0)
    new_file_size = int(os.stat(scan.temp_out).st_size / 1048576.0)
    dest_file = '{}/{}.mp4'.format(scan.source_dir, scan.source_basename)
    log.debug('copy %s -> %s', scan.temp_out, dest_file)
    shutil.copyfile(scan.temp_out, dest_file)
    if not os.path.samefile(scan.source, dest_file):
        log.debug('remove old video %s', scan.source)
        os.remove(scan.source)
    log.debug('remove tempdir %s', scan.temp_dir)
    shutil.rmtree(scan.temp_dir)
    return {'original_size_mb': original_file_size, 'new_size_mb': new_file_size, 'job_id': job_id}
