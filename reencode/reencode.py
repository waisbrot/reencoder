'''
Functions to perform video re-encoding (by calling ExifTool and ffmpeg)
'''

import os.path as path
import time
import tempfile
import json
import subprocess
import shutil
import os
from collections import namedtuple
import logging


log = logging.getLogger('reencoder')


ScanResult = namedtuple('ScanResult',
                        [
                            'source', 'source_dir', 'source_basename', 'scale_arg',
                            'audio_bitrate', 'temp_out', 'temp_dir',
                        ])


class IgnoreFileException(Exception):
    pass


MAX_REQ_SIZE = 2048


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


def cleanup(scan):
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
    return {'original_size_mb': original_file_size, 'new_size_mb': new_file_size}
