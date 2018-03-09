FROM jrottenberg/ffmpeg:3.4-centos
LABEL maintainer="code@waisbrot.net" \
      name="reencode" \
      vendor="code@waisbrot.net"
WORKDIR /opt/reencode
ENTRYPOINT ["/usr/bin/python3", "-m", "reencode", "--live-progress", "-vvv", "--file"]
RUN yum install -q -y epel-release && yum install -q -y python34 python34-pip perl-Image-ExifTool
ADD requirements.txt .
RUN pip3 install -r requirements.txt
ADD . .

