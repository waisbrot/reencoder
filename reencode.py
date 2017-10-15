#!/usr/bin/env python3

import argparse
import logging
import sys
import os.path as path
import tempfile
import json
import subprocess
import copy
import shutil
import os
import progressbar
from multiprocessing import Process, Queue

progressbar.streams.wrap_stderr()

logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s %(levelname)s %(filename)s:%(lineno)s: %(message)s',
    datefmt='%H:%M:%S'
)
log = logging.getLogger(__name__)

queue = Queue()

def start_pbar():
    def run_pbar(q):
        pmessage = progressbar.FormatCustomText('File: %(file)s -- %(status)s', dict(file='none', status='idle'))
        widgets = [
            progressbar.AnimatedMarker(),
            ' :: ', pmessage, ' :: ',
            progressbar.Timer(),
        ]
        pbar = progressbar.ProgressBar(widgets=widgets, max_value=progressbar.UnknownLength)
        while True:
            pbar.update()
            try:
                message = q.get(True, 1)
                pmessage.update_mapping(**message)
            except Exception:
                pass
    process = Process(target=run_pbar, args=(queue,), daemon=True)
    process.start()


def parse_args():
    parser = argparse.ArgumentParser(description="Re-encode video files")
    parser.add_argument('--width', type=int, required=False, default=1280, help="Max width of the video (height will be calculated automatically)")
    parser.add_argument('--encoding', type=str, required=False, default='h265', choices=['h264', 'h265'], help="Video encoding. Either h264 or h265")
    parser.add_argument('--bitrate', type=int, required=False, default=2000000, help="Video bit-rate")
    parser.add_argument('--file', type=str, required=True, help="File to re-encode")
    return parser.parse_args()

def scan_file(args):
    source = args.file
    queue.put({'file': path.basename(source), 'status': 'scanning'})
    (_, ext) = path.splitext(source)

    if ext in ['nfo', 'sub', 'idx', 'txt']:
        queue.put({'status': 'not video'})
        sys.exit(0)

    (source_basename, _) = path.splitext(path.basename(source))
    source_dir = path.dirname(source)
    temp_dir = tempfile.mkdtemp()
    temp_out = '{}/{}.mp4'.format(temp_dir, source_basename)
    pass_log = '{}/pass.log'.format(temp_dir)
    source_info = json.loads(subprocess.check_output(['exiftool', '-j', source]).decode('utf-8'))[0]
    video_bitrate = args.bitrate
    audio_bitrate = "128k"

    if not source_info["MIMEType"].startswith("video/"):
        queue.put({'status': 'not video'})
        sys.exit(0)

    if "DisplayWidth" not in source_info:
        [width, height] = source_info["ImageSize"].split('x')
        source_info["DisplayWidth"] = width
        source_info["DisplayHeight"] = height

    probe_data = json.loads(subprocess.check_output(['ffprobe', '-show_format', '-of', 'json', source], stderr=subprocess.DEVNULL).decode('utf-8'))
    is_low_bitrate = int(probe_data["format"]["bit_rate"]) < (video_bitrate + 500000)
    is_hevc = "CompressorID" in source_info and source_info["CompressorID"] == "hev1"
    if is_hevc and is_low_bitrate:
        queue.put({'status': 'already a low bit-rate HEVC'})
        sys.exit(0)

    scale_arg = "scale=0:0"
    origin_width = int(source_info["DisplayWidth"])
    origin_height = int(source_info["DisplayHeight"])
    if origin_width > args.width:
        factor = args.width / origin_width
        target_height = int(origin_height * factor)
        if target_height != 0:
            target_height = target_height + 1
        scale_arg = "scale={}:{}".format(args.width, target_height)
    return {
        'source': source,
        'source_dir': source_dir,
        'source_basename': source_basename,
        'scale_arg': scale_arg,
        'audio_bitrate': audio_bitrate,
        'temp_out': temp_out,
        'temp_dir': temp_dir,
    }

def reencode(source=None, scale_arg=None, audio_bitrate=None, temp_out=None, temp_dir=None, **kwargs):
    command = [
        'ffmpeg',
        '-y',
        '-i', source,
        '-c:v', 'libx265',
        '-preset', 'medium',
        '-crf', '28',
        '-vf', scale_arg,
        '-c:a', 'aac',
        '-b:a', audio_bitrate,
        '-f', 'mp4',
        '-hide_banner',
        '-nostats',
        '-v', 'warning',
        temp_out,
    ]
    queue.put({'status': 'encoding'})
    subprocess.check_call(command, cwd=temp_dir, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def cleanup(source=None, source_dir=None, source_basename=None, temp_out=None, temp_dir=None, **kwargs):
    queue.put({'status': 'cleanup'})
    dest_file = '{}/{}.mp4'.format(source_dir, source_basename)
    shutil.copyfile(temp_out, dest_file)
    os.remove(source)
    shutil.rmtree(temp_dir)
    queue.put({'status': 'done'})

try:
    start_pbar()
    args = parse_args()
    kwargs = scan_file(args)
    reencode(**kwargs)
    cleanup(**kwargs)
finally:
    pass
