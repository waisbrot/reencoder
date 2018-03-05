import os.path as path
import sys
import tempfile
import json
import subprocess
import shutil
import os
from collections import namedtuple
import logging


log = logging.getLogger('reencoder')


ScanResult = namedtuple('ScanResult',
                        ['source', 'source_dir', 'source_basename', 'scale_arg', 'audio_bitrate', 'temp_out', 'temp_dir'])


class IgnoreFileException(Exception):
    pass


def process(file_, bitrate, width, queue):
    try:
        scan_result = scan_file(file_, bitrate, width, queue)
        reencode(scan_result, queue)
        cleanup(scan_result, queue)
    except IgnoreFileException:
        pass
    except subprocess.CalledProcessError:
        log.exception('CalledProcessError')
        queue.put({'status': 'error'})
    except Exception:
        log.exception('Error processing file')
        queue.put({'status': 'error'})


def scan_file(file_, bitrate, width, queue):
    source = file_
    queue.put({'file': path.basename(source), 'status': 'scanning'})
    (_, ext) = path.splitext(source)

    (source_basename, _) = path.splitext(path.basename(source))
    source_dir = path.dirname(source)
    temp_dir = tempfile.mkdtemp()
    temp_out = '{}/{}.mp4'.format(temp_dir, source_basename)
    source_info = json.loads(subprocess.check_output(['exiftool', '-j', source]).decode('utf-8'))[0]
    video_bitrate = bitrate
    audio_bitrate = "128k"

    if not source_info["MIMEType"].startswith("video/"):
        queue.put({'status': 'not video'})
        raise IgnoreFileException()

    if "DisplayWidth" not in source_info:
        [w, h] = source_info["ImageSize"].split('x')
        source_info["DisplayWidth"] = w
        source_info["DisplayHeight"] = h

    probe_data = json.loads(subprocess.check_output(['ffprobe', '-show_format', '-of', 'json', source], stderr=subprocess.DEVNULL).decode('utf-8'))
    is_low_bitrate = int(probe_data["format"]["bit_rate"]) < (video_bitrate + 500000)
    is_hevc = "CompressorID" in source_info and source_info["CompressorID"] == "hev1"
    if is_hevc and is_low_bitrate:
        queue.put({'status': 'already a low bit-rate HEVC'})
        raise IgnoreFileException()

    scale_arg = "scale=0:0"
    origin_width = int(source_info["DisplayWidth"])
    origin_height = int(source_info["DisplayHeight"])
    if origin_width > width:
        factor = width / origin_width
        target_height = -1 #int(origin_height * factor)
        #if target_height != 0:
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


def reencode(scan, queue):
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
    queue.put({'status': 'encoding'})
    log.debug('--> %s', ' '.join(command))
    subprocess.check_call(command, cwd=scan.temp_dir, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def cleanup(scan, queue):
    queue.put({'status': 'cleanup'})
    dest_file = '{}/{}.mp4'.format(scan.source_dir, scan.source_basename)
    log.debug('copy %s -> %s', scan.temp_out, dest_file)
    shutil.copyfile(scan.temp_out, dest_file)
    if not os.path.samefile(scan.source, dest_file):
        log.debug('remove old video %s', scan.source)
        os.remove(scan.source)
    log.debug('remove tempdir %s', scan.temp_dir)
    shutil.rmtree(scan.temp_dir)
    queue.put({'status': 'done'})
